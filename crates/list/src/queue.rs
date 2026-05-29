use core::slice;
use std::{
    alloc::{Allocator, Global, Layout, handle_alloc_error},
    fmt::Debug,
    mem,
    ops::{Index, IndexMut},
    ptr::{self, NonNull},
};

use crate::{Increment, increment};

pub struct Queue<T, A = Global, C = Increment>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
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

pub struct IntoIter<T, A, C>(Queue<T, A, C>)
where
    A: Allocator,
    C: FnMut(Layout, usize) -> usize;

impl<T, A, C> Iterator for IntoIter<T, A, C>
where
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len, Some(self.0.len))
    }
}

impl<T, A, C> IntoIterator for Queue<T, A, C>
where
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Item = T;
    type IntoIter = IntoIter<T, A, C>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

pub struct Iter<'a, T> {
    seg1: slice::Iter<'a, T>,
    seg2: slice::Iter<'a, T>,
}

pub struct IterMut<'a, T> {
    seg1: slice::IterMut<'a, T>,
    seg2: slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seg1.next() {
            Some(r) => Some(r),
            None => {
                mem::swap(&mut self.seg1, &mut self.seg2);
                self.seg1.next()
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (seg1_min, seg1_max) = self.seg1.size_hint();
        let (seg2_min, seg2_max) = self.seg2.size_hint();

        (
            seg1_min + seg2_min,
            seg1_max.zip(seg2_max).map(|(a, b)| a + b),
        )
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seg1.next() {
            Some(r) => Some(r),
            None => {
                mem::swap(&mut self.seg1, &mut self.seg2);
                self.seg1.next()
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (seg1_min, seg1_max) = self.seg1.size_hint();
        let (seg2_min, seg2_max) = self.seg2.size_hint();

        (
            seg1_min + seg2_min,
            seg1_max.zip(seg2_max).map(|(a, b)| a + b),
        )
    }
}

impl<T: Sized> Default for Queue<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A, C> Debug for Queue<T, A, C>
where
    T: Sized + Debug,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, A, C> Drop for Queue<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    fn drop(&mut self) {
        for i in 0..self.len {
            let pos = wrap(self.head + i, self.cap);
            unsafe { self.buf.add(pos).drop_in_place() }
        }

        let layout = Layout::array::<T>(self.cap).expect("layout");

        match NonNull::new(self.buf as *mut u8) {
            Some(ptr) => unsafe { self.alloc.deallocate(ptr, layout) },
            None => debug_assert_eq!(self.cap, 0),
        }
    }
}

#[inline]
const fn wrap(val: usize, max: usize) -> usize {
    if val < max {
        val
    } else {
        val.saturating_sub(max)
    }
}

impl<T, A1, C1, A2, C2> PartialEq<Queue<T, A2, C2>> for Queue<T, A1, C1>
where
    T: PartialEq,
    A1: Allocator,
    A2: Allocator,
    C1: FnMut(Layout, usize) -> usize,
    C2: FnMut(Layout, usize) -> usize,
{
    fn eq(&self, other: &Queue<T, A2, C2>) -> bool {
        if self.len != other.len {
            return false;
        }

        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl<T: Eq, A: Allocator, C: FnMut(Layout, usize) -> usize> Eq for Queue<T, A, C> {}

impl<T, A, C> Clone for Queue<T, A, C>
where
    T: Clone,
    A: Clone + Allocator,
    C: Clone + FnMut(Layout, usize) -> usize,
{
    fn clone(&self) -> Self {
        let alloc = self.alloc.clone();
        let layout = Layout::array::<T>(self.cap).expect("layout");
        let buf = match NonNull::new(self.buf as *mut u8) {
            Some(_) => alloc
                .allocate(layout)
                .map(|ptr| ptr.as_ptr() as *mut T)
                .map_err(|_| handle_alloc_error(layout))
                .unwrap(),
            None => ptr::null_mut(),
        };

        for i in 0..self.len {
            let pos = wrap(self.head + i, self.cap);
            let cloned = unsafe { &*self.buf.add(pos) }.clone();
            unsafe { buf.add(i).write(cloned) }
        }

        Self {
            buf,
            head: 0,
            tail: wrap(self.len, self.cap),
            len: self.len,
            cap: self.cap,
            alloc,
            incre: self.incre.clone(),
        }
    }
}

impl<T, A, C> Index<usize> for Queue<T, A, C>
where
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        let len = self.len;
        self.get(idx)
            .unwrap_or_else(|| panic!("{idx} beyond [0, {len})"))
    }
}

impl<T, A, C> IndexMut<usize> for Queue<T, A, C>
where
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        let len = self.len;
        self.get_mut(idx)
            .unwrap_or_else(|| panic!("{idx} beyond [0, {len})"))
    }
}

