#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize, Ordering};
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::Condvar;
    use std::thread;
    use std::time::Duration;
    use super::*;

    #[test]
    fn thread_demo() {
        use std::thread;

        fn f() {
            println!("Hello, world!");
            let id = thread::current().id();
            println!("Thread id: {:?}", id);
        }
        let t1 = thread::spawn(f);
        let t2 = thread::spawn(f);
        t1.join().unwrap();
        t2.join().unwrap();
        println!("Hello from the main thread.");

        // move a variable to a new thread
        let numbers = vec![1, 2, 3];
        thread::spawn(move || {
            for n in &numbers {
                println!("{n}");
            }
        }).join().unwrap();

        // getting a value back out of the thread
        let numbers = Vec::from_iter(0..=1000);
        let t = thread::spawn(move || {
            let len = numbers.len();
            let sum = numbers.iter().sum::<usize>();
            sum / len
        });
        let average = t.join().unwrap();
        println!("Average: {}", average);

        // scope of a thread
        let numbers = vec![1, 2, 3];
        thread::scope(|s| {
            s.spawn(|| {
                println!("len: {}", numbers.len());
            });
            s.spawn(|| {
                for n in &numbers {
                    println!("{}", n);
                }
            });
        });
    }

    #[test]
    fn interior_mutability() {
        // cell: it only allows you to copy the value out (if T is Copy),
        // or replace it with another value as a whole.
        // In addition, it can only be used within a single thread.
        fn cell_demo(v: &Cell<Vec<i32>>) {
            let mut v2 = v.take();
            v2.push(3);
            v.set(v2);
        }
        cell_demo(&Cell::new(vec![1, 2]));

        // ref_cell allows to borrow its contents,
        // if it is already mutable borrowed, it will panic
        // It can only be used in a single thread.
        fn ref_cell_demo() {
            use std::cell::RefCell;
            let v = RefCell::new(vec![1, 2]);
            let mut v2 = v.borrow_mut();
            v2.push(3);
            drop(v2);
            let v3 = v.borrow();
            println!("{:?}", v3);
        }
        ref_cell_demo();

        // rwlock is the concurrent version of a RefCell
        fn rwlock_demo() {
            use std::sync::RwLock;
            let v = RwLock::new(vec![1, 2]);
            let mut v2 = v.write().unwrap();
            v2.push(3);
            drop(v2);
            let v3 = v.read().unwrap();
            println!("{:?}", v3);
        }
        rwlock_demo();

        // atomic is the concurrent version of a Cell.
        // And the atomic type cannot wrap arbitrary type,cause it
        // requires support of underlying platform.
        fn atomic_demo() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let v = AtomicUsize::new(0);
            v.fetch_add(1, Ordering::SeqCst);
            println!("{}", v.load(Ordering::SeqCst));
        }

        // all above types provide the ability of interior mutability
        // are all built on the top of UnsafeCell<T>.
    }

    fn send_and_sync() {
        // The language uses two special traits to keep track of
        // which types can be safely used across threads:
        // 1. send
        // 2. sync

        // to opt out of the send and sync traits, we can use
        // the marker type PhantomData<T>

        use std::marker::PhantomData;

        struct X {
            handle: i32,
            // this is a zero field
            _not_sync: PhantomData<Cell<()>>,
        }
    }

    #[test]
    fn cond_ver() {
        use std::sync::Mutex;
        use std::collections::VecDeque;
        let queue = Mutex::new(VecDeque::new());
        let not_empty = Condvar::new();
        thread::scope(|s| {
            s.spawn(|| {
                loop {
                    let mut q = queue.lock().unwrap();
                    let item = loop {
                        if let Some(item) = q.pop_front() {
                            break item;
                        } else {
                            q = not_empty.wait(q).unwrap();
                        }
                    };
                    drop(q);
                    dbg!(item);
                }
            });
            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                not_empty.notify_one();
                thread::sleep(Duration::from_secs(i));
            }
        })
    }

    #[test]
    fn atomic_stop_flag() {
        static STOP: AtomicBool = AtomicBool::new(false);
        let background_thread = thread::spawn(|| {
            while !STOP.load(Ordering::Relaxed) {
                println!("working");
            }
        });
        for line in std::io::stdin().lines() {
            match line.unwrap().as_str() {
                "help" => println!("help"),
                "stop" => break,
                cmd => println!("unknown command: {}", cmd),
            }
        }
        STOP.store(true, Ordering::Relaxed);
        background_thread.join().unwrap();
    }

    #[test]
    fn atomic() {
        let num_done = AtomicUsize::new(0);
        let main_thread = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                for i in 0..100 {
                    println!("{}", i);
                    num_done.store(i + 1, Ordering::Relaxed);
                    main_thread.unpark();
                }
            });
            loop {
                let n = num_done.load(Ordering::Relaxed);
                if n == 100 {
                    break;
                }
                println!("Working.. {n}/100 done");
                thread::park_timeout(Duration::from_millis(100));
            }
        });

        println!("Done");
    }

    #[test]
    fn lazy_initialize() {
        // use std::sync::Once
    }

    #[test]
    fn cas() {
        let num_done = &AtomicUsize::new(0);
        thread::scope(|s| {
            for t in 0..4 {
                s.spawn(move || {
                    for i in 0..25 {
                        num_done.fetch_add(1, Relaxed);
                    }
                });
            }

            loop {
                let n = num_done.load(Relaxed);
                if n == 100 {
                    break;
                }
                println!("Working.. {n}/100 done");
                thread::sleep(Duration::from_secs(1));
            }
        });
        println!("Done!");
    }

    #[test]
    fn id_allocation() {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let mut id = NEXT_ID.load(Relaxed);
        loop {
            assert!(id < 1000);
            match NEXT_ID.compare_exchange_weak(id, id + 1, Relaxed, Relaxed) {
                Ok(_) => break,
                Err(x) => id = x,
            }
        }

        NEXT_ID.fetch_update(Relaxed, Relaxed,
                             |n| n.checked_add(1)).expect("too many IDs!");
    }

    #[test]
    fn ordering() {
        // static DATA: AtomicU32 = AtomicU32::new(0);
        static mut DATA: u32 = 0;
        static READY: AtomicBool = AtomicBool::new(false);
        thread::spawn(|| {
            unsafe {
                DATA = 123
            }
            READY.store(true, Release); // release applies to the store operation
        });
        while !READY.load(Acquire) { // acquire applies to the load operation
            thread::sleep(Duration::from_millis(1));
        }
        unsafe {
            assert_eq!(DATA, 123);
        }
    }

    #[test]
    fn mutex_but_atomic() {
        static mut DATA: String = String::new();
        static LOCK: AtomicBool = AtomicBool::new(false);
        fn f () {
            if LOCK.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
                unsafe {
                    DATA.push_str("hello");
                }
                LOCK.store(false, Release);
            }
        }
        thread::scope(|s| {
            for _ in 0..100 {
                s.spawn(f);
            }
        });
    }

    #[test]
    fn atomic_ptr() {
        struct Data {}
        static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());
        let mut p = PTR.load(Acquire);
        if p.is_null() {
            p = Box::into_raw(Box::new(Data {}));
            if let Err(e) = PTR.compare_exchange(
                std::ptr::null_mut(),
                p,
                Release,
                Acquire,
            ){
                drop(unsafe {
                    Box::from_raw(p)
                });
                p = e;
            }
        }
        unsafe {
            &*p;
        }
    }
}
