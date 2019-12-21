use std::fs::File;
use std::io::prelude::*;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use lexer::{Lexer, LexiconBuilder, Next};

fn bench_kjv(c: &mut Criterion) {
    let mut file = File::open("benches/kjv.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let lexicon = LexiconBuilder::new()
        .ignore_chars(" ")
        .rule(1, "[a-zA-Z]+")
        .unwrap()
        .rule(2, "[0-9]+")
        .unwrap()
        .rule(3, "[.,'\":]")
        .unwrap()
        .build();

    let mut lexer = Lexer::new(&lexicon, &contents);

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("KJV", |b| {
        lexer.reset();

        b.iter(|| loop {
            match lexer.next() {
                Next::Token(_, _) => {}
                Next::Error(_) => {}
                Next::End => break,
            }
        })
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_kjv
}

criterion_main!(benches);
