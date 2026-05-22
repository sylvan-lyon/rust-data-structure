#![feature(allocator_api)]

use std::alloc::Layout;
pub mod queue;
pub mod stack;

pub type Increment = fn(elem: Layout, curr: usize) -> usize;

#[inline]
pub const fn increment(elem: Layout, curr: usize) -> usize {
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

#[cfg(test)]
mod test {
    use std::alloc::Global;

    use crate::stack::Stack;

    #[test]
    fn test() {
        let mut count = 0;

        let _: Stack<(), _, _> = Stack::new_in(Global, |_, _| {
            count += 1;
            0
        });
    }
}
