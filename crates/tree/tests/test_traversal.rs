use tree::{
    Tree,
    traverse::{InOrder, LevelOrder, PostOrder, PreOrder},
};

fn build_tree_1() -> Tree<i32> {
    Tree::from_slice(&[Some(1), Some(2), Some(3)])
}

/// ```
///         1
///      /     \
///     2       3
///    / \     /  \
///   4  5    6    7
///  / \     /      \
/// 8  9    10      11
/// ```
fn build_tree_2() -> Tree<i32> {
    Tree::from_slice(&[
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        None,
        None,
        Some(10),
        None,
        None,
        Some(11),
    ])
}

#[test]
fn test_preorder_traversal_1() {
    let mut seq = vec![];
    build_tree_1().traverse::<PreOrder>(|value| seq.push(*value));
    assert_eq!(seq, vec![1, 2, 3]);
}

#[test]
fn test_preorder_traversal_2() {
    let mut seq = vec![];
    build_tree_2().traverse::<PreOrder>(|value| seq.push(*value));
    assert_eq!(seq, [1, 2, 4, 8, 9, 5, 3, 6, 10, 7, 11]);
}

#[test]
fn test_inorder_traversal_1() {
    let mut seq = vec![];
    build_tree_1().traverse::<InOrder>(|value| seq.push(*value));
    assert_eq!(seq, vec![2, 1, 3]);
}

#[test]
fn test_inorder_traversal_2() {
    let mut seq = vec![];
    build_tree_2().traverse::<InOrder>(|value| seq.push(*value));
    assert_eq!(seq, [8, 4, 9, 2, 5, 1, 10, 6, 3, 7, 11]);
}

#[test]
fn test_postorder_traversal_1() {
    let mut seq = vec![];
    build_tree_1().traverse::<PostOrder>(|value| seq.push(*value));
    assert_eq!(seq, vec![2, 3, 1]);
}

#[test]
fn test_postorder_traversal_2() {
    let mut seq = vec![];
    build_tree_2().traverse::<PostOrder>(|value| seq.push(*value));
    assert_eq!(seq, [8, 9, 4, 5, 2, 10, 6, 11, 7, 3, 1]);
}

#[test]
fn test_levelorder_traversal_1() {
    let mut seq = vec![];
    build_tree_1().traverse::<LevelOrder>(|value| seq.push(*value));
    assert_eq!(seq, vec![1, 2, 3]);
}

#[test]
fn test_levelorder_traversal_mut_1() {
    let mut seq = vec![];
    let mut tree = build_tree_1();
    tree.traverse_mut::<LevelOrder>(|value| *value *= 2);
    tree.traverse::<LevelOrder>(|value| seq.push(*value));
    assert_eq!(seq, [2, 4, 6]);
}

#[test]
fn test_levelorder_traversal_2() {
    let mut seq = vec![];
    build_tree_2().traverse::<LevelOrder>(|value| seq.push(*value));
    assert_eq!(seq, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

#[test]
fn test_levelorder_traversal_mut_2() {
    let mut seq = vec![];
    let mut tree = build_tree_2();
    tree.traverse_mut::<LevelOrder>(|value| *value *= 2);
    tree.traverse::<LevelOrder>(|value| seq.push(*value));
    assert_eq!(seq, [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22]);
}
