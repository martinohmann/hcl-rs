use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{Body, Value};

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("benches/terraform.tf").unwrap();
    let input_large = std::fs::read_to_string("benches/network.tf").unwrap();
    let deeply_nested = std::fs::read_to_string("benches/deeply_nested.tf").unwrap();
    let body: Body = hcl::from_str(&input).unwrap();
    let value: Value = hcl::from_str(&input).unwrap();

    #[cfg(not(feature = "nom"))]
    {
        c.bench_function("parse", |b| b.iter(|| hcl::parse(&input).unwrap()));
        c.bench_function("parse_large", |b| {
            b.iter(|| hcl::parse(&input_large).unwrap())
        });
        c.bench_function("parse_deeply_nested", |b| {
            b.iter(|| hcl::parse(&deeply_nested).unwrap())
        });
    }

    #[cfg(feature = "nom")]
    {
        c.bench_function("parse", |b| {
            b.iter(|| hcl::parser::parse_spanned(&input).unwrap())
        });

        c.bench_function("parse_large", |b| {
            b.iter(|| hcl::parser::parse_spanned(&input_large).unwrap())
        });

        c.bench_function("parse_deeply_nested", |b| {
            b.iter(|| hcl::parser::parse_spanned(&deeply_nested).unwrap())
        });

        c.bench_function("parse_large_convert", |b| {
            b.iter(|| {
                hcl::parser::parse_spanned(&input_large)
                    .map(|node| Body::from(node.into_value()))
                    .unwrap()
            })
        });

        c.bench_function("parse_deeply_nested_convert", |b| {
            b.iter(|| {
                hcl::parser::parse_spanned(&deeply_nested)
                    .map(|node| Body::from(node.into_value()))
                    .unwrap()
            })
        });

        c.bench_function("parse_unspanned", |b| {
            b.iter(|| hcl::parse(&input).unwrap())
        });
        c.bench_function("parse_unspanned_large", |b| {
            b.iter(|| hcl::parse(&input_large).unwrap())
        });

        c.bench_function("parse_unspanned_deeply_nested", |b| {
            b.iter(|| hcl::parser::parse(&deeply_nested).unwrap())
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
