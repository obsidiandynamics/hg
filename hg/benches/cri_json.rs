use criterion::{Criterion, criterion_group, criterion_main};
use hg::parser::parse;
use std::fs;
use hg::lexer::Tokeniser;
use hg::symbols::SymbolTable;

fn criterion_benchmark(c: &mut Criterion) {
    fn bench_file(c: &mut Criterion, name: &str) {
        let data = fs::read_to_string(format!("hg/benches/data/{name}.json")).unwrap();
        let data = data.as_str();

        c.bench_function(format!("cri_json_lexer-{name}").as_str(), |b| {
            let data = std::hint::black_box(data);
            b.iter(|| {
                Tokeniser::new(data, SymbolTable::default()).count()
            })
        });

        c.bench_function(format!("cri_json_parser-{name}").as_str(), |b| {
            let data = std::hint::black_box(data);
            b.iter_with_setup(
                || Tokeniser::new(data, SymbolTable::default()),
                |iter| {
                    let verse = parse(iter).unwrap();
                    verse
                },
            );
        });

        c.bench_function(format!("cri_json_combined-{name}").as_str(), |b| {
            let data = std::hint::black_box(data);
            b.iter(|| {
                let verse = parse(Tokeniser::new(data, SymbolTable::default())).unwrap();
                verse
            });
        });
    }
    bench_file(c, "canada");
    bench_file(c, "citm");
    bench_file(c, "small");
    bench_file(c, "text-1kb");
    bench_file(c, "twitter");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
