#![feature(fn_traits, unboxed_closures)]

pub struct Resursive {
    times: i32,
}

impl Resursive {
    fn new() -> Resursive {
        Self { times: 0 }
    }
}

impl FnOnce<()> for Resursive {
    type Output = i32;

    extern "rust-call" fn call_once(mut self, args: ()) -> Self::Output {
        println!("self call once");
        self.call_mut(args)
    }
}

impl FnMut<()> for Resursive {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> Self::Output {
        if self.times == 3 {
            return self.times;
        }
        println!("Hello world!");
        self.times += 1;
        self.call_mut(());
        self.times
    }
}

fn main() {
    println!("before first call");
    let mut closure = Resursive::new();
    println!("printed {} times.", closure());
    println!("before second call");
    closure.call_mut(());
    println!("already printed {} times.", closure());
}
