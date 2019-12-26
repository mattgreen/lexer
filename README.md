# TODO: choose name for Rust lexing library

A batteries-included lexing library for Rust based on finite automata.

This is a small library intended to automate much of the tedium associated with lexing so you can move onto more interesting problems.

## Sample Usage

```
let lexicon = LexiconBuilder::new()
    .ignore_chars(" \t\r\n")
    .pattern(1, "[a-zA-Z]+")
    .pattern(2, "[0-9]+")
    .pattern(3, "[.,'\":\\[\\];{}()!?-]")
    .build()
    .unwrap();

let content = "hello there, Gil";
let mut lexer = Lexer::new(&lexicon, &contents);
loop {
    match lexer.next() {
        Next::Token(id, text, pos) => println!("Token: #{}: \"{}\"", text, pos),
        Next::Error(c, pos) => println!("ERROR: {}
        Next::End => break,
    }
}
```

## Features

* Compiles regular expressions to finite automata
* Efficient dispatch of input to applicable rules via lookup tables for first character
* Leftmost-longest match semantics (`iffy` matched instead of `if` `fy` even when `if` is a pattern of interest)
* Optionally specify characters to be ignored (such as whitespace)
* Line and column number tracking

## Roadmap

* Flesh out Unicode support
* Optional support for indent/dedent tokens
* High-level `derive`-based API
* Expand test and benchmark suite
* Convert NFAs to DFAs
* Consider submitting crate to crates.io

## Performance
Times are from a i9 processor running at 2.4ghz.

### Programming Language workload
```
sqlite3.c (7.6 MB of C)    time: 4.6849 ms   849,549 tokens
```

### NLP-type workload
```
kjv.txt (4.1MB of text)    time: 727.46 ns   1,010,863 tokens
```

