mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hcl::parser;

fn parse(c: &mut Criterion) {
    let tests = testdata::load().unwrap();

    let mut group = c.benchmark_group("parse");

    common::for_each_test(&mut group, &tests, |group, test| {
        let len = test.input.len();

        group.throughput(Throughput::Bytes(len as u64));

        group.bench_function(BenchmarkId::new("simple", test.name()), |b| {
            parser::parse(&test.input).unwrap();
            b.iter(|| black_box(parser::parse(&test.input).unwrap()))
        });

        #[cfg(feature = "winnow")]
        group.bench_function(BenchmarkId::new("raw", test.name()), |b| {
            parser::parse_raw(&test.input).unwrap();
            b.iter(|| black_box(parser::parse_raw(&test.input).unwrap()))
        });
    });

    group.finish();
}

criterion_group!(benches, parse);
criterion_main!(benches);
