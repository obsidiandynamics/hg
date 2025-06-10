use criterion::{Criterion, criterion_group, criterion_main};
use hg::parser::parse;
use std::fs;
use hg::lexer::Tokeniser;

fn criterion_benchmark(c: &mut Criterion) {
    fn bench_file(c: &mut Criterion, name: &str) {
        let data = fs::read_to_string(format!("benches/data/{name}.json")).unwrap();
        let data = data.as_str();

        c.bench_function(format!("cri_json_lexer-{name}").as_str(), |b| {
            let data = std::hint::black_box(data);
            b.iter(|| {
                // let maybe_error = Tokeniser::new(data)
                //     .map(Result::err)
                //     .skip_while(Option::is_none)
                //     .map(Option::unwrap)
                //     .next();
                // assert!(maybe_error.is_none(), "error: {}", maybe_error.unwrap());
                // maybe_error

                // exp_count_tokens(data)

                Tokeniser::new(data).count()
                // data.char_indices().count()
                // Graphemes::from(data).count()

                // BasicParser::from(data).count()
            })
        });

        c.bench_function(format!("cri_json_parser-{name}").as_str(), |b| {
            b.iter_with_setup(
                || Tokeniser::new(data),
                |iter| {
                    let verse = parse(iter).unwrap();
                    verse
                },
            );
        });

        c.bench_function(format!("cri_json_combined-{name}").as_str(), |b| {
            b.iter(|| {
                let verse = parse(Tokeniser::new(data)).unwrap();
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

// enum Element<'a> {
//     Line(Cow<'a, str>),
// }
// 
// struct BasicParser<'a> {
//     bytes: &'a [u8],
//     iter: NewlineTerminatedBytes<'a>,
//     buffer: Buffer,
//     // offset: usize,
// }
// 
// impl<'a> From<&'a str> for BasicParser<'a> {
//     #[inline(always)]
//     fn from(str: &'a str) -> Self {
//         Self {
//             bytes: str.as_bytes(),
//             iter: NewlineTerminatedBytes::new(str.bytes()),
//             buffer: Buffer { offset: 0, len: 0 },
//             // offset: 0
//         }
//     }
// }
// 
// struct Buffer {
//     offset: usize,
//     len: usize,
// }
// 
// impl Buffer {
//     #[inline(always)]
//     fn push(&mut self, offset: usize) {
//         if self.len == 0 {
//             self.offset = offset;
//         }
//         self.len += 1;
//     }
// 
//     #[inline(always)]
//     fn string<'b>(&self, bytes: &'b [u8]) -> Cow<'b, str> {
//         let str = unsafe { str::from_utf8_unchecked(&bytes[self.offset..self.offset + self.len]) };
//         Cow::Borrowed(str)
//     }
// 
//     #[inline(always)]
//     fn clear(&mut self) {
//         self.offset = 0;
//         self.len = 0;
//     }
// }
// 
// impl<'a> Iterator for BasicParser<'a> {
//     type Item = Element<'a>;
// 
//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((offset, byte)) = self.iter.next() {
//             match byte {
//                 b'\n' => {
//                     let line = self.buffer.string(self.bytes);
//                     self.buffer.clear();
//                     return Some(Line(line));
//                 }
//                 _ => self.buffer.push(offset),
//             }
//         }
//         None
//     }
// }
