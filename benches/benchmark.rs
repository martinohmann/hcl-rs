use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{parser, Body, Value};

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("benches/terraform.tf").unwrap();
    let input_large = std::fs::read_to_string("benches/network.tf").unwrap();
    let deeply_nested = std::fs::read_to_string("benches/deeply_nested.tf").unwrap();
    let body: Body = hcl::from_str(&input).unwrap();
    let value: Value = hcl::from_str(&input).unwrap();

    c.bench_function("parse", |b| b.iter(|| parser::parse(&input).unwrap()));
    c.bench_function("parse_large", |b| {
        b.iter(|| parser::parse(&input_large).unwrap())
    });
    c.bench_function("parse_deeply_nested", |b| {
        b.iter(|| parser::parse(&deeply_nested).unwrap())
    });

    #[cfg(feature = "nom-spanned")]
    {
        c.bench_function("parse_raw", |b| {
            b.iter(|| parser::parse_raw(&input).unwrap())
        });
        c.bench_function("parse_raw_large", |b| {
            b.iter(|| parser::parse_raw(&input_large).unwrap())
        });
        c.bench_function("parse_raw_deeply_nested", |b| {
            b.iter(|| parser::parse_raw(&deeply_nested).unwrap())
        });
    }

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
