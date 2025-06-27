mod common;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use hcl::{Body, Value};

fn ser(c: &mut Criterion) {
    let tests = testdata::load().unwrap();

    let mut group = c.benchmark_group("ser");

    common::for_each_test(&mut group, &tests, |group, test| {
        let body: Body = hcl::from_str(&test.input).unwrap();
        let value: Value = hcl::from_str(&test.input).unwrap();

        group.bench_function(BenchmarkId::new("body", test.name()), |b| {
            hcl::to_string(&body).unwrap();
            b.iter(|| black_box(hcl::to_string(&body).unwrap()))
        });

        group.bench_function(BenchmarkId::new("value", test.name()), |b| {
            hcl::to_string(&value).unwrap();
            b.iter(|| black_box(hcl::to_string(&value).unwrap()))
        });
    });

    group.finish();
}

fn de(c: &mut Criterion) {
    let tests = testdata::load().unwrap();

    let mut group = c.benchmark_group("de");

    common::for_each_test(&mut group, &tests, |group, test| {
        let body: Body = hcl::from_str(&test.input).unwrap();

        group.bench_function(BenchmarkId::new("body", test.name()), |b| {
            hcl::from_body::<Body>(body.clone()).unwrap();
            b.iter_batched(
                || body.clone(),
                |body| black_box(hcl::from_body::<Body>(body).unwrap()),
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("value", test.name()), |b| {
            hcl::from_body::<Value>(body.clone()).unwrap();
            b.iter_batched(
                || body.clone(),
                |body| black_box(hcl::from_body::<Value>(body).unwrap()),
                BatchSize::SmallInput,
            )
        });
    });

    group.finish();
}

criterion_group!(benches, ser, de);
criterion_main!(benches);
