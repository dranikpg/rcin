extern crate rcin;
use rcin::cin;

#[allow(unused_must_use)]
fn main() {
    let mut x: i32 = 0;
    print!("Please enter 1: ");
    cin >> &mut x;
    if x == 1 {
        println!("Thanks!");
    } else {
        panic!("Input wasn't 1");
    }
}
