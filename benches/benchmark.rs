use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    c.bench_function("hcl::parse", |b| {
        let input = std::fs::read_to_string("specsuite/hcl/terraform.hcl").unwrap();

        b.iter(|| hcl::parse(&input))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
