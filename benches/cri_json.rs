use criterion::{criterion_group, criterion_main, Criterion};
use hg::lexer::Tokeniser;
use hg::parser::parse;
use std::collections::VecDeque;
use std::fs;

fn criterion_benchmark(c: &mut Criterion) {
    fn bench_file(c: &mut Criterion, name: &str) {
        let data = fs::read_to_string(format!("benches/data/{name}.json")).unwrap();
        let data = data.as_str();
        
        c.bench_function(format!("cri_json_lexer-{name}").as_str(), |b| {
            b.iter(|| {
                let maybe_error = Tokeniser::new(data)
                    .map(Result::err)
                    .skip_while(Option::is_none)
                    .map(Option::unwrap)
                    .next();
                assert!(maybe_error.is_none(), "error: {}", maybe_error.unwrap());
                maybe_error

                // Tokeniser::new(data).count()
                // data.char_indices().count()
            })
        });

        c.bench_function(format!("cri_json_parser-{name}").as_str(), |b| {
            b.iter_with_setup(
            || Tokeniser::new(data).map(Result::unwrap).collect::<VecDeque<_>>(),
            |tokens| {
                let verse = parse(tokens).unwrap();
                verse
            });
        });
    }
    bench_file(c, "canada");
    bench_file(c, "citm");
    bench_file(c, "small");
    bench_file(c, "twitter");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
