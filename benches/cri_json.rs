use std::io::BufReader;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use hg::lexer::tokenise;
use hg::parser::parse;

fn criterion_benchmark(c: &mut Criterion) {
    let str = r#"{
        "key1": "value1",
        "key2": 1234,
        "key3": 1234.5678,
        "key4": -345,
        "key5": true,
        "key6": null,
        "emptyArray": [
        ],
        "employees": [
            {
                "id": 1,
                "details": {"name": "John Wick", "age": 42, "dogOwner": true}
            },
            {
                "id": 2,
                "details": {"name": "Max Payne", "age": 39, "dogOwner": false}
            }
        ]
    }"#;

    c.bench_function("cri_json_lexer", |b| {
        b.iter_batched(|| {
            BufReader::with_capacity(1_000, str.as_bytes())
        }, |reader| {
            let tokens = tokenise(reader).unwrap();
            tokens
        }, BatchSize::LargeInput)
    });
    
    c.bench_function("cri_json_lexer_parser", |b| {
        b.iter_batched(|| {
            BufReader::with_capacity(1_000, str.as_bytes())
        }, |reader| {
            let tokens = tokenise(reader).unwrap();
            let verse = parse(tokens).unwrap();
            verse
        }, BatchSize::LargeInput)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);