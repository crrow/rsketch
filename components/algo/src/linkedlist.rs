mod bad_single_linked_stack {
    use std::mem;

    #[derive(Debug)]
    pub struct List<T> {
        // the list's type size is as exactly as Link, zero-abstraction
        // for keeping internal structure private
        head: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> List<T> {
            List { head: Link::Empty }
        }
        // push a new element to the head of the list
        pub fn push(&mut self, elem: T) {
            let new_node = Box::new(Node {
                elem,
                next: mem::replace(&mut self.head, Link::Empty),
            });
            self.head = Link::More(new_node); // make the new node as the head
        }
        pub fn pop(&mut self) -> Option<T> {
            // we want to get the value out from the mutable reference,
            // so we use mem::replace to take the value out and replace it with Link::Empty.
            match mem::replace(&mut self.head, Link::Empty) {
                Link::Empty => None, // if the head is empty, return None
                Link::More(node) => {
                    // if the head has more, we want to get the value out
                    // and make the next node as the head
                    self.head = node.next;
                    Some(node.elem)
                }
            }
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            let mut cur = mem::replace(&mut self.head, Link::Empty);
            while let Link::More(mut boxed_node) = cur {
                cur = mem::replace(&mut boxed_node.next, Link::Empty);
            }
        }
    }

    #[derive(Debug)]
    enum Link<T> {
        Empty,
        More(Box<Node<T>>),
    }
    #[derive(Debug)]
    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn t() {
            let mut list = List::new();
            assert_eq!(list.pop(), None);
            list.push(1);
            list.push(2);
            list.push(3);
            assert_eq!(list.pop(), Some(3));
            assert_eq!(list.pop(), Some(2));
            list.push(4);
            list.push(5);
            assert_eq!(list.pop(), Some(5));
            assert_eq!(list.pop(), Some(4));
            assert_eq!(list.pop(), Some(1));
            assert_eq!(list.pop(), None);
            assert_eq!(list.pop(), None);
        }
    }
}
mod ok_stack {
    use std::{mem, ops::Deref};

    #[derive(Debug)]
    pub struct List<T> {
        // the list's type size is as exactly as Link, zero-abstraction
        // for keeping internal structure private
        head: Option<Box<Node<T>>>,
    }

    impl<T> List<T> {
        pub fn new() -> List<T> {
            List { head: None }
        }
        // push a new element to the head of the list
        pub fn push(&mut self, elem: T) {
            let head = self.head.take();
            let new_node = Box::new(Node { elem, next: head });
            self.head = Some(new_node); // make the new node as the head
        }
        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                self.head = node.next;
                node.elem
            })
        }
        pub fn peek(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.elem)
        }
        pub fn peek_mut(&mut self) -> Option<&mut T> {
            self.head.as_mut().map(|node| &mut node.elem)
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            while let Some(mut boxed_node) = self.head.take() {
                self.head = boxed_node.next.take()
            }
        }
    }

    pub struct IntoIter<T>(List<T>);

    impl<T> List<T> {
        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter(self)
        }
    }
    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop()
        }
    }

    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<T> {
            Iter {
                // next: self.head.as_ref().map(|node| node.as_ref()),
                next: self.head.as_deref(),
            }
        }
    }
    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                self.next = node.next.as_deref();
                &node.elem
            })
        }
    }

    pub struct IterMut<'a, T> {
        next: Option<&'a mut Node<T>>,
    }
    impl<T> List<T> {
        fn iter_mut(&mut self) -> IterMut<'_, T> {
            IterMut {
                next: self.head.as_deref_mut(),
            }
        }
    }
    impl<'a, T> Iterator for IterMut<'a, T> {
        type Item = &'a mut T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.take().map(|node| {
                self.next = node.next.as_deref_mut();
                &mut node.elem
            })
        }
    }

    #[derive(Debug)]
    struct Node<T> {
        elem: T,
        next: Option<Box<Node<T>>>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn t() {
            let mut list = List::new();
            assert_eq!(list.pop(), None);
            list.push(1);
            list.push(2);
            list.push(3);
            assert_eq!(list.pop(), Some(3));
            assert_eq!(list.pop(), Some(2));
            list.push(4);
            list.push(5);
            assert_eq!(list.pop(), Some(5));
            assert_eq!(list.pop(), Some(4));
            assert_eq!(list.pop(), Some(1));
            assert_eq!(list.pop(), None);
            assert_eq!(list.pop(), None);
        }
        #[test]
        fn peek() {
            let mut list = List::new();
            assert_eq!(list.peek_mut(), None);
            assert_eq!(list.peek(), None);
            list.push(1);
            list.push(2);
            list.push(3);
            assert_eq!(list.peek_mut(), Some(&mut 3));
            list.peek_mut().map(|value| *value = 42);
            assert_eq!(list.peek(), Some(&42));
            assert_eq!(list.pop(), Some(42));
        }

        #[test]
        fn into_iter() {
            let mut list = List::new();
            list.push(1);
            list.push(2);
            list.push(3);
            let mut iter = list.into_iter();
            assert_eq!(iter.next().unwrap(), 3);
            assert_eq!(iter.next().unwrap(), 2);
            assert_eq!(iter.next().unwrap(), 1);
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn iter() {
            let mut list = List::new();
            list.push(1);
            list.push(2);
            list.push(3);
            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(&3));
            assert_eq!(iter.next(), Some(&2));
            assert_eq!(iter.next(), Some(&1));
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn iter_mut() {
            let mut list = List::new();
            list.push(1);
            list.push(2);
            list.push(3);
            let mut iter = list.iter_mut();
            assert_eq!(iter.next(), Some(&mut 3));
            assert_eq!(iter.next(), Some(&mut 2));
            assert_eq!(iter.next(), Some(&mut 1));
            assert_eq!(iter.next(), None);
        }
    }
}

