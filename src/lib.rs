//! `cin`-like input from `stdin` for all types that implement `FromStr`.
//!
//! Useful for quick prototyping and debugging without passing any state around.
//!
//! (And for people who complain that input in rust is too verbose.)
//!
//! It stores a buffer of the last line and tries to consume it first.
//! It will block until it finds any sequence of non-whitespace characters.
//!
//! Depends on the [lazy_static](https://docs.rs/lazy_static) crate for storing global state.
//!
//! ## Example
//! ```no_run
//! # extern crate rcin;
//! # fn main() {
//! let x: i32 = rcin::read_next(); // reads until it finds a valid i32
//!
//! print!("Enter three numbers: "); // flushes stdout by default before any input
//! let mut max = std::i32::MIN;
//! for _ in 0..3 {
//!     let t = rcin::read_safe();  // safe = unwrap_or_default
//!     max = std::cmp::max(max, t);
//! }
//! println!("Max: {}", max);
//!
//! print!("Ready to continue?");
//! rcin::pause(); // wait for newline
//! # }
//! ```
//!
//! ## Thread safety
//!
//! `Rcin` is thread safe, but all threads share one buffer.
//! (Parallel input from `stdin` is not a usable thing, is it?)
//!
//! `pause` is __not__ a common lock for all threads.
//!
//! ## Corner case
//!
//! Does __not__ read the input char by char like cin and requires whitespace between groups.
//!
//! Reading an int:
//! ```text
//! C++: 17GARBAGE => 17 // perfectly fine lol
//! RCin: 17GARBAGE => None
//! ```
use lazy_static::lazy_static;
use std::cell::{Ref, RefCell, RefMut};
use std::io::{BufRead, Write};
use std::str::FromStr;
use std::sync::Mutex;
struct RCin {
    data: Vec<char>,
    auto_flush: bool,
}
impl RCin {
    const fn new() -> Self {
        RCin {
            data: Vec::new(),
            auto_flush: true,
        }
    }
    fn update(&mut self) {
        if self.auto_flush {
            std::io::stdout().flush().ok();
        }
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        self.data = buf.trim().chars().rev().collect();
    }
    fn drain(&mut self) {
        while self.data.last().map_or(false, |c: &char| c.is_whitespace()) {
            self.data.pop();
        }
    }
    fn get<T: FromStr>(&mut self) -> Result<T, T::Err> {
        let mut buf = String::new();
        loop {
            if self.data.is_empty() {
                self.update();
            }
            while !self.data.is_empty() {
                let c = self.data.pop().unwrap();
                if c.is_whitespace() {
                    break;
                } else {
                    buf.push(c);
                }
            }
            self.drain();
            if !buf.is_empty() {
                return T::from_str(buf.as_str());
            }
        }
    }
}
lazy_static! {
    static ref GLOB: Mutex<RefCell<RCin>> = Mutex::new(RefCell::new(RCin::new()));
}

/// One-liner to read
pub fn read<T: FromStr>() -> Option<T> {
    let guard = GLOB.lock().unwrap();
    let mut rc: RefMut<RCin> = (*guard).borrow_mut();
    rc.get().ok()
}

/// One-liner to read and apply unwrap_or_default
pub fn read_safe<T: FromStr + Default>() -> T {
    read().unwrap_or_default()
}

/// One-liner to read until a value is valid
pub fn read_next<T: FromStr>() -> T {
    loop {
        if let Some(t) = read() {
            return t;
        }
    }
}

/// One-liner to read a __nonempty__ line
pub fn read_line() -> String {
    let guard = GLOB.lock().unwrap();
    let rc: Ref<RCin> = (*guard).borrow();
    if rc.auto_flush {
        std::io::stdout().flush().ok();
    }
    let mut buf = String::new();
    while buf.trim().is_empty() {
        buf.clear();
        std::io::stdin().lock().read_line(&mut buf).ok();
    }
    buf
}

/// One-liner to await a newline
pub fn pause() {
    let guard = GLOB.lock().unwrap();
    let rc: Ref<RCin> = (*guard).borrow();
    if rc.auto_flush {
        std::io::stdout().flush().ok();
    }
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
}

/// Clears the internal buffer and returns its contents
pub fn consume() -> String {
    let guard = GLOB.lock().unwrap();
    let mut rc: RefMut<RCin> = (*guard).borrow_mut();
    let out = std::mem::replace(&mut rc.data, Vec::new());
    out.iter().rev().collect()
}

/// Clears the internal buffer
pub fn clear() {
    let guard = GLOB.lock().unwrap();
    let mut rc: RefMut<RCin> = (*guard).borrow_mut();
    rc.data.clear();
}

/// !Set to `true` __by default__!
pub fn set_flush(flush: bool) {
    let guard = GLOB.lock().unwrap();
    let mut rcin: RefMut<RCin> = (*guard).borrow_mut();
    rcin.auto_flush = flush;
}
