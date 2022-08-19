use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{Body, Value};

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("specsuite/hcl/terraform.hcl").unwrap();
    let body: Body = hcl::from_str(&input).unwrap();
    let value: Value = hcl::from_str(&input).unwrap();

    c.bench_function("hcl::parse", |b| b.iter(|| hcl::parse(&input)));

    c.bench_function("hcl::from_str::<Body>", |b| {
        b.iter(|| hcl::from_str::<Body>(&input))
    });

    c.bench_function("hcl::from_str::<Value>", |b| {
        b.iter(|| hcl::from_str::<Value>(&input))
    });

    c.bench_function("hcl::to_string(&Body)", |b| {
        b.iter(|| hcl::to_string(&body))
    });

    c.bench_function("hcl::to_string(&Value)", |b| {
        b.iter(|| hcl::to_string(&value))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