mod persistent_singly_linked_list {
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> List<T> {
            List { head: None }
        }

        // takes a list and an element, and returns a new list with that element at the
        // front
        pub fn prepend(&self, elem: T) -> List<T> {
            List {
                head: Some(Rc::new(Node {
                    elem,
                    next: self.head.clone(),
                })),
            }
        }

        // takes a list and returns the same list with the first element removed
        pub fn tail(&self) -> List<T> {
            List {
                head: self.head.as_ref().and_then(|node| node.next.clone()),
            }
        }

        pub fn head(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.elem)
        }

        pub fn iter(&self) -> Iter<'_, T> {
            Iter {
                // next: self.head.as_ref().map(|node| node.as_ref()),
                // have Option<Rc<Node<T>>> want Option<&Node<T>>
                next: self.head.as_deref(),
            }
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            let mut head = self.head.take();
            while let Some(n) = head {
                // try to know if there is only one reference to the node
                if let Ok(mut node) = Rc::try_unwrap(n) {
                    head = node.next.take();
                } else {
                    break;
                }
            }
        }
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                self.next = node.next.as_deref();
                &node.elem
            })
        }
    }

    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    type Link<T> = Option<Rc<Node<T>>>;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn basic() {
            let list = List::new();
            assert_eq!(list.head(), None);

            let list = list.prepend(1).prepend(2).prepend(3);
            assert_eq!(list.head(), Some(&3));

            let list = list.tail();
            assert_eq!(list.head(), Some(&2));

            let list = list.tail();
            assert_eq!(list.head(), Some(&1));

            let list = list.tail();
            assert_eq!(list.head(), None);

            let list = list.tail();
            assert_eq!(list.head(), None)
        }

        #[test]
        fn iter() {
            let list = List::new();
            assert_eq!(list.head(), None);

            let list = list.prepend(1).prepend(2).prepend(3);
            assert_eq!(list.head(), Some(&3));

            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(&3));
            assert_eq!(iter.next(), Some(&2));
            assert_eq!(iter.next(), Some(&1));
        }
    }
}

mod bad_safe_double_linked_deque {
    use std::{
        cell::{Ref, RefCell, RefMut},
        mem,
        ops::Deref,
        rc::Rc,
    };

    pub struct List<T> {
        head: Link<T>,
        tail: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            Self {
                head: None,
                tail: None,
            }
        }

        pub fn push_front(&mut self, elem: T) {
            let new_node = Rc::new(RefCell::new(Node::new(elem)));
            match self.head.take() {
                Some(old_head) => {
                    old_head.borrow_mut().prev = Some(new_node.clone());
                    new_node.borrow_mut().next = Some(old_head);
                    self.head = Some(new_node);
                }
                None => {
                    self.tail = Some(new_node.clone());
                    self.head = Some(new_node);
                }
            }
        }

