use criterion::{criterion_group, criterion_main, Criterion};
use hg::graphemes::Graphemes;

fn criterion_benchmark(c: &mut Criterion) {
    fn bench_str(c: &mut Criterion, name: &str, data: &str) {
        c.bench_function(format!("cri_graphemes-{name}").as_str(), |b| {
            b.iter(|| {
                Graphemes::from(data).count()
            })
        });
    }
    
    bench_str(c, "ascii", r#"
    I was not surprised. Indeed, my only wonder was that he had not already been mixed upon 
    this extraordinary case, which was the one topic of conversation through the length and 
    breadth of England. For a whole day my companion had rambled about the room with his chin 
    upon his chest and his brows knitted, charging and recharging his pipe with the 
    strongest black tobacco, and absolutely deaf to any of my questions or remarks. Fresh 
    editions of every paper had been sent up by our news agent, only to be glanced over and 
    tossed down into a corner. Yet, silent as he was, I knew perfectly well what it was over 
    which he was brooding. There was but one problem before the public which could challenge 
    his powers of analysis, and that was the singular disappearance of the favorite for the 
    Wessex Cup, and the tragic murder of its trainer. When, therefore, he suddenly announced 
    his intention of setting out for the scene of the drama it was only what I had both 
    expected and hoped for.
    "#);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
