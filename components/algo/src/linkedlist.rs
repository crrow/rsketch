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
        cell::{Ref, RefCell},
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

        pub fn peek_front(&self) -> Option<Ref<T>> {
            self.head
                .as_ref()
                .map(|node| Ref::map(node.borrow(), |node| &node.elem))
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            while self.pop_front().is_some() {}
        }
    }

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
        }

        #[test]
        fn peek() {
            let mut list = List::new();
            assert_eq!(list.pop_front(), None);

            list.push_front(1);
            list.push_front(2);
            list.push_front(3);

            assert_eq!(&*list.peek_front().unwrap(), &3);
        }
    }
}
