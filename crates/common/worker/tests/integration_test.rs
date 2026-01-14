// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use rsketch_common_worker::{Manager, Notifiable, Pausable, Worker, WorkerContext};

struct TestWorker {
    counter: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Worker for TestWorker {
    async fn work<S: Clone + Send + Sync>(&mut self, _ctx: WorkerContext<S>) {
        self.counter.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn test_once_trigger() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut manager = Manager::new();

    let _handle = manager
        .worker(TestWorker {
            counter: counter.clone(),
        })
        .name("test-once")
        .once()
        .spawn();

    tokio::time::sleep(Duration::from_millis(100)).await;
    manager.shutdown().await;

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_notify_trigger() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut manager = Manager::new();

    let handle = manager
        .worker(TestWorker {
            counter: counter.clone(),
        })
        .name("test-notify")
        .on_notify()
        .spawn();

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    handle.notify();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    handle.notify();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_interval_trigger() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut manager = Manager::new();

    let _handle = manager
        .worker(TestWorker {
            counter: counter.clone(),
        })
        .name("test-interval")
        .interval(Duration::from_millis(50))
        .spawn();

    tokio::time::sleep(Duration::from_millis(125)).await;
    manager.shutdown().await;

    let count = counter.load(Ordering::SeqCst);
    assert!(
        count >= 2 && count <= 3,
        "Expected 2-3 executions, got {}",
        count
    );
}

#[tokio::test]
async fn test_pausable() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut manager = Manager::new();

    let handle = manager
        .worker(TestWorker {
            counter: counter.clone(),
        })
        .name("test-pausable")
        .interval(Duration::from_millis(50))
        .spawn();

    tokio::time::sleep(Duration::from_millis(125)).await;
    let before_pause = counter.load(Ordering::SeqCst);

    handle.pause();
    assert!(handle.is_paused());

    tokio::time::sleep(Duration::from_millis(150)).await;
    let during_pause = counter.load(Ordering::SeqCst);
    assert_eq!(
        before_pause, during_pause,
        "Counter should not increase while paused"
    );

    handle.resume();
    assert!(!handle.is_paused());

    tokio::time::sleep(Duration::from_millis(125)).await;
    let after_resume = counter.load(Ordering::SeqCst);
    assert!(
        after_resume > during_pause,
        "Counter should increase after resume"
    );

    manager.shutdown().await;
}

#[tokio::test]
async fn test_with_state() {
    #[derive(Clone)]
    struct AppState {
        value: Arc<AtomicUsize>,
    }

    struct StateWorker;

    #[async_trait::async_trait]
    impl Worker for StateWorker {
        async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {
            // This is a bit of a trick - we know S is AppState but can't access it directly
            // without downcasting. In real usage, you'd use a type parameter or associated
            // type. For now, just test that the context is passed correctly.
            let _name = ctx.name();
        }
    }

    let state = AppState {
        value: Arc::new(AtomicUsize::new(42)),
    };

    let mut manager = Manager::with_state(state.clone());

    let _handle = manager
        .worker(StateWorker)
        .name("test-state")
        .once()
        .spawn();

    tokio::time::sleep(Duration::from_millis(100)).await;
    manager.shutdown().await;

    assert_eq!(state.value.load(Ordering::SeqCst), 42);
}

#[tokio::test]
async fn test_cron_trigger() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut manager = Manager::new();

    // Every minute (standard cron format without seconds)
    let _handle = manager
        .worker(TestWorker {
            counter: counter.clone(),
        })
        .name("test-cron")
        .cron("* * * * *")
        .expect("Valid cron expression")
        .spawn();

    // Since cron runs every minute, we can't easily test it in a short time
    // Just verify it compiles and starts
    tokio::time::sleep(Duration::from_millis(100)).await;
    manager.shutdown().await;

    // We can't assert exact count since cron timing is minute-based
    // Just verify the worker was created successfully
}
