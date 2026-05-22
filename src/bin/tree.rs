use tree::TreeNode;

fn main() {
    println!("{}", std::mem::size_of::<TreeNode<()>>())
}
