#![feature(allocator_api)]

use std::alloc::Layout;
pub mod queue;
pub mod stack;

pub trait CapacityIncrement {
    fn appropriate_size(&mut self, elem: Layout, curr: usize) -> usize;
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DefaultIncrement;

impl CapacityIncrement for DefaultIncrement {
    fn appropriate_size(&mut self, elem: Layout, curr: usize) -> usize {
        match curr {
            0 => match elem.pad_to_align().size() {
                0 | 1 => 8,
                x if x <= 1024 => 4,
                _ => 1,
            },
            x if x <= 1024 => 2 * x,
            x => x + x / 2,
        }
    }
}

impl<T> CapacityIncrement for T
where
    T: FnMut(Layout, usize) -> usize,
{
    fn appropriate_size(&mut self, elem: Layout, curr: usize) -> usize {
        self(elem, curr)
    }
}
