use phf::phf_set;

// prefer contains for small array
const KEYWORDS: phf::Set<&'static str> = phf_set! {
  "Infinity",
  "undefined",
  "NaN",
  "isFinite",
  "isNaN",
  "parseFloat",
  "parseInt",
  "decodeURI",
  "decodeURIComponent",
  "encodeURI",
  "encodeURIComponent",
  "Math",
  "Number",
  "Date",
  "Array",
  "Object",
  "Boolean",
  "String",
  "RegExp",
  "Map",
  "Set",
  "JSON",
  "Intl",
  "BigInt",
};

const KEYS: &[&str] = &[
    "Array",
    "BigInt",
    "Boolean",
    "Date",
    "Infinity",
    "Intl",
    "JSON",
    "Map",
    "Math",
    "NaN",
    "Number",
    "Object",
    "RegExp",
    "Set",
    "String",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "undefined",
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
    for name in ["Infinity", "BigInt", "not_exist"] {
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
