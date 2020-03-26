`cin`-like input from `stdin` for all types that implement `FromStr`.

Useful for quick prototyping and debugging without passing any state around.

(And for people who complain that input in rust is too verbose)

It stores a buffer of the last line and tries to consume it first.
It will block until it finds any sequence of non-whitespace characters.

Depends on the [lazy_static](https://docs.rs/lazy_static) crate for storing global state.

## Example
```rust
let x: i32 = rcin::read_next(); // reads until it finds a valid i32

print!("Enter three numbers: "); // flushes stdout by default before any input
let mut max = std::i32::MIN;
for _ in 0..3 {
    let t = rcin::read_safe();  // safe = unwrap_or_default
    max = std::cmp::max(max, t);
}
println!("Max: {}", max);

print!("Ready to continue?");
rcin::pause(); // wait for newline
```

## Thread safety

`Rcin` is thread safe, but all threads share one buffer.
(Parallel input from `stdin` is not a usable thing, is it?)

`pause` is __not__ a common lock for all threads.

## Corner case

Does __not__ read the input char by char like cin and requires whitespace between groups.

Reading an int:
```text
C++: 17GARBAGE => 17 // perfectly fine lol
RCin: 17GARBAGE => None
```
