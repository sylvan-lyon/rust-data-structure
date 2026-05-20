use queue::Queue;

use crate::{Traverse, TreeNode};

pub struct PreOrder;
pub struct InOrder;
pub struct PostOrder;
pub struct LevelOrder;

impl Traverse for PreOrder {
    fn traverse<T>(tree: Option<&TreeNode<T>>, f: &mut impl FnMut(&T)) {
        if let Some(node) = tree {
            f(&node.value);
            Self::traverse(node.left.as_ref().map(Box::as_ref), f);
            Self::traverse(node.right.as_ref().map(Box::as_ref), f);
        }
    }

    fn traverse_mut<T>(tree: Option<&mut TreeNode<T>>, f: &mut impl FnMut(&mut T)) {
        if let Some(node) = tree {
            f(&mut node.value);
            Self::traverse_mut(node.left.as_mut().map(Box::as_mut), f);
            Self::traverse_mut(node.right.as_mut().map(Box::as_mut), f);
        }
    }
}

impl Traverse for InOrder {
    fn traverse<T>(tree: Option<&TreeNode<T>>, f: &mut impl FnMut(&T)) {
        if let Some(node) = tree {
            Self::traverse(node.left.as_ref().map(Box::as_ref), f);
            f(&node.value);
            Self::traverse(node.right.as_ref().map(Box::as_ref), f);
        }
    }

    fn traverse_mut<T>(tree: Option<&mut TreeNode<T>>, f: &mut impl FnMut(&mut T)) {
        if let Some(node) = tree {
            Self::traverse_mut(node.left.as_mut().map(Box::as_mut), f);
            f(&mut node.value);
            Self::traverse_mut(node.right.as_mut().map(Box::as_mut), f);
        }
    }
}

impl Traverse for PostOrder {
    fn traverse<T>(tree: Option<&TreeNode<T>>, f: &mut impl FnMut(&T)) {
        if let Some(node) = tree {
            Self::traverse(node.left.as_ref().map(Box::as_ref), f);
            Self::traverse(node.right.as_ref().map(Box::as_ref), f);
            f(&node.value);
        }
    }

    fn traverse_mut<T>(tree: Option<&mut TreeNode<T>>, f: &mut impl FnMut(&mut T)) {
        if let Some(node) = tree {
            Self::traverse_mut(node.left.as_mut().map(Box::as_mut), f);
            Self::traverse_mut(node.right.as_mut().map(Box::as_mut), f);
            f(&mut node.value);
        }
    }
}

impl Traverse for LevelOrder {
    fn traverse<T>(tree: Option<&TreeNode<T>>, f: &mut impl FnMut(&T)) {
        if let Some(node) = tree {
            let mut todo = Queue::new();
            todo.push(node);

            while let Some(node) = todo.pop() {
                f(&node.value);
                node.left.as_ref().map(|left| todo.push(left));
                node.right.as_ref().map(|right| todo.push(right));
            }
        }
    }

    fn traverse_mut<T>(tree: Option<&mut TreeNode<T>>, f: &mut impl FnMut(&mut T)) {
        if let Some(node) = tree {
            let mut todo = Queue::new();
            todo.push(node);

            while let Some(node) = todo.pop() {
                f(&mut node.value);
                node.left.as_mut().map(|left| todo.push(left));
                node.right.as_mut().map(|right| todo.push(right));
            }
        }
    }
}