        pub fn push_back(&mut self, elem: T) {
            let new_tail = Rc::new(RefCell::new(Node::new(elem)));
            match self.tail.take() {
                Some(old_tail) => {
                    old_tail.borrow_mut().next = Some(new_tail.clone());
                    new_tail.borrow_mut().prev = Some(old_tail);
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head = Some(new_tail.clone());
                    self.tail = Some(new_tail);
                }
            }
        }

        pub fn pop_front(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                match node.borrow_mut().next.take() {
                    Some(new_head) => {
                        new_head.borrow_mut().prev.take();
                        self.head = Some(new_head);
                    }
                    None => {
                        self.tail.take();
                    }
                };
                Rc::try_unwrap(node).ok().unwrap().into_inner().elem
            })
        }

        pub fn pop_back(&mut self) -> Option<T> {
            self.tail.take().map(|old_tail| {
                match old_tail.borrow_mut().prev.take() {
                    // take the tail's previous node
                    Some(new_tail) => {
                        new_tail.borrow_mut().next.take();
                        self.tail = Some(new_tail);
                    }
                    None => {
                        self.head.take();
                    }
                };
                Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
            })
        }

        pub fn peek_front(&self) -> Option<Ref<T>> {
            self.head
                .as_ref()
                .map(|node| Ref::map(node.borrow(), |node| &node.elem))
        }

        pub fn peek_back(&self) -> Option<Ref<T>> {
            self.tail
                .as_ref()
                .map(|node| Ref::map(node.borrow(), |node| &node.elem))
        }

        pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
            self.tail
                .as_ref()
                .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
        }

        pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
            self.head
                .as_ref()
                .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
        }
    }

    pub struct IntoIter<T>(List<T>);

    impl<T> List<T> {
        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter(self)
        }
    }
    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            while self.pop_front().is_some() {}
        }
    }

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }

    impl<T> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.0.pop_back()
        }
    }

    // pub struct Iter<T>(Option<Rc<Node<T>>>);

    // impl<T> List<T> {
    //     pub fn iter(&self) -> Iter<T> {
    //         Iter(self.head.as_ref().map(|head| head.clone()))
    //     }
    // }

    // impl<T> Iterator for Iter<T> {
    //     type Item = T;
    //     fn next(&mut self) -> Option<Self::Item> {
    //         self.0.take().map(|node_ref| {
    //             let (next, elem) = Ref::map_split(node_ref, |node| (&node.next, &node.elem));
    //             self.0 = next.as_ref().map(|head| head.borrow());

    //             elem
    //         })
    //     }
    // }

    type Link<T> = Option<Rc<RefCell<Node<T>>>>;
    struct Node<T> {
        elem: T,
        next: Link<T>,
        prev: Link<T>,
    }

    impl<T> Node<T> {
        fn new(t: T) -> Self {
            Self {
                elem: t,
                next: None,
                prev: None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn basic() {
            let mut list = List::new();
            assert_eq!(list.pop_front(), None);

            list.push_front(1);
            list.push_front(2);
            list.push_front(3);

            assert_eq!(list.pop_front(), Some(3));
            assert_eq!(list.pop_front(), Some(2));

            list.push_front(4);
            list.push_front(5);

            assert_eq!(list.pop_front(), Some(5));
            assert_eq!(list.pop_front(), Some(4));
            assert_eq!(list.pop_front(), Some(1));
            assert_eq!(list.pop_front(), None);

            // Check empty list behaves right
            assert_eq!(list.pop_back(), None);

            // Populate list
            list.push_back(1);
            list.push_back(2);
            list.push_back(3);

            // Check normal removal
            assert_eq!(list.pop_back(), Some(3));
            assert_eq!(list.pop_back(), Some(2));

            // Push some more just to make sure nothing's corrupted
            list.push_back(4);
            list.push_back(5);

            // Check normal removal
            assert_eq!(list.pop_back(), Some(5));
            assert_eq!(list.pop_back(), Some(4));

            // Check exhaustion
            assert_eq!(list.pop_back(), Some(1));
            assert_eq!(list.pop_back(), None);
        }

        #[test]
        fn peek() {
            let mut list = List::new();
            assert!(list.peek_front().is_none());
            assert!(list.peek_back().is_none());
            assert!(list.peek_front_mut().is_none());
            assert!(list.peek_back_mut().is_none());

            list.push_front(1);
            list.push_front(2);
            list.push_front(3);

            assert_eq!(&*list.peek_front().unwrap(), &3);
            assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
            assert_eq!(&*list.peek_back().unwrap(), &1);
            assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
        }

        #[test]
        fn into_iter() {
            let mut list = List::new();
            list.push_front(1);
            list.push_front(2);
            list.push_front(3);

            let mut iter = list.into_iter();
            assert_eq!(iter.next(), Some(3));
            assert_eq!(iter.next_back(), Some(1));
            assert_eq!(iter.next(), Some(2));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }
    }
}

mod ok_unsafe_singly_linked_queue {
    use std::{mem, ptr};

    pub struct List<T> {
        head: Link<T>,
        tail: *mut Node<T>,
    }
    type Link<T> = Option<Box<Node<T>>>;

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List {
                head: None,
                tail: ptr::null_mut(),
            }
        }

        pub fn push(&mut self, elem: T) {
            let mut new_tail = Box::new(Node { elem, next: None });
            let raw_tail: *mut _ = &mut *new_tail;
            if !self.tail.is_null() {
                // old tail exists
                unsafe {
                    (*self.tail).next = Some(new_tail);
                }
            } else {
                // old tail doesn't exist
                self.head = Some(new_tail);
            }
            self.tail = raw_tail;
        }

        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                let new_head = node.next;
                self.head = new_head;
                if self.head.is_none() {
                    self.tail = ptr::null_mut();
                }
                node.elem
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(test)]
        mod test {
            use super::List;
            #[test]
            fn basics() {
                let mut list = List::new();

                // Check empty list behaves right
                assert_eq!(list.pop(), None);

                // Populate list
                list.push(1);
                list.push(2);
                list.push(3);

                // Check normal removal
                assert_eq!(list.pop(), Some(1));
                assert_eq!(list.pop(), Some(2));

                // Push some more just to make sure nothing's corrupted
                list.push(4);
                list.push(5);

                // Check normal removal
                assert_eq!(list.pop(), Some(3));
                assert_eq!(list.pop(), Some(4));

                // Check exhaustion
                assert_eq!(list.pop(), Some(5));
                assert_eq!(list.pop(), None);

                // Check the exhaustion case fixed the pointer right
                list.push(6);
                list.push(7);

                // Check normal removal
                assert_eq!(list.pop(), Some(6));
                assert_eq!(list.pop(), Some(7));
                assert_eq!(list.pop(), None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miri() {
        unsafe {
            let mut data = 10;
            let ref1 = &mut data;
            let ptr2 = ref1 as *mut _;

            // ORDER SWAPPED!
            *ref1 += 1;
            *ptr2 += 2;

            println!("{}", data);
        }
    }

    #[test]
    fn shared_reference() {
        fn opaque_read(val: &i32) {
            println!("{}", val)
        }
        unsafe {
            let mut data = 10;
            let mref1 = &mut data;
            let sref2 = &mref1;
            let sref3 = sref2;
            let sref4 = &*sref2;
            opaque_read(sref3);
            opaque_read(sref2);
            opaque_read(sref4);
            opaque_read(sref2);
            opaque_read(sref3);
            *mref1 += 1;
            opaque_read(&data);
        }

        unsafe {
            let mut data = 10;
            let mref1 = &mut data;
            let ptr2 = mref1 as *mut i32;
            let sref3 = &*mref1;
            let ptr4 = sref3 as *const i32 as *mut i32;
            *ptr4 += 4;
            opaque_read(&sref3);
        }
    }

    #[test]
    fn interior_mutability() {
        use std::cell::Cell;

        let mut data = Cell::new(10);
        let mref1 = &mut data;
        let pte2 = mref1 as *mut Cell<i32>;
        let sref3 = &*mref1;
        sref3.set(sref3.get() + 3);
        unsafe {
            (*pte2).set((*pte2).get() + 4);
        }
        mref1.set(mref1.get() + 1);
        println!("{}", data.get());
    }
}
