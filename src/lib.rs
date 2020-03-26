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
//!
//! ```no_run
//! use rcin::cin;
//!
//! let x: i32 = cin.read_next(); // reads until it finds a valid i32
//!
//! print!("Enter three numbers: "); // flushes stdout by default before any input
//! let mut max = std::i32::MIN;
//! for _ in 0..3{
//!     let t = cin.read_safe();  // safe = unwrap_or_default
//!     max = std::cmp::max(max, t);
//! }
//! println!("Max: {}", max);
//!
//! print!("Ready to continue?");
//! cin.pause(); //wait for newline
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
struct RCinInner {
    data: Vec<char>,
    auto_flush: bool,
}
impl RCinInner {
    const fn new() -> Self {
        RCinInner {
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
    #[derive(Debug, Copy, Clone)]
    pub static ref cin: Rcin = Rcin::new();
    static ref GLOB: Mutex<RefCell<RCinInner>> = Mutex::new(RefCell::new(RCinInner::new()));
}

impl<T> std::ops::Shr<&mut T> for cin
where T: FromStr {
    type Output = bool;
    fn shr(self, rhs: &mut T) -> Self::Output {
        if let Some(value) = self.read() {
            *rhs = value;
            true
        } else {
            false
        }
    }
}

pub struct Rcin {}

impl Rcin {

    fn new() -> Rcin { Rcin {} }

    /// One-liner to read
    pub fn read<T: FromStr>(&self) -> Option<T> {
        let guard = GLOB.lock().unwrap();
        let mut rc: RefMut<RCinInner> = (*guard).borrow_mut();
        rc.get().ok()
    }

    /// One-liner to read and apply unwrap_or_default
    pub fn read_safe<T: FromStr + Default>(&self) -> T{
        self.read().unwrap_or_default()
    }

    /// One-liner to read until a value is valid
    pub fn read_next<T: FromStr>(&self) -> T{
        loop {
            match self.read(){
                Some(t) => return t,
                _ => ()
            }
        }
    }

    /// One-liner to read a __nonempty__ line
    pub fn read_line(&self) -> String {
        let guard = GLOB.lock().unwrap();
        let rc: Ref<RCinInner> = (*guard).borrow();
        if rc.auto_flush{
            std::io::stdout().flush().ok();
        }
        let mut buf = String::new();
        while buf.trim().len() == 0{
            buf.clear();
            std::io::stdin().lock().read_line(&mut buf).ok();
        }
        buf
    }

    /// One-liner to await a newline
    pub fn pause(&self){
        let guard = GLOB.lock().unwrap();
        let rc: Ref<RCinInner> = (*guard).borrow();
        if rc.auto_flush{
            std::io::stdout().flush().ok();
        }
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
    }

    /// Clears the internal buffer and returns its contents
    pub fn consume(&self) -> String{
        let guard = GLOB.lock().unwrap();
        let mut rc: RefMut<RCinInner> = (*guard).borrow_mut();
        let out = std::mem::replace(&mut rc.data, Vec::new());
        out.iter().rev().collect()
    }

    /// Clears the internal buffer
    pub fn clear(&self){
        let guard = GLOB.lock().unwrap();
        let mut rc: RefMut<RCinInner> = (*guard).borrow_mut();
        rc.data.clear();
    }

    /// !Set to `true` __by default__!
    pub fn set_flush(&self, flush: bool) {
        let guard = GLOB.lock().unwrap();
        let mut rcin: RefMut<RCinInner> = (*guard).borrow_mut();
        rcin.auto_flush = flush;
    }
}
