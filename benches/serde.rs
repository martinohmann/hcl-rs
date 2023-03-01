mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hcl::{Body, Value};

fn ser(c: &mut Criterion) {
    let tests = common::load_tests().unwrap();

    let mut group = c.benchmark_group("ser");

    common::for_each_test(&mut group, &tests, |group, test| {
        let body: Body = hcl::from_str(&test.input).unwrap();
        let value: Value = hcl::from_str(&test.input).unwrap();

        group.bench_function(BenchmarkId::new("body", &test.id), |b| {
            hcl::to_string(&body).unwrap();
            b.iter(|| black_box(hcl::to_string(&body).unwrap()))
        });

        group.bench_function(BenchmarkId::new("value", &test.id), |b| {
            hcl::to_string(&value).unwrap();
            b.iter(|| black_box(hcl::to_string(&value).unwrap()))
        });
    });

    group.finish();
}

fn de(c: &mut Criterion) {
    let tests = common::load_tests().unwrap();

    let mut group = c.benchmark_group("de");

    common::for_each_test(&mut group, &tests, |group, test| {
        let len = test.input.len();

        group.throughput(Throughput::Bytes(len as u64));

        group.bench_function(BenchmarkId::new("body", &test.id), |b| {
            hcl::from_str::<Body>(&test.input).unwrap();
            b.iter(|| black_box(hcl::from_str::<Body>(&test.input).unwrap()))
        });

        group.bench_function(BenchmarkId::new("value", &test.id), |b| {
            hcl::from_str::<Value>(&test.input).unwrap();
            b.iter(|| black_box(hcl::from_str::<Value>(&test.input).unwrap()))
        });
    });

    group.finish();
}

criterion_group!(benches, ser, de);
criterion_main!(benches);
