use phf::phf_set;

const KEYWORDS: phf::Set<&'static str> = phf_set! {
    "key",
    "ref",
    "onVnodeMounted",
    "onVnodeUpdated",
    "onVnodeUnmounted",
    "onVnodeBeforeMount",
    "onVnodeBeforeUpdate",
    "onVnodeBeforeUnmount",
};

const KEYS: &[&str] = &[
    "key",
    "ref",
    "onVnodeMounted",
    "onVnodeUpdated",
    "onVnodeUnmounted",
    "onVnodeBeforeMount",
    "onVnodeBeforeUpdate",
    "onVnodeBeforeUnmount",
];

fn test_phf(s: &str) -> bool {
    KEYWORDS.contains(s)
}
fn test_arr(s: &str) -> bool {
    KEYS.contains(&s)
}

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn test_enum_eq(c: &mut Criterion) {
    for name in ["key", "onVnodeBeforeUnmount", "not_exist"] {
        c.bench_with_input(BenchmarkId::new("test phf", name), &name, |b, n| {
            b.iter(|| test_phf(n));
        });
        c.bench_with_input(BenchmarkId::new("test arr", name), &name, |b, n| {
            b.iter(|| test_arr(n));
        });
    }
}

criterion_group!(benches, test_enum_eq);
criterion_main!(benches);
