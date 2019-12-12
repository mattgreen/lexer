use std::fs::File;
use std::io::prelude::*;

use criterion::{criterion_group, criterion_main, Criterion};

use lexer::{LexiconBuilder, Lexer, Next};

fn bench_kjv(c: &mut Criterion) {
    let mut file = File::open("benches/kjv.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let lexicon = LexiconBuilder::new()
        .ignore_chars(" ")
        .rule(1, "[a-zA-Z]+").unwrap()
        .rule(2, "[0-9]+").unwrap()
        .rule(3, "[.,'\":]").unwrap()
        .build();

    c.bench_function("KJV", |b| {
        let mut lexer = Lexer::new(&lexicon, &contents);

        b.iter(|| {
            loop {
                match lexer.next() {
                    Next::Token(_, _) => {}
                    Next::Error(_) => {},
                    Next::End => break,
                }
            }
        })
    });
}


criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_kjv
}

criterion_main!(benches);