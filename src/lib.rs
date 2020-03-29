//!  
//! Input crate that mimics c++ cin/ifstream
//!
//! Rcin is great for prototyping and offers one crucial advantage over the classical
//! BufReader approach. The only methods of BufRead/Read that work with strings are `read_to_string`
//! and `read_line`, which can be unusable if a big file consists only of one or a few lines.
//!
//! In contrast to similar input streams for rust, rcin can extract single characters.
//!
//! Rcin also includes a static wrapper over stdin
//!
//! ## Examples
//!
//! To read from stdin
//!
//! ``` rust
//!     use rcin::rin;
//!
//!     // read any type that implements FromString
//!     let mut i = rin.read().unwrap_or(2020);
//!     // c++ style operator overload
//!     while rin >> &mut i{
//!         println!("{}", i);
//!     }
//!     // read a line
//!     rin.read_line();
//! ```
//!
//! To read a file or any source that implements Read
//!
//! ``` rust
//!     use std::fs::File;
//!     use rcin::RInStream;
//!
//!     let f = File::open("test.txt").unwrap();
//!     let mut reader = RInStream::from_file(f); // create RInStream instance
//!     reader.skip_line();                       // skip first line
//!     while reader.valid(){                     // no errors from the source
//!         match reader.read::<i32>(){
//!             Some(v) => (),
//!             None => ()
//!         }
//!     }
//! ```
//!
//! ## Inner mechanics
//!
//! The inner stream buffers the source data exactly like BufRead with the same default buffer size,
//! but then tries to extract valid utf8 chars from the byte sequence.
//!
//! Utf8 offers many whitespaces, however the most used ones consist only of a single byte(tab,
//! space, ...). If any further versions will exist, they might leave out support for uncommon whitespaces,
//! to speed up the parsing process and use the builtin utf8 parser. This also means that such streams won't be
//! able to read the data char by char efficiently.
//!

use lazy_static::lazy_static;
use std::cell::{RefCell, RefMut};
use std::io::{stdin, Read};
use std::str::FromStr;
use std::sync::Mutex;
use std::fs::File;

const DEFAULT_BUF_SIZE: usize = 8_000; //8 KB like BufReader

/*
    Internal buffered stream that reads char by char using an utf8 decoder
*/
struct Stream {
    source: Box<dyn Read + Send>,
    buf: Vec<u8>,
    ptr: usize,
    limit: usize,
    error: bool, // true when the source returns an error
}
impl Stream {
    fn new(source: Box<dyn Read + Send>, buf_size: usize) -> Self {
        let vc = vec![0; buf_size];
        Stream {
            source,
            buf: vc,
            ptr: 0,
            limit: 0,
            error: false,
        }
    }
    fn refill(&mut self) {
        let res = self.source.read(&mut self.buf);
        self.ptr = 0;
        match res {
            Ok(0) => {
                self.error = true;
            }
            Ok(n) => {
                self.limit = n;
                self.error = false;
            }
            Err(_e) => {
                self.error = true;
            }
        }
    }
    fn pop_byte(&mut self) -> Option<u8> {
        // no bytes left in buffer
        if self.ptr >= self.limit {
            self.refill();
        }
        if self.error {
            None
        } else {
            self.ptr += 1;
            Some(self.buf[self.ptr - 1])
        }
    }
    // decoder tested on https://onlineutf8tools.com/convert-utf8-to-bytes
    fn pop_char(&mut self) -> Option<char> {
        let c1: u32 = self.pop_byte()? as u32;
        let res: u32;

        if c1 & 0xC0 == 0x80{
            return None;
        }
        /*
          Zero continuation (0 to 127)
        */
        if (c1 & 0x80) == 0 {
            res = c1;
            return std::char::from_u32(res);
        }
        /*
            One continuation (128 to 2047)
        */
        if (c1 & 0xE0) == 0xC0 {
            let c2 = self.pop_byte()? as u32;
            res = ((c1 & 0x1F) << 6) | (c2 & 0x3F << 0);
            return std::char::from_u32(res);
        }
        /*
            Two continuations (2048 to 55295 and 57344 to 65535)
        */
        else if (c1 & 0xF0) == 0xE0 {
            let c2 = self.pop_byte()? as u32;
            let c3 = self.pop_byte()? as u32;
            res = ((c1 & 0x0F) << 12) | (c2 & 0x3F << 6) | (c3 & 0x3F << 0);
            return std::char::from_u32(res);
        }
        /*
            Three continuations (65536 to 1114111)
        */
        else if (c1 & 0xF8) == 0xF0 {
            let c2 = self.pop_byte()? as u32;
            let c3 = self.pop_byte()? as u32;
            let c4 = self.pop_byte()? as u32;
            res = (c1 & 0x07 << 18) | (c2 & 0x3F << 12) | (c3 & 0x3F << 6) | (c4 & 0x3F << 0);
            return std::char::from_u32(res);
        }
        None
    }
    fn read<T: FromStr>(&mut self) -> Option<T> {
        let mut buf = String::new();
        loop {
            match self.pop_char() {
                None => break, //maybe eof
                Some(c) => {
                    if c.is_whitespace() {
                        if !buf.is_empty() {
                            // whitespace after data
                            break;
                        }
                    } else {
                        buf.push(c);
                    }
                }
            }
        }
        if buf.is_empty() {
            return None;
        }
        T::from_str(&buf).ok()
    }
    fn read_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        loop {
            match self.pop_char() {
                None => break, //might be EOF
                Some('\n') => break,
                Some(c) => buf.push(c),
            }
        }
        if self.error && buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }
    fn skip_line(&mut self) {
        loop {
            match self.pop_char() {
                None => break,
                Some('\n') => break,
                _ => (),
            }
        }
    }
    fn valid(&self) -> bool {
        !self.error
    }
}

