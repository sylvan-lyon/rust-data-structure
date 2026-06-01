#![feature(fn_traits, unboxed_closures)]

pub struct Recursive {
    times: i32,
}

impl Recursive {
    fn new() -> Recursive {
        Self { times: 0 }
    }
}

impl FnOnce<()> for Recursive {
    type Output = i32;

    extern "rust-call" fn call_once(mut self, args: ()) -> Self::Output {
        self.call_mut(args)
    }
}

impl FnMut<()> for Recursive {
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
    let mut closure = Recursive::new();
    println!("printed {} times.", closure());
    println!("before second call");
    closure.call_mut(());
    println!("already printed {} times.", closure());
}
