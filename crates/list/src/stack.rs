use core::slice;
use std::{
    alloc::{Allocator, Global, Layout, handle_alloc_error},
    fmt::Debug,
    hash::Hash,
    ops::{Index, IndexMut},
    ptr::{NonNull, drop_in_place},
};

use crate::{increment, Increment};

pub struct Stack<T, A = Global, C = Increment>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    /// an array with `self.cap` slots of T
    buf: *mut T,

    /// how much does `self.buf` can bear
    cap: usize,

    /// next slot to push, which means top - 1 is next position to pop
    top: usize,

    /// allocator
    alloc: A,

    /// increment calculator
    incre: C,
}

pub struct StackIter<T, A, C>(Stack<T, A, C>)
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize;

impl<T: Sized> Default for Stack<T, Global, Increment> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sized> Stack<T, Global, Increment> {
    #[inline]
    pub fn new() -> Self {
        Self::new_in(Global, increment)
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global, increment)
    }
}

impl<T: Sized + Clone> Stack<T, Global, Increment> {
    /// build a stack by cloning the elements of a slice
    ///
    /// ```rust
    /// # use list::stack::Stack;
    ///
    /// let a = Stack::from_slice(&["Hello", ", ", "world"]);
    /// let mut b = Stack::new();
    /// b.push("Hello");
    /// b.push(", ");
    /// b.push("world");
    ///
    /// assert_eq!(a, b);
    /// ```
    pub fn from_slice(slice: &[T]) -> Self {
        if slice.len() != 0 {
            let vec = slice.iter().cloned().collect::<Vec<_>>();
            return Self {
                cap: vec.capacity(),
                top: slice.len(),
                buf: vec.leak().as_mut_ptr(),
                ..Self::new()
            };
        }

        Self::new()
    }
}

impl<T, A, C> PartialEq for Stack<T, A, C>
where
    T: Sized + PartialEq,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }

    /// check if two stacks equals
    ///
    /// ```rust
    /// # use list::stack::Stack;
    ///
    /// let a = Stack::from_slice(&["Hello", ", ", "world"]);
    /// let mut b = Stack::new();
    /// b.push("Hello");
    /// b.push(", ");
    /// b.push("world");
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.as_slice();
        let rhs = other.as_slice();

        lhs == rhs
    }
}

impl<T: PartialEq, A: Allocator, C: FnMut(Layout, usize) -> usize> Eq for Stack<T, A, C> {}

impl<T: Hash, A: Allocator, C: FnMut(Layout, usize) -> usize> Hash for Stack<T, A, C> {
    /// compute hash value of self
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// use std::hash::{Hash, Hasher};
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// let (vec, mut vec_hash) = (vec!["Hello", "world"], DefaultHasher::new());
    /// let (stack, mut stack_hash) = (Stack::from_slice(vec.as_slice()), DefaultHasher::new());
    /// vec.hash(&mut vec_hash);
    /// stack.hash(&mut stack_hash);
    /// assert_eq!(vec_hash.finish(), stack_hash.finish());
    /// ```
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T, A, C> Iterator for StackIter<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Item = T;

    /// polls next value from stack
    ///
    /// ```rust
    /// # use list::stack::Stack;
    ///
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// stack.push(2);
    /// let mut iter = stack.into_iter();
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T, A, C> IntoIterator for Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Item = T;
    type IntoIter = StackIter<T, A, C>;

    /// consumes and turn itself into a [`StackIter`]
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StackIter(self)
    }
}

impl<T, A, C> Debug for Stack<T, A, C>
where
    T: Sized + Debug,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    /// debug format, bottom on the left, top on the right
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1); // bottom
    /// stack.push(2); // top
    /// assert_eq!(format!("{:?}", stack), "[1, 2]")
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<T: Sized, A: Allocator, C: FnMut(Layout, usize) -> usize> Drop for Stack<T, A, C> {
    fn drop(&mut self) {
        // first we call destructor on each slot, reversed, in other words,
        // we destruct values in stack output order
        for i in (0..self.top).rev() {
            unsafe { drop_in_place(self.buf.add(i)) }
        }

        let layout = Layout::array::<T>(self.cap).unwrap();

        match NonNull::new(self.buf as *mut u8) {
            Some(ptr) => unsafe { self.alloc.deallocate(ptr, layout) },
            None => debug_assert_eq!(self.cap, 0),
        }
    }
}

impl<T, A, C> Clone for Stack<T, A, C>
where
    T: Sized + Clone,
    A: Allocator + Clone,
    C: FnMut(Layout, usize) -> usize + Clone,
{
    fn clone(&self) -> Self {
        let cap = self.cap;
        let top = self.top;
        let alloc = self.alloc.clone();
        let incre = self.incre.clone();

        let buf = alloc
            .allocate(Layout::array::<T>(cap).expect("layout"))
            .expect("allocation")
            .as_ptr() as *mut T;

        for i in 0..top {
            let cloned = unsafe { &*self.buf.add(i) }.clone();
            unsafe { buf.add(i).write(cloned) }
        }

        Self {
            buf,
            cap,
            top,
            alloc,
            incre,
        }
    }
}