/*
    CIN
*/

/// Stateless wrapper around stdin stream
pub struct RCin;
impl RCin {
    /// Read value
    pub fn read<T: FromStr>(&self) -> Option<T> {
        let guard = GLOB_STREAM.lock().unwrap();
        let mut rc: RefMut<Stream> = (*guard).borrow_mut();
        rc.read()
    }
    /// Read line
    pub fn read_line(&self) -> Option<String> {
        let guard = GLOB_STREAM.lock().unwrap();
        let mut rc: RefMut<Stream> = (*guard).borrow_mut();
        rc.read_line()
    }
    /// Skip all chars until next newline
    pub fn skip_line(&self) {
        let guard = GLOB_STREAM.lock().unwrap();
        let mut rc: RefMut<Stream> = (*guard).borrow_mut();
        rc.skip_line()
    }
    /// Read the next character (can be whitespace)
    pub fn read_char(&self) -> Option<char>{
        let guard = GLOB_STREAM.lock().unwrap();
        let mut rc: RefMut<Stream> = (*guard).borrow_mut();
        rc.pop_char()
    }
}
impl<T> std::ops::Shr<&mut T> for rin
where
    T: FromStr,
{
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

lazy_static! {
    /// Global stdin stream instance
    #[derive(Copy, Clone)]
    pub static ref rin: RCin = RCin{};
    static ref GLOB_STREAM: Mutex<RefCell<Stream>> =
        Mutex::new(RefCell::new(Stream::new(Box::new(stdin()),DEFAULT_BUF_SIZE)));
}

/*
    RInStream
*/
/// Stream based on a source
pub struct RInStream {
    source: Stream,
}
impl RInStream {
    /// Create new stream from file
    pub fn from_file(f: File) -> Self {
        Self::from_source(Box::new(f))
    }
    /// Create new stream from source
    pub fn from_source(src: Box<dyn Read + Send>) -> Self {
        Self::new(src, DEFAULT_BUF_SIZE)
    }
    /// Create new stream from source with given buffer size in bytes
    pub fn new(src: Box<dyn Read + Send>, cap: usize) -> Self {
        RInStream {
            source: Stream::new(src, cap),
        }
    }
    /// Read the next character (can be whitespace)
    pub fn read_char(&mut self) -> Option<char>{
        self.source.pop_char()
    }
    /// Read value
    pub fn read<T: FromStr>(&mut self) -> Option<T> {
        self.source.read()
    }
    /// Read line
    pub fn read_line(&mut self) -> Option<String> {
        self.source.read_line()
    }
    /// Skip all chars until next newline
    pub fn skip_line(&mut self) {
        self.source.skip_line()
    }
    /// Stream becomes invalid as soon as the source returns an error
    pub fn valid(&self) -> bool {
        self.source.valid()
    }
}

