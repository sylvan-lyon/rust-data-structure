#![feature(allocator_api)]
use std::{
    alloc::{Global, Layout},
    assert_matches,
    cell::{Cell, RefCell},
};

use list::queue::Queue;
use rand::random;

const fn always_twice(_: Layout, curr: usize) -> usize {
    if curr == 0 { 1 } else { curr * 2 }
}

const TIMES: usize = 5_000;
const FIRST_HALF: usize = TIMES / 2;
const LAST_HALF: usize = TIMES - FIRST_HALF;

#[test]
fn fuzz_test_queue_primitives() {
    let mut queue = Queue::new_in(Global, always_twice);
    let mut fuzz = Vec::with_capacity(TIMES);

    // pushing random value into the queue
    (0..TIMES).for_each(|_| {
        let r = random::<i32>();
        fuzz.push(r);
        queue.push(r);
    });

    // test clone
    let mut clone = queue.clone();

    assert_eq!(queue.len(), TIMES);
    assert_eq!(clone.len(), TIMES);

    // clone[i] should equal to queue[idx]
    (0..TIMES).for_each(|idx| assert_eq!(clone[idx], queue[idx]));

    // stack[i] should equal to fuzz[idx]
    (0..TIMES).for_each(|idx| assert_eq!(fuzz[idx], queue[idx]));

    (0..FIRST_HALF).for_each(|idx| {
        assert_eq!(queue.peek(), clone.peek());
        assert_matches!(queue.pop(), Some(x) if fuzz[idx] == x);
        assert_matches!(clone.pop(), Some(x) if fuzz[idx] == x);
    });
    assert_eq!(queue.len(), LAST_HALF);
    assert_eq!(clone.len(), LAST_HALF);

    // now again, this enforces queue to wrap up
    (0..FIRST_HALF).for_each(|_| {
        let r = random::<i32>();
        fuzz.push(r);
        queue.push(r);
    });
    let mut clone = queue.clone();
    assert_eq!(queue.len(), TIMES);
    assert_eq!(clone.len(), TIMES);
    assert!(!queue.is_empty());
    assert!(!clone.is_empty());

    // queue[i] should equal to fuzz[idx]
    (0..TIMES).for_each(|idx| assert_eq!(fuzz[idx + FIRST_HALF], queue[idx]));

    (0..TIMES).for_each(|idx| {
        assert_eq!(queue.peek(), clone.peek());
        assert_matches!(queue.pop(), Some(x) if fuzz[idx + FIRST_HALF] == x);
        assert_matches!(clone.pop(), Some(x) if fuzz[idx + FIRST_HALF] == x);
    });

    assert_matches!(queue.pop(), None);
    assert_matches!(clone.pop(), None);
}

#[test]
fn test_queue_drop() {
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
        // queue's lifetime begins here
        let mut queue = Queue::new();

        (0..TIMES).for_each(|_| {
            queue.push(Item::new());
        });

        let _cloned = queue.clone();
    }
    // ends before this back curly brace

    assert!(DROPPED_CLONE.with_borrow(|vec| vec.iter().all(|dropped| *dropped)));
    assert!(DROPPED.with_borrow(|vec| vec.iter().all(|dropped| *dropped)));
    assert!(
        DROPPED_CLONE_SEQ
            .with_borrow(|vec| { vec.iter().enumerate().all(|(index, id)| index == *id) })
    );
    assert!(
        DROPPED_SEQ.with_borrow(|vec| { vec.iter().enumerate().all(|(index, id)| index == *id) })
    );

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