impl<T, A, C> IndexMut<usize> for Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    /// # Get mutable reference or [copy](Copy) of `index + 1`th element of the stack
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    /// stack[0] = '💖';
    /// assert_eq!(stack.pop(), Some('💖'));
    /// ```
    ///
    /// # Panics if index out of bound
    ///
    /// ```rust,should_panic
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    ///
    /// let _ = stack[1];   // panics!
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index stack beyond [0, top)")
    }
}

impl<T, A, C> Index<usize> for Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Output = T;

    /// # Get immutable reference or [copy](Copy) of `index + 1`th element of the stack
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    /// let first = stack[0];
    /// assert_eq!(stack.pop(), Some(first));
    /// ```
    ///
    /// ```rust,should_panic
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    ///
    /// let _ = stack[1];   // panics!
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index stack beyond [0, top)")
    }
}

impl<T, A, C> Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    #[inline]
    pub fn new_in(alloc: A, incre: C) -> Self {
        Self {
            buf: std::ptr::null_mut(),
            cap: 0,
            top: 0,
            alloc,
            incre,
        }
    }

    #[inline]
    pub fn with_capacity_in(cap: usize, alloc: A, incre: C) -> Self {
        let layout = Layout::array::<T>(cap).expect("layout");
        let ptr = alloc.allocate(layout).expect("allocation").as_ptr() as *mut _;

        Self {
            buf: ptr,
            cap,
            top: 0,
            alloc,
            incre,
        }
    }

    /// get `index`th element's reference of this stack, returns [`None`] if stack is empty
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// assert_eq!(stack.get(0), Some(1).as_ref());
    /// assert_eq!(stack.get(1), None);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.top {
            Some(unsafe { &*self.buf.add(index) })
        } else {
            None
        }
    }

    /// get `index`th element's mutable reference of this stack, returns [`None`] if stack is empty
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// assert_eq!(stack.get_mut(0), Some(1).as_mut());
    /// assert_eq!(stack.get(1), None);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.top {
            Some(unsafe { &mut *self.buf.add(index) })
        } else {
            None
        }
    }

    /// get inner data as a slice
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// assert_eq!(stack.as_slice(), &[]);
    /// stack.push(1);
    /// stack.push(2);
    /// assert_eq!(stack.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        if self.is_empty() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.buf, self.top) }
        }
    }

    /// get inner data as a mutable slice
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// assert_eq!(stack.as_slice_mut(), &mut []);
    /// stack.push(1);
    /// stack.push(2);
    /// assert_eq!(stack.as_slice_mut(), &mut [1, 2]);
    /// ```
    #[inline]
    pub fn as_slice_mut(&self) -> &mut [T] {
        if self.is_empty() {
            &mut []
        } else {
            unsafe { slice::from_raw_parts_mut(self.buf, self.top) }
        }
    }

    /// get top element's reference of this stack, returns [`None`] if stack is empty
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// assert_eq!(stack.peek(), Some(1).as_ref());
    /// assert_eq!(stack.pop(), Some(1));
    /// assert_eq!(stack.peek(), None);
    /// ```
    #[inline]
    pub fn peek(&self) -> Option<&T> {
        if self.top > 0 {
            Some(unsafe { &*self.buf.add(self.top - 1) })
        } else {
            None
        }
    }

    /// get top element's mutable reference of this stack, returns [`None`] if stack is empty
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// assert_eq!(stack.peek_mut(), Some(1).as_mut());
    /// assert_eq!(stack.pop(), Some(1));
    /// assert_eq!(stack.peek_mut(), None);
    /// ```
    #[inline]
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.top > 0 {
            Some(unsafe { &mut *self.buf.add(self.top - 1) })
        } else {
            None
        }
    }

    /// pushes value into this stack
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// stack.push(2);
    /// assert_eq!(stack.pop(), Some(2));
    /// assert_eq!(stack.pop(), Some(1));
    /// assert_eq!(stack.pop(), None);
    /// ```
    #[inline]
    pub fn push(&mut self, value: T) {
        if self.top == self.cap {
            let new_cap = (self.incre)(Layout::new::<T>(), self.cap);
            self.grow_to(new_cap);
        }

        let pos = self.top;
        self.top += 1;

        unsafe { self.buf.add(pos).write(value) };
    }

    /// pop value from this stack, returns [`None`] if this stack is empty
    ///
    /// ```rust
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(1);
    /// stack.push(2);
    /// assert_eq!(stack.pop(), Some(2));
    /// assert_eq!(stack.pop(), Some(1));
    /// assert_eq!(stack.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.top == 0 {
            return None;
        }

        self.top -= 1;

        Some(unsafe { self.buf.add(self.top).read() })
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.cap == self.top
    }

    /// # Grows self to `new_cap`
    ///
    /// does nothing if `new_cap` <= `self.capacity()`
    pub fn grow_to(&mut self, new_cap: usize) {
        assert!(self.cap < new_cap);

        let (old_cap, old_buf) = (self.cap, self.buf);
        let layout = Layout::array::<T>(new_cap).expect("layout");
        let new_buf = self
            .alloc
            .allocate(layout)
            .map(|ptr| ptr.as_ptr() as *mut T)
            .map_err(|_| handle_alloc_error(layout))
            .unwrap();

        if !self.buf.is_null() {
            for i in 0..self.top {
                unsafe { new_buf.add(i).write(old_buf.add(i).read()) }
            }
        }

        match NonNull::new(old_buf as *mut u8) {
            Some(ptr) => {
                assert_ne!(old_cap, 0);
                let old_layout = Layout::array::<T>(old_cap).expect("layout");
                unsafe { self.alloc.deallocate(ptr, old_layout) }
            }
            None => assert_eq!(old_cap, 0),
        }

        self.cap = new_cap;
        self.buf = new_buf;
    }
}