impl<T> Queue<T, Global> {
    #[inline]
    pub const fn new() -> Self {
        Self::new_in(Global, increment)
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global, increment)
    }
}

impl<T, A, C> Queue<T, A, C>
where
    T: Sized,
    A: Allocator,
    C: FnMut(Layout, usize) -> usize,
{
    #[inline]
    pub const fn new_in(alloc: A, incre: C) -> Self {
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

    pub fn with_capacity_in(cap: usize, alloc: A, incre: C) -> Self {
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

    pub fn from_iter_in(iter: impl Iterator<Item = T>, alloc: A, incre: C) -> Self {
        let cap = match iter.size_hint() {
            (_, Some(max)) => max,
            (min, None) => min,
        };

        let mut queue = Self::with_capacity_in(cap, alloc, incre);
        iter.for_each(|elem| queue.push(elem));
        queue
    }

    pub fn from_vec_in(vec: Vec<T>, alloc: A, incre: C) -> Self {
        let (buf, len, cap) = vec.into_raw_parts();

        Self {
            buf,
            len,
            cap,
            head: 0,
            tail: len,
            alloc,
            incre,
        }
    }

    /// the capacity of whole ring buffer
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == self.cap
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn get(&self, idx: usize) -> Option<&T> {
        if self.is_empty() || !idx < self.len {
            return None;
        }

        let pos = wrap(self.head + idx, self.cap);
        Some(unsafe { &*self.buf.add(pos) })
    }

    #[inline]
    pub const fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if self.is_empty() || !idx < self.len {
            return None;
        }

        let pos = wrap(self.head + idx, self.cap);
        Some(unsafe { &mut *self.buf.add(pos) })
    }

    #[inline]
    pub const fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { &*self.buf.add(self.head) })
    }

    #[inline]
    pub const fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }

        Some(unsafe { &mut *self.buf.add(self.head) })
    }

    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let old = self.head;
        self.head += 1;
        self.head = wrap(self.head, self.cap);
        self.len -= 1;

        Some(unsafe { self.buf.add(old).read() })
    }

    pub fn push(&mut self, value: T) {
        if self.is_full() {
            let new_cap = (self.incre)(Layout::new::<T>(), self.cap);
            self.grow_to(new_cap);
        }

        let old = self.tail;
        self.tail += 1;
        self.tail = wrap(self.tail, self.cap);
        self.len += 1;

        unsafe { self.buf.add(old).write(value) }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        let buf = self.buf;
        use core::slice::from_raw_parts;
        let (seg1, seg2) = if self.is_contiguous() {
            (
                unsafe { from_raw_parts(buf.add(self.head), self.len) }.iter(),
                [].iter(),
            )
        } else {
            (
                unsafe { from_raw_parts(buf.add(self.head), self.cap - self.head) }.iter(),
                unsafe { from_raw_parts(buf, self.tail) }.iter(),
            )
        };

        Iter { seg1, seg2 }
    }

    pub fn iter_mut<'a>(&'a self) -> IterMut<'a, T> {
        let buf = self.buf;
        use core::slice::from_raw_parts_mut;
        let (seg1, seg2) = if self.is_contiguous() {
            (
                unsafe { from_raw_parts_mut(buf.add(self.head), self.len) }.iter_mut(),
                [].iter_mut(),
            )
        } else {
            (
                unsafe { from_raw_parts_mut(buf.add(self.head), self.cap - self.head) }.iter_mut(),
                unsafe { from_raw_parts_mut(buf, self.tail) }.iter_mut(),
            )
        };

        IterMut { seg1, seg2 }
    }

    pub fn grow_to(&mut self, new_cap: usize) {
        if self.cap >= new_cap {
            return;
        }

        let (old_cap, old_buf) = (self.cap, self.buf as *const T);
        let new_layout = Layout::array::<T>(new_cap).expect("layout");
        let new_buf = self
            .alloc
            .allocate(new_layout)
            .map(|ptr| ptr.as_ptr() as *mut T)
            .map_err(|_| handle_alloc_error(new_layout))
            .unwrap();

        match NonNull::new(old_buf as *mut u8) {
            Some(ptr) => {
                // in case self is grown from 0
                assert_ne!(old_cap, 0);
                for i in 0..self.len {
                    let old_pos = wrap(self.head + i, old_cap);
                    let new_pos = i;
                    let value = unsafe { old_buf.add(old_pos).read() };
                    unsafe { new_buf.add(new_pos).write(value) }
                }

                let old_layout = Layout::array::<T>(old_cap).expect("layout");
                unsafe { self.alloc.deallocate(ptr, old_layout) }
            }
            None => assert_eq!(old_cap, 0),
        }

        self.head = 0;
        self.tail = self.len;
        self.cap = new_cap;
        self.buf = new_buf;
    }

    #[inline]
    pub fn is_contiguous(&self) -> bool {
        let end = if self.tail == 0 {
            self.cap
        } else {
            self.tail - 1
        };
        self.head <= end
    }

    pub fn make_contiguous(&mut self) {
        if self.is_contiguous() {
            return;
        }

        for i in 0..self.len {
            let pos = wrap(self.head + i, self.cap);
            let old_pos = unsafe { &mut *self.buf.add(pos) };
            let new_pos = unsafe { &mut *self.buf.add(i) };
            std::mem::swap(new_pos, old_pos);
        }

        self.head = 0;
        self.tail = self.len;
    }
}

