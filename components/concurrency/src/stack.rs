use std::{
    ptr,
    ptr::null_mut,
    sync::atomic::{
        AtomicPtr,
        Ordering::{Acquire, Release},
    },
};

pub struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}
struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(null_mut()),
        }
    }
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Acquire);
            if head == null_mut() {
                return None;
            }
            let next = unsafe { (*head).next };
            if self
                .head
                .compare_exchange(head, next, Release, Acquire)
                .is_ok()
            {
                return Some(unsafe { ptr::read(&(*head).data) });
            }
        }
    }
    pub fn push(&self, t: T) {
        let n = Box::into_raw(Box::new(Node {
            data: t,
            next: null_mut(),
        }));
        loop {
            let head = self.head.load(Acquire);
            unsafe { (*n).next = head };
            if self
                .head
                .compare_exchange(head, n, Release, Acquire)
                .is_ok()
            {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack() {
        let s: Stack<u8> = Stack::new();
        s.push(1);
        let x = s.pop().unwrap();
        assert_eq!(x, 1);
    }
}
