use std::{
    alloc::{Allocator, Global, Layout, handle_alloc_error},
    ptr::NonNull,
};

use crate::{CapacityIncrement, DefaultIncrement};

pub struct Queue<T, A = Global, C = DefaultIncrement>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    /// buffer, which contains `self.cap` slots of `T`
    buf: *mut T,

    /// next position to pop (a slot contains valid data)
    head: usize,

    /// next position to insert (an uninitialized slot)
    tail: usize,

    /// capacity of buf, valid from `[0, usize::MAX]`
    cap: usize,

    /// number of elements
    len: usize,

    /// allocator
    alloc: A,

    /// increment calculator
    incre: C,
}

impl<T: Sized> Default for Queue<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A, C> Drop for Queue<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    fn drop(&mut self) {
        for i in 0..self.len {
            let pos = (self.head + i) % self.cap;
            unsafe { self.buf.add(pos).drop_in_place() }
        }

        let layout = Layout::array::<T>(self.cap).unwrap();

        match NonNull::new(self.buf as *mut u8) {
            Some(ptr) => unsafe { self.alloc.deallocate(ptr, layout) },
            None => debug_assert_eq!(self.cap, 0),
        }
    }
}

impl<T> Queue<T, Global> {
    #[inline]
    pub fn new() -> Self {
        Self::new_in(Global, DefaultIncrement)
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_exact_in(cap, Global, DefaultIncrement)
    }
}

impl<T, A, C> Queue<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: CapacityIncrement,
{
    #[inline]
    pub fn new_in(alloc: A, incre: C) -> Self {
        Self {
            buf: std::ptr::null_mut(),
            head: 0,
            tail: 0,
            cap: 0,
            len: 0,
            alloc,
            incre,
        }
    }

    #[inline]
    pub fn with_capacity_exact_in(cap: usize, alloc: A, incre: C) -> Self {
        let layout = Layout::array::<T>(cap).expect("layout");
        let buf = alloc.allocate(layout).expect("allocation").as_ptr() as *mut _;

        Self {
            buf,
            head: 0,
            tail: 0,
            cap,
            len: 0,
            alloc,
            incre,
        }
    }

    /// the capacity of whole ring buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(unsafe_code)]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let old = self.head;
        self.head += 1;
        self.head %= self.cap;
        self.len -= 1;

        Some(unsafe { self.buf.add(old).read() })
    }

    pub fn push(&mut self, value: T) {
        if self.is_full() {
            let new_cap = self.incre.appropriate_size(Layout::new::<T>(), self.cap);
            self.grow_to(new_cap);
        }

        let old = self.tail;
        self.tail += 1;
        self.tail %= self.cap;
        self.len += 1;

        unsafe { self.buf.add(old).write(value) };
    }

    pub fn grow_to(&mut self, new_cap: usize) {
        if self.cap >= new_cap {
            return;
        }

        let (old_cap, old_buf) = (self.cap, self.buf);
        let new_layout = Layout::array::<T>(new_cap).expect("layout");
        let new_buf = self
            .alloc
            .allocate(new_layout)
            .map(|ptr| ptr.as_ptr() as *mut T)
            .map_err(|_| handle_alloc_error(new_layout))
            .unwrap();

        if !self.buf.is_null() {
            for i in 0..self.tail {
                unsafe { new_buf.add(i).write(old_buf.add(i).read()) }
            }

            let delta = new_cap - self.cap;
            for i in self.head..self.cap {
                let old_i = i;
                let new_i = delta + i;
                unsafe { new_buf.add(new_i).write(old_buf.add(old_i).read()) }
            }
        }

        match NonNull::new(old_buf as *mut u8) {
            Some(ptr) => {
                // in case self is grown from 0
                assert_ne!(old_cap, 0);
                self.head += new_cap - old_cap;
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
    use crate::queue::Queue;
    use rand::random;
    use std::{
        assert_matches,
        cell::{Cell, RefCell},
    };

    const TIMES: usize = 100_000;

    #[test]
    fn test_queue_primitives() {
        let mut queue = Queue::new();
        let mut fuzz = Vec::with_capacity(TIMES);

        (0..TIMES).for_each(|_| {
            let r = random::<i32>();
            fuzz.push(r);
            queue.push(r);
        });

        (0..TIMES).for_each(|idx| {
            assert_matches!(queue.pop(), Some(x) if fuzz[idx] == x);
        });

        assert_matches!(queue.pop(), None);
    }

    #[test]
    fn test_queue_drop() {
        thread_local! {
            /// DROPED[i] == false means the flower with id `i` has not been dropped
            static DROPED: RefCell<Vec<bool>> = RefCell::new((0..TIMES).map(|_| false).collect());

            /// next item id
            static NEXT_ID: Cell<usize> = Cell::new(0);
        }

        #[derive(Clone)]
        struct Item {
            id: usize,
        }

        impl Item {
            fn rand() -> Self {
                let id = NEXT_ID.get();
                NEXT_ID.set(id + 1);
                Self { id }
            }
        }

        impl Drop for Item {
            fn drop(&mut self) {
                DROPED.with_borrow_mut(|vec| vec[self.id] = true)
            }
        }

        {
            let mut queue = Queue::new();

            (0..TIMES).for_each(|_| {
                queue.push(Item::rand());
            });
        }

        assert!(DROPED.with_borrow(|v| v.iter().all(|presence| *presence)));
    }
}
