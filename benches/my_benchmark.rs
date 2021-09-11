#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum TextMode {
    Data,
    RawText,
    RCData,
}
impl std::fmt::Display for TextMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

fn test_eq_data(t: TextMode) -> bool {
    t == TextMode::Data
}
fn test_match_data(t: TextMode) -> bool {
    matches!(t, TextMode::Data)
}

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn test_enum_eq(c: &mut Criterion) {
    use TextMode::*;
    for name in [Data, RawText, RCData].iter() {
        c.bench_with_input(BenchmarkId::new("test enum match", name), &name, |b, &n| {
            b.iter(|| test_match_data(*n));
        });
        c.bench_with_input(BenchmarkId::new("test enum eq", name), &name, |b, &n| {
            b.iter(|| test_eq_data(*n));
        });
    }
}

criterion_group!(benches, test_enum_eq);
criterion_main!(benches);
