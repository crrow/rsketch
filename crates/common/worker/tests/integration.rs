use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use rsketch_common_worker::{Manager, Trigger, Worker, WorkerConfig, WorkerContext};
use tokio::time::sleep;

struct CounterWorker {
    counter: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl Worker for CounterWorker {
    fn name() -> &'static str { "CounterWorker" }

    fn trigger() -> Trigger { Trigger::Interval(Duration::from_millis(100)) }

    async fn work(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct NotifyWorker {
    counter: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl Worker for NotifyWorker {
    fn name() -> &'static str { "NotifyWorker" }

    fn trigger() -> Trigger { Trigger::Notify }

    async fn work(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct OnceWorker {
    counter: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl Worker for OnceWorker {
    fn name() -> &'static str { "OnceWorker" }

    fn trigger() -> Trigger { Trigger::Once }

    async fn work(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_interval_worker() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let _handle = manager.register(CounterWorker {
        counter: counter.clone(),
    });

    sleep(Duration::from_millis(550)).await;

    manager.shutdown().await.unwrap();

    let final_count = counter.load(Ordering::SeqCst);
    assert!(
        (4..=6).contains(&final_count),
        "Expected 4-6 ticks, got {}",
        final_count
    );
}

#[tokio::test]
async fn test_notify_worker() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let handle = manager.register(NotifyWorker {
        counter: counter.clone(),
    });

    sleep(Duration::from_millis(50)).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Should not execute without notification"
    );

    for _ in 0..3 {
        handle.notify();
        sleep(Duration::from_millis(50)).await;
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        3,
        "Should execute exactly 3 times"
    );

    manager.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_once_worker() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let _handle = manager.register(OnceWorker {
        counter: counter.clone(),
    });

    sleep(Duration::from_millis(100)).await;

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Should execute exactly once"
    );

    sleep(Duration::from_millis(200)).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "Should still be 1 after waiting"
    );

    manager.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_multiple_workers() {
    let counter1 = Arc::new(AtomicU32::new(0));
    let counter2 = Arc::new(AtomicU32::new(0));

    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let handle1 = manager.register(NotifyWorker {
        counter: counter1.clone(),
    });

    let _handle2 = manager.register(CounterWorker {
        counter: counter2.clone(),
    });

    sleep(Duration::from_millis(50)).await;

    handle1.notify();
    handle1.notify();

    sleep(Duration::from_millis(250)).await;

    assert_eq!(counter1.load(Ordering::SeqCst), 2);
    assert!(counter2.load(Ordering::SeqCst) >= 2);

    manager.shutdown().await.unwrap();
}

struct HangingWorker {
    hang_duration: Duration,
}

#[async_trait::async_trait]
impl Worker for HangingWorker {
    fn name() -> &'static str { "HangingWorker" }

    fn trigger() -> Trigger { Trigger::Once }

    async fn work(&mut self, ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        tokio::select! {
            _ = sleep(self.hang_duration) => {
                // Simulate a worker that doesn't respect cancellation
            }
            _ = ctx.cancelled() => {
                // This worker ignores cancellation
                sleep(self.hang_duration).await;
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_shutdown_timeout() {
    let config = WorkerConfig::builder()
        .shutdown_timeout(Duration::from_millis(200))
        .build();
    let mut manager = Manager::start(config).unwrap();

    // Spawn a worker that will hang during shutdown
    let _handle = manager.register(HangingWorker {
        hang_duration: Duration::from_secs(10),
    });

    sleep(Duration::from_millis(50)).await;

    let start = std::time::Instant::now();
    manager.shutdown().await.unwrap();
    let elapsed = start.elapsed();

    // Should timeout around 200ms, not wait for 10 seconds
    assert!(
        elapsed < Duration::from_millis(500),
        "Shutdown took {:?}, expected < 500ms",
        elapsed
    );
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = WorkerConfig::builder()
        .shutdown_timeout(Duration::from_secs(5))
        .build();
    let mut manager = Manager::start(config).unwrap();

    let _handle = manager.register(CounterWorker {
        counter: counter.clone(),
    });

    sleep(Duration::from_millis(250)).await;

    let start = std::time::Instant::now();
    manager.shutdown().await.unwrap();
    let elapsed = start.elapsed();

    // Should shutdown quickly (well before timeout) for cooperative workers
    assert!(
        elapsed < Duration::from_millis(500),
        "Shutdown took {:?}, expected < 500ms for cooperative worker",
        elapsed
    );
}

struct LifecycleWorker {
    started:    Arc<AtomicU32>,
    shutdown:   Arc<AtomicU32>,
    work_count: Arc<AtomicU32>,
}

#[async_trait::async_trait]
impl Worker for LifecycleWorker {
    fn name() -> &'static str { "LifecycleWorker" }

    fn trigger() -> Trigger { Trigger::Interval(Duration::from_millis(100)) }

    async fn on_start(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.started.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn work(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.work_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn on_shutdown(&mut self, _ctx: &WorkerContext) -> rsketch_common_worker::Result<()> {
        self.shutdown.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_lifecycle_hooks() {
    let started = Arc::new(AtomicU32::new(0));
    let shutdown = Arc::new(AtomicU32::new(0));
    let work_count = Arc::new(AtomicU32::new(0));

    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let _handle = manager.register(LifecycleWorker {
        started:    started.clone(),
        shutdown:   shutdown.clone(),
        work_count: work_count.clone(),
    });

    sleep(Duration::from_millis(350)).await;

    assert_eq!(
        started.load(Ordering::SeqCst),
        1,
        "on_start should be called once"
    );
    assert!(
        work_count.load(Ordering::SeqCst) >= 2,
        "work should be called multiple times"
    );
    assert_eq!(
        shutdown.load(Ordering::SeqCst),
        0,
        "on_shutdown should not be called yet"
    );

    manager.shutdown().await.unwrap();

    assert_eq!(
        shutdown.load(Ordering::SeqCst),
        1,
        "on_shutdown should be called once"
    );
}

#[tokio::test]
async fn test_worker_pause_resume() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = WorkerConfig::builder().build();
    let mut manager = Manager::start(config).unwrap();

    let handle = manager.register(CounterWorker {
        counter: counter.clone(),
    });

    // Let it run for a bit
    sleep(Duration::from_millis(250)).await;
    let count_before_pause = counter.load(Ordering::SeqCst);
    assert!(
        count_before_pause >= 2,
        "Should have executed at least twice"
    );

    // Pause the worker
    handle.pause();
    assert!(handle.is_paused());
    sleep(Duration::from_millis(250)).await;

    // Counter should not increase while paused
    let count_while_paused = counter.load(Ordering::SeqCst);
    assert_eq!(
        count_before_pause, count_while_paused,
        "Counter should not increase while paused"
    );

    // Resume the worker
    handle.resume();
    assert!(!handle.is_paused());
    sleep(Duration::from_millis(250)).await;

    // Counter should increase again
    let count_after_resume = counter.load(Ordering::SeqCst);
    assert!(
        count_after_resume > count_while_paused,
        "Counter should increase after resume"
    );

    manager.shutdown().await.unwrap();
}
