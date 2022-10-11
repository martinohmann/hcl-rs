use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{Body, Value};

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("specsuite/hcl/terraform.hcl").unwrap();
    let body: Body = hcl::from_str(&input).unwrap();
    let value: Value = hcl::from_str(&input).unwrap();

    let nested_arrays = r#"
variable "test" {
  level1 = [[[[[[[[[[[[]]]]]]]]]]]]
}
"#;

    c.bench_function("hcl::parse(&nested_arrays)", |b| {
        b.iter(|| hcl::parse(&nested_arrays))
    });

    let nested_objects = r#"
variable "test" {
  level1 = {
    level2 = {
      level3 = {
        level4 = {
          level5 = {
            level6 = {
              level7 = {
                level8 = {
                  level9 = {
                    level10 = {
                      level11 = {
                        level12 = {
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;

    c.bench_function("hcl::parse(&nested_objects)", |b| {
        b.iter(|| hcl::parse(&nested_objects))
    });

    let nested_func_calls = r#"
variable "test" {
  level1 = map(object({
    level2 = map(object({
      level3 = map(object({
        level4 = map(object({
        }))
      }))
    }))
  }))
}
"#;

    c.bench_function("hcl::parse(&nested_func_calls)", |b| {
        b.iter(|| hcl::parse(&nested_func_calls))
    });

    let nested_func_calls2 = r#"
variable "test" {
  level1 = map(map(map(map(map(map(map(map(map(map(map(map())))))))))))
}
"#;

    c.bench_function("hcl::parse(&nested_func_calls2)", |b| {
        b.iter(|| hcl::parse(&nested_func_calls2))
    });

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
