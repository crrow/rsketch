use std::{marker::PhantomData, ptr::NonNull};

struct LinkedList<L, T> {
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
    _marker: PhantomData<*const L>,
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{mpsc, Arc, Mutex},
        thread,
    };

    use super::*;
    #[test]
    fn test_threads() {
        let handle = thread::spawn(|| "hello from a thread");

        println!("Hello from main thread");
        println!("{}", handle.join().unwrap());

        let x = 1;
        let handle = thread::spawn(move || {
            println!("x is {}", x);
        });
        handle.join().unwrap();

        let data = Arc::new(Mutex::new(9));
        let (tx, rx) = mpsc::channel();
        for _ in 0..10 {
            let (data, tx) = (data.clone(), tx.clone());
            thread::spawn(move || {
                let mut data = data.lock().unwrap();
                *data += 1;
                tx.send(()).unwrap();
            });
        }
        for _ in 0..10 {
            rx.recv().unwrap();
        }
    }
}
