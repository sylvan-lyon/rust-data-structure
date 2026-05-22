use list::stack::Stack;

fn main() {
    let mut stack = Stack::new();
    stack.push(1);
    stack.push(2);

    println!("{:?}", stack)
}
