use criterion::{criterion_group, criterion_main, Criterion};
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
        b.iter(|| {
            let tokens = tokenise(str).unwrap();
            tokens
        })
    });
    
    c.bench_function("cri_json_lexer_parser", |b| {
        b.iter(|| {
            let tokens = tokenise(str).unwrap();
            let verse = parse(tokens).unwrap();
            verse
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);