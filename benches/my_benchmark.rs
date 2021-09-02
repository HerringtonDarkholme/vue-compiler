#[derive(PartialEq, Eq)]
enum TextMode {
    Data,
    RawText,
    RCData,
}

trait OptionLike {
    fn skip(&self) -> bool;
    fn get_name(s: &str) -> TextMode;
}
struct Option {
    skip: bool,
}
impl OptionLike for Option {
    #[inline]
    fn skip(&self) -> bool {
        self.skip
    }
    fn get_name(s: &str) -> TextMode {
        get_name(s)
    }
}

fn get_name(s: &str) -> TextMode {
    match s {
        "script" => TextMode::RawText,
        "textarea" => TextMode::RCData,
        _ => TextMode::Data,
    }
}

fn is_special<O: OptionLike>(s: &str, opt: &O) -> bool {
    opt.skip() && O::get_name(s) == TextMode::Data
}

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn from_elem(c: &mut Criterion) {
    for name in [
        (Option { skip: true }, "script"),
        (Option { skip: true }, "textarea"),
        (Option { skip: true }, "div"),
        (Option { skip: false }, "script"),
        (Option { skip: false }, "textarea"),
        (Option { skip: false }, "div"),
    ]
    .iter()
    {
        let n = if name.0.skip {
            "test opt: skipped"
        } else {
            "test opt: no skip"
        };
        c.bench_with_input(BenchmarkId::new(n, name.1), &name, |b, &n| {
            b.iter(|| is_special(n.1, &n.0));
        });
    }
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
