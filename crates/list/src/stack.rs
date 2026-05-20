use std::{
    alloc::{Allocator, Global, Layout, handle_alloc_error},
    ops::{Index, IndexMut},
    ptr::{NonNull, drop_in_place},
};

use crate::{CapacityIncrement, DefaultIncrement};

pub struct Stack<T, A = Global, C = DefaultIncrement>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
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

pub struct StackIter<'a, T, A, C>(&'a mut Stack<T, A, C>)
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement;

impl<T: Sized> Stack<T, Global, DefaultIncrement> {
    #[inline]
    pub fn new() -> Self {
        Self::new_in(Global, DefaultIncrement)
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global, DefaultIncrement)
    }
}

impl<'a, T, A, C> Iterator for StackIter<'a, T, A, C>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T, A, C> IntoIterator for &'a mut Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    type Item = T;
    type IntoIter = StackIter<'a, T, A, C>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StackIter(self)
    }
}

impl<T: Sized, A: Allocator, C: CapacityIncrement> Drop for Stack<T, A, C> {
    fn drop(&mut self) {
        // first we call destructor on each slot
        for i in 0..self.top {
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
    C: CapacityIncrement + Clone,
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
    C: CapacityIncrement,
{
    /// # Get mutable reference or [copy](Copy) of `index + 1`th element of the stack
    ///
    /// ```rust,should_panic
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    ///
    /// let _ = stack[1];   // panics!
    /// ```
    ///
    /// ```rust,compile_fail
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(String::from("👋")); // string `👋` locates at index `0` of this stack
    ///
    /// let _move: String = stack[0];   // compilation error
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.top {
            panic!("indexed beyond [0, top)")
        } else {
            unsafe { &mut *self.buf.add(index) }
        }
    }
}

impl<T, A, C> Index<usize> for Stack<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    type Output = T;

    /// # Get immutable reference or [copy](Copy) of `index + 1`th element of the stack
    ///
    /// ```rust,should_panic
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push('👋');   // char `👋` locates at index `0` of this stack
    ///
    /// let _ = stack[1];   // panics!
    /// ```
    ///
    /// ```rust,compile_fail
    /// # use list::stack::Stack;
    /// let mut stack = Stack::new();
    /// stack.push(String::from("👋")); // string `👋` locates at index `0` of this stack
    ///
    /// let _move: String = stack[0];   // compilation error
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.top {
            panic!("indexed beyond [0, top)")
        } else {
            unsafe { &*self.buf.add(index) }
        }
    }
}

impl<T: Sized, A: Allocator, C: CapacityIncrement> Stack<T, A, C> {
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
            cap: cap,
            top: 0,
            alloc,
            incre,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if self.top == self.cap {
            let new_cap = self.incre.appropriate_size(Layout::new::<T>(), self.cap);
            self.grow_to(new_cap);
        }

        let pos = self.top;
        self.top += 1;

        unsafe { self.buf.add(pos).write(value) };
    }

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

        let old_buf = self.buf;
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
            /// DROPED[i] == false means the flower with id `i` has not been dropped
            static DROPED: RefCell<Vec<bool>> = RefCell::new((0..TIMES).map(|_| false).collect());

            /// DROPED_CLONE[i] == false means the flower with id `i` has not been dropped
            static DROPED_CLONE: RefCell<Vec<bool>> = RefCell::new((0..TIMES).map(|_| false).collect());

            /// next item id
            static NEXT_ID: Cell<usize> = Cell::new(0);
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

        assert!(DROPED_CLONE.with_borrow(|vec| vec.iter().all(|droped| *droped)));
        assert!(DROPED.with_borrow(|vec| vec.iter().all(|droped| *droped)));

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
                        DROPED_CLONE.with_borrow_mut(|vec| vec[self.id] = true)
                    } else {
                        DROPED.with_borrow_mut(|vec| vec[self.id] = true)
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
