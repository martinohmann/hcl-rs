mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn parse(c: &mut Criterion) {
    let tests = testdata::load().unwrap();

    let mut group = c.benchmark_group("parse");

    common::for_each_test(&mut group, &tests, |group, test| {
        let len = test.input.len();

        group.throughput(Throughput::Bytes(len as u64));

        group.bench_function(BenchmarkId::new("hcl-rs", test.name()), |b| {
            hcl::parse(&test.input).unwrap();
            b.iter(|| black_box(hcl::parse(&test.input).unwrap()))
        });

        group.bench_function(BenchmarkId::new("hcl-edit", test.name()), |b| {
            hcl_edit::parser::parse_body(&test.input).unwrap();
            b.iter(|| black_box(hcl_edit::parser::parse_body(&test.input).unwrap()))
        });
    });

    group.finish();
}

criterion_group!(benches, parse);
criterion_main!(benches);
