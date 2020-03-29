 
Input crate that mimics c++ cin/ifstream

Rcin is great for prototyping and offers one crucial advantage over the classical
BufReader approach. The only methods of BufRead/Read that work with strings are `read_to_string`
and `read_line`, which can be unusable if a big file consists only of one or a few lines.

In contrast to similar input streams for rust, rcin can extract single characters.

Rcin also includes a static wrapper over stdin

## Examples
To read from stdin
``` rust
    
    let mut i = rin.read().unwrap_or(2020); // read any type that implements FromString
    
    while rin >> &mut i{ // c++ style operator overload
        println!("{}", i);
    }
    
    cin.read_line(); // read a line
```
To read a file or any source that implements Read
``` rust
    let f = File::open("test.txt").unwrap();
    let mut reader = RInStream::from_file(f); // create RInStream instance
    reader.skip_line();                       // skip first line
    while reader.valid(){                     // while there are no errors from the source
        match reader.read::<i32>(){           // read i32
            Some(v) => (),
            None => ()
        }
    }
```
## Inner mechanics
The inner stream buffers the source data exactly like BufRead with the same default buffer size,
but then tries to extract valid utf8 chars from the byte sequence. 

Utf8 offers many whitespaces, however the most used ones consist only of a single byte(tab, 
space, ...). If any further versions will exist, they might leave out support for uncommon whitespaces, 
to speed up the parsing process and use the builtin utf8 parser. This also means that such streams won't be 
able to read the data char by char efficiently.