use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{Body, Result, Value};
use serde::{Deserialize, Serialize};

fn roundtrip<'de, T>(input: &'de str) -> Result<String>
where
    T: Deserialize<'de> + Serialize,
{
    let v = hcl::from_str::<T>(input)?;
    hcl::to_string(&v)
}

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("specsuite/hcl/terraform.hcl").unwrap();

    c.bench_function("hcl::parse", |b| b.iter(|| hcl::parse(&input)));

    c.bench_function("hcl::from_str::<Body>", |b| {
        b.iter(|| hcl::from_str::<Body>(&input))
    });

    c.bench_function("hcl::from_str::<Value>", |b| {
        b.iter(|| hcl::from_str::<Value>(&input))
    });

    c.bench_function("roundtrip::<Body>", |b| {
        b.iter(|| roundtrip::<Body>(&input))
    });

    c.bench_function("roundtrip::<Value>", |b| {
        b.iter(|| roundtrip::<Value>(&input))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