#[cfg(test)]
mod test {
    use crate::queue::Queue;
    use std::alloc::{Global, Layout};

    const fn always_twice(_: Layout, curr: usize) -> usize {
        if curr == 0 { 1 } else { curr * 2 }
    }

    #[test]
    fn test_contiguous() {
        let mut queue = Queue::new_in(Global, always_twice);

        queue.push('A'); // A
        queue.push('B'); // AB
        queue.push('C'); // ABC.

        queue.pop(); // .BC.
        assert_eq!(queue.head, 1);
        assert_eq!(queue.tail, 3);

        queue.push('D'); // .BCD
        assert!(queue.is_contiguous());
        queue.push('E'); // ECBD
        assert!(!queue.is_contiguous());
        assert_eq!(queue.head, 1);
        assert_eq!(queue.tail, 1);
        assert_eq!(queue.len, 4);
        assert_eq!(queue.cap, 4);
    }

    #[test]
    fn test_queue_clone_contiguous() {
        let mut queue = Queue::new_in(Global, always_twice);
        assert!(queue.is_contiguous());

        queue.push('A'); // A
        queue.push('B'); // AB
        queue.push('C'); // ABC.

        queue.pop(); // .BC.
        assert_eq!(queue.head, 1);
        assert_eq!(queue.tail, 3);

        let clone = queue.clone();
        assert_eq!(clone.head, 0);
        assert_eq!(clone.tail, 2);
        assert!(clone.is_contiguous());

        queue.push('D'); // .BCD
        assert!(queue.is_contiguous());

        let clone = queue.clone();
        assert_eq!(clone.head, 0);
        assert_eq!(clone.tail, 3);
        assert!(clone.is_contiguous());

        queue.push('E'); // EBCD
        assert!(!queue.is_contiguous());

        let clone = queue.clone();
        assert_eq!(clone.head, 0);
        assert_eq!(clone.tail, 0);
        assert!(clone.is_contiguous());

        assert!(queue.clone().is_contiguous());
        assert_eq!(queue.head, 1);
        assert_eq!(queue.tail, 1);
        assert_eq!(queue.len, 4);
        assert_eq!(queue.cap, 4);
    }