#[cfg(test)]
mod test {
    use crate::stack::Stack;
    use rand::random;
    use std::{
        assert_matches,
        cell::{Cell, RefCell},
    };

    const TIMES: usize = 100_000;

    #[test]
    fn fuzz_test_stack_primitives() {
        let mut stack = Stack::new();
        let mut fuzz = Vec::with_capacity(TIMES);

        // pushing random value into the stack
        (0..TIMES).for_each(|_| {
            let r = random::<i32>();
            fuzz.push(r);
            stack.push(r);
        });

        // test clone
        let mut clone = stack.clone();

        // cloned and stack must in different location
        assert_ne!(stack.buf, clone.buf);

        // clone[i] should equal to stack[idx]
        (0..TIMES).for_each(|idx| assert_eq!(clone[idx], stack[idx]));

        // stack[i] should equal to fuzz[idx]
        (0..TIMES).for_each(|idx| assert_eq!(fuzz[idx], stack[idx]));

        // notice the `rev`, because stack will reverse the input sequence
        (0..TIMES).rev().for_each(|idx| {
            assert_matches!(stack.pop(), Some(x) if fuzz[idx] == x);
            assert_matches!(clone.pop(), Some(x) if fuzz[idx] == x);
        });

        assert_matches!(stack.pop(), None);
        assert_matches!(clone.pop(), None);
    }

    #[test]
    fn test_stack_drop() {
        thread_local! {
            /// DROPPED[i] == false means the flower with id `i` has not been dropped
            static DROPPED: RefCell<Vec<bool>> = RefCell::new((0..TIMES).map(|_| false).collect());

            /// DROPPED_SEQ[i] == id means the item with id `id` is the `i + 1`th item dropped
            static DROPPED_SEQ: RefCell<Vec<usize>> = RefCell::new(Vec::with_capacity(TIMES));

            /// DROPPED_CLONE[i] == false means the flower with id `i` has not been dropped
            static DROPPED_CLONE: RefCell<Vec<bool>> = RefCell::new((0..TIMES).map(|_| false).collect());

            /// DROPPED_CLONE_SEQ[i] == id means the item with id `id` is the `i + 1`th item dropped
            static DROPPED_CLONE_SEQ: RefCell<Vec<usize>> = RefCell::new(Vec::with_capacity(TIMES));

            /// next item id
            static NEXT_ID: Cell<usize> = const { Cell::new(0) };
        }

        struct Item {
            id: usize,
            cloned: bool,
        }

        {
            // stack's lifetime begins here
            let mut stack = Stack::new();

            (0..TIMES).for_each(|_| {
                stack.push(Item::new());
            });

            let _cloned = stack.clone();
        }
        // ends before this back curly brace

        assert!(DROPPED_CLONE.with_borrow(|vec| vec.iter().all(|dropped| *dropped)));
        assert!(DROPPED.with_borrow(|vec| vec.iter().all(|dropped| *dropped)));
        assert!(DROPPED_CLONE_SEQ.with_borrow(|vec| {
            vec.iter()
                .enumerate()
                .all(|(index, id)| index + id == TIMES - 1)
        }));
        assert!(DROPPED_SEQ.with_borrow(|vec| {
            vec.iter()
                .enumerate()
                .all(|(index, id)| index + id == TIMES - 1)
        }));

        // impl Item
        const _: () = {
            impl Item {
                fn new() -> Self {
                    let id = NEXT_ID.get();
                    NEXT_ID.set(id + 1);
                    Self { id, cloned: false }
                }
            }

            impl Drop for Item {
                fn drop(&mut self) {
                    if self.cloned {
                        DROPPED_CLONE_SEQ.with_borrow_mut(|vec| vec.push(self.id));
                        DROPPED_CLONE.with_borrow_mut(|vec| vec[self.id] = true)
                    } else {
                        DROPPED_SEQ.with_borrow_mut(|vec| vec.push(self.id));
                        DROPPED.with_borrow_mut(|vec| vec[self.id] = true)
                    }
                }
            }

            impl Clone for Item {
                fn clone(&self) -> Self {
                    Self {
                        id: self.id,
                        cloned: true,
                    }
                }
            }
        };
    }
}
