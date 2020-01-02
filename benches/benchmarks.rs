use std::fs::File;
use std::io::prelude::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lexer::{Lexer, Lexicon, LexiconBuilder, Next};

fn c_lexicon() -> Lexicon {
    LexiconBuilder::new()
        .ignore_chars(" \t\r\n")
        .pattern(0, "/\\*([^*]|\\*+[^/])*\\*+/")
        .pattern(1, "#[ \t]*[a-zA-Z]+")
        .pattern(2, "<[a-zA-Z0-9_./]+>")
        .pattern(3, "[a-zA-Z_][a-zA-Z0-9_]*")
        .pattern(4, "[0-9]+")
        .pattern(5, "[+\\-*/!%\\^|&<>=~]+")
        .pattern(6, r#""(\\"|[^"]*)""#)
        .pattern(7, "'[^']+'")
        .literal(100, "(")
        .literal(101, ")")
        .literal(102, ";")
        .literal(103, "{")
        .literal(104, "}")
        .literal(105, ",")
        .literal(106, "[")
        .literal(107, "]")
        .literal(108, "\\")
        .literal(109, "?")
        .literal(110, ":")
        .literal(111, ".")
        .literal(200, "if")
        .literal(201, "else")
        .literal(202, "const")
        .literal(203, "return")
        .literal(204, "for")
        .literal(205, "struct")
        .literal(205, "switch")
        .literal(206, "case")
        .literal(207, "while")
        .literal(208, "do")
        .literal(208, "break")
        .literal(209, "static")
        .literal(210, "extern")
        .build()
        .unwrap()
}

fn bench_iter(c: &mut Criterion) {
    let mut file = File::open("benches/sqlite3.c").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    c.bench_function("raw iteration", |b| {
        b.iter(|| for c in contents.chars() {
            black_box(c.is_ascii());
        });
    });
}

fn bench_sqlite3(c: &mut Criterion) {
    let mut file = File::open("benches/sqlite3.c").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let lexicon = LexiconBuilder::new()
        .ignore_chars(" \t\r\n")
        .pattern(0, "/\\*([^*]|\\*+[^/])*\\*+/")
        .pattern(1, "#[ \t]*[a-zA-Z]+")
        .pattern(2, "<[a-zA-Z0-9_./]+>")
        .pattern(3, "[a-zA-Z_][a-zA-Z0-9_]*")
        .pattern(4, "[0-9]+")
        .pattern(5, "[+\\-*/!%\\^|&<>=~]+")
        .pattern(6, r#""(\\"|[^"]*)""#)
        .pattern(7, "'[^']+'")
        .literal(100, "(")
        .literal(101, ")")
        .literal(102, ";")
        .literal(103, "{")
        .literal(104, "}")
        .literal(105, ",")
        .literal(106, "[")
        .literal(107, "]")
        .literal(108, "\\")
        .literal(109, "?")
        .literal(110, ":")
        .literal(111, ".")
        .literal(200, "if")
        .literal(201, "else")
        .literal(202, "const")
        .literal(203, "return")
        .literal(204, "for")
        .literal(205, "struct")
        .literal(205, "switch")
        .literal(206, "case")
        .literal(207, "while")
        .literal(208, "do")
        .literal(208, "break")
        .literal(209, "static")
        .literal(210, "extern")
        .build()
        .unwrap();

    let mut lexer = Lexer::new(&lexicon, &contents);
    let mut count = 0;

    c.bench_function("sqlite3.c", |b| {
        b.iter(|| {
            lexer.reset();
            count = 0;

            while let Some(t) = lexer.next() {
            match t {
                Next::Token(_, _, _) => count += 1,
                Next::Error(_, _) => {}
            }
        }})
    });
    println!("{}", count);
}

fn bench_gzip(c: &mut Criterion) {
    let mut file = File::open("benches/gzip.c").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut lexer = Lexer::new(&c_lexicon(), &contents);
    let mut count = 0;

    c.bench_function("gzip.c", |b| {
        b.iter(|| {
            lexer.reset();
            count = 0;

            while let Some(t) = lexer.next() {
            match t {
                Next::Token(_, _, _) => count += 1,
                Next::Error(_, _) => {}
            };
        }});
    });
    println!("{}", count);
}

fn bench_kjv(c: &mut Criterion) {
    let mut file = File::open("benches/kjv.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let lexicon = LexiconBuilder::new()
        .ignore_chars(" \t\r\n")
        .pattern(1, "[a-zA-Z]+")
        .pattern(2, "[0-9]+")
        .pattern(3, "[.,'\":\\[\\];{}()!?-]")
        .build()
        .unwrap();

    let mut lexer = Lexer::new(&lexicon, &contents);
    let mut count = 0;

    c.bench_function("KJV", |b| {
        b.iter(|| {
            lexer.reset();
            count = 0;

            while let Some(t) = lexer.next() {
            match t {
                Next::Token(_, _, _) => count += 1,
                Next::Error(_, _) => {}
            };
        }});
    });
    println!("{}", count);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_iter, bench_gzip, bench_sqlite3, bench_kjv
}

criterion_main!(benches);