    #[test]
    fn test_queue_iter() {
        let mut queue = Queue::new_in(Global, always_twice);

        queue.push('A'); // A
        queue.push('B'); // AB
        queue.push('C'); // ABC.

        let iter = queue.iter();
        assert_eq!(iter.seg1.len(), 3);
        assert_eq!(iter.seg2.len(), 0);
        assert_eq!(Vec::from_iter(iter.copied()), vec!['A', 'B', 'C']);

        queue.pop(); // .BC.

        let iter = queue.iter();
        assert_eq!(iter.seg1.len(), 2);
        assert_eq!(iter.seg2.len(), 0);
        assert_eq!(Vec::from_iter(iter.copied()), vec!['B', 'C']);

        queue.push('D'); // .BCD
        let iter = queue.iter();
        assert_eq!(iter.seg1.len(), 3);
        assert_eq!(iter.seg2.len(), 0);
        assert_eq!(Vec::from_iter(iter.copied()), vec!['B', 'C', 'D']);

        queue.push('E'); // EBCD
        let iter = queue.iter();
        assert_eq!(iter.seg1.len(), 3);
        assert_eq!(iter.seg2.len(), 1);
        assert_eq!(
            Vec::from_iter(iter.seg1.clone().copied()),
            vec!['B', 'C', 'D']
        );
        assert_eq!(Vec::from_iter(iter.seg2.clone().copied()), vec!['E']);
        assert_eq!(Vec::from_iter(iter.copied()), vec!['B', 'C', 'D', 'E']);

        queue.pop();
        queue.pop(); // E..D
        let iter = queue.iter();
        assert_eq!(iter.seg1.len(), 1);
        assert_eq!(iter.seg2.len(), 1);
        assert_eq!(Vec::from_iter(iter.seg1.clone().copied()), vec!['D']);
        assert_eq!(Vec::from_iter(iter.seg2.clone().copied()), vec!['E']);
        assert_eq!(Vec::from_iter(iter.copied()), vec!['D', 'E']);
    }

    #[test]
    fn test_queue_eq() {
        let mut queue = Queue::new_in(Global, always_twice);
        assert!(queue.is_contiguous());

        queue.push('A'); // A
        queue.push('B'); // AB
        queue.push('C'); // ABC.
        let mut queue2 = Queue::new();
        queue2.push('A');
        queue2.push('B');
        queue2.push('C');
        assert_eq!(queue, queue2);

        queue.pop(); // .BC.
        let mut queue2 = Queue::new();
        queue2.push('B');
        queue2.push('C');
        assert_eq!(queue, queue2);

        queue.push('D'); // .BCD
        let mut queue2 = Queue::new();
        queue2.push('B');
        queue2.push('C');
        queue2.push('D');
        assert_eq!(queue, queue2);

        queue.push('E'); // EBCD
        let mut queue2 = Queue::new();
        queue2.push('B');
        queue2.push('C');
        queue2.push('D');
        queue2.push('E');
        assert_eq!(queue, queue2);

        queue.pop();
        queue.pop(); // E..D
        let mut queue2 = Queue::new();
        queue2.push('D');
        queue2.push('E');
        assert_eq!(queue, queue2);
    }
}
