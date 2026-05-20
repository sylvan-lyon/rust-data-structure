use std::assert_matches;
use tree::Tree;

#[test]
fn build_empty_tree_from_slice() {
    // length == 0 means no node
    let s = Vec::<Option<i32>>::new();
    let root = Tree::from_slice(&s).unwrap();

    assert_matches!(&root, None, "root is not none");

    // first node is none also means no node
    let s = vec![Option::<i32>::None];
    let root = Tree::from_slice(&s).unwrap();

    assert_matches!(&root, None, "root is not none");
}

#[test]
fn build_full_tree_1_layer_from_slice() {
    let s = [Some(1)];

    let root = Tree::from_slice(&s).unwrap();

    // nodex means this node should have value of x
    assert_matches!(&root, Some(node1) if node1.value == 1, "root value is not 1");
    let node1 = root.unwrap();

    assert_matches!(node1.left, None, "root has left child");
    assert_matches!(node1.right, None, "root has right child");
}

#[test]
fn build_full_tree_2_layer_from_slice() {
    let s = [Some(1), Some(2), Some(3)];

    let root = Tree::from_slice(&s).unwrap();

    // nodex means this node should have value of x
    assert_matches!(&root, Some(node1) if node1.value == 1, "root value is not 1");
    let node1 = root.unwrap();

    assert_matches!(&node1.left, Some(node2) if node2.value == 2, "left child of root is not 2");
    assert_matches!(&node1.right, Some(node3) if node3.value == 3, "right child of root is not 3");

    let node2 = node1.left.unwrap();
    let node3 = node1.right.unwrap();

    assert_matches!(node2.left, None, "node2 has left child");
    assert_matches!(node2.right, None, "node2 has right child");

    assert_matches!(node3.left, None, "node3 is not none");
    assert_matches!(node3.right, None, "node2 has right child");
}

#[test]
fn build_full_tree_3_layer_from_slice() {
    let mut s = Vec::new();
    for i in 1..=7 {
        s.push(Some(i));
    }

    let root = Tree::from_slice(&s).unwrap();

    // nodex means this node should have value of x
    assert_matches!(&root, Some(node1) if node1.value == 1, "root value is not 1");
    let node1 = root.unwrap();

    assert_matches!(&node1.left, Some(node2) if node2.value == 2, "left child of root is not 2");
    assert_matches!(&node1.right, Some(node3) if node3.value == 3, "right child of root is not 3");

    let node2 = node1.left.unwrap();
    let node3 = node1.right.unwrap();

    assert_matches!(&node2.left, Some(node4) if node4.value == 4, "left child of node2 is not 4");
    assert_matches!(&node2.right, Some(node5) if node5.value == 5, "right child of node2 is not 5");

    assert_matches!(&node3.left, Some(node6) if node6.value == 6, "left child of node3 is not 6");
    assert_matches!(&node3.right, Some(node7) if node7.value == 7, "right child of node3 is not 7");

    let node4 = node2.left.unwrap();
    let node5 = node2.right.unwrap();
    let node6 = node3.left.unwrap();
    let node7 = node3.right.unwrap();

    assert_matches!(&node4.left, None, "node4 has left child");
    assert_matches!(&node4.right, None, "node4 has right child");

    assert_matches!(&node5.left, None, "node5 has left child");
    assert_matches!(&node5.right, None, "node5 has right child");

    assert_matches!(&node6.left, None, "node6 has left child");
    assert_matches!(&node6.right, None, "node6 has right child");

    assert_matches!(&node7.left, None, "node7 has left child");
    assert_matches!(&node7.right, None, "node7 has right child");
}

#[test]
fn build_complete_tree_6_nodes_from_slice() {
    let mut s = Vec::new();
    for i in 1..=6 {
        s.push(Some(i));
    }

    let root = Tree::from_slice(&s).unwrap();

    // nodex means this node should have value of x
    assert_matches!(&root, Some(node1) if node1.value == 1, "root value is not 1");
    let node1 = root.unwrap();

    assert_matches!(&node1.left, Some(node2) if node2.value == 2, "left child of root is not 2");
    assert_matches!(&node1.right, Some(node3) if node3.value == 3, "right child of root is not 3");

    let node2 = node1.left.unwrap();
    let node3 = node1.right.unwrap();

    assert_matches!(&node2.left, Some(node4) if node4.value == 4, "left child of node2 is not 4");
    assert_matches!(&node2.right, Some(node5) if node5.value == 5, "right child of node2 is not 5");

    assert_matches!(&node3.left, Some(node6) if node6.value == 6, "left child of node3 is not 6");
    assert_matches!(&node3.right, None, "node3 has right child");

    let node4 = node2.left.unwrap();
    let node5 = node2.right.unwrap();

    assert_matches!(&node4.left, None, "node4 has left child");
    assert_matches!(&node4.right, None, "node4 has right child");

    assert_matches!(&node5.left, None, "node5 has left child");
    assert_matches!(&node5.right, None, "node5 has right child");
}

#[test]
fn build_random_tree_1() {
    // the tree looks like this:
    //       1
    //      / \
    //     2  none
    //    / \
    // none  3
    let s = [Some(1), Some(2), None, None, Some(3)];

    let root = Tree::from_slice(&s).unwrap();

    // nodex means this node should have value of x
    assert_matches!(&root, Some(node1) if node1.value == 1, "root value is not 1");
    let node1 = root.unwrap();

    assert_matches!(&node1.left, Some(node2) if node2.value == 2, "left child of root is not 2");
    assert_matches!(node1.right, None, "root has right child");

    let node2 = node1.left.unwrap();

    assert_matches!(node2.left, None, "node2 has left child");
    assert_matches!(&node2.right, Some(node3) if node3.value == 3, "right child of node2 is not 3");

    let node3 = node2.right.unwrap();

    assert_matches!(&node3.left, None, "node3 has left child");
    assert_matches!(&node3.right, None, "node3 has right child");
}

#[test]
fn build_tree_with_strange_input() {
    // the input looks like this, so in fact, the output tree is a tree with single node
    //        1
    //      /   \
    //   none   none   ==>     1
    //  /  \   /   \
    //  3  4   5   6
    let s = [Some(1), None, None, Some(2), Some(3), Some(4), Some(5)];

    let root = Tree::from_slice(&s).unwrap();

    assert_matches!(&root, Some(root) if root.value == 1, "root value is not 1");

    let node1 = root.unwrap();
    assert_matches!(&node1.left, None, "root has left child");
    assert_matches!(&node1.right, None, "root has right child");
}
