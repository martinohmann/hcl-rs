use criterion::{criterion_group, criterion_main, Criterion};
use hcl::{Body, Value};

fn benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("benches/terraform.hcl").unwrap();
    let body: Body = hcl::from_str(&input).unwrap();
    let value: Value = hcl::from_str(&input).unwrap();

    c.bench_function("hcl::parse", |b| b.iter(|| hcl::parse(&input)));

    #[cfg(feature = "nom")]
    c.bench_function("hcl::parser::parse_spanned", |b| {
        b.iter(|| hcl::parser::parse_spanned(&input))
    });

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

    let deeply_nested = r#"
        variable "network_integration" {
          description = "Map of networking integrations between accounts"
          type = map(object({
            friendly_name = string,
            vpcs = map(object({
              id           = string
              cidr         = string
              region       = string
              description  = string
              subnets      = map(string)
              route_tables = map(string)
              security_groups = map(object({
                id = string
                rules = map(object({
                  direction   = string
                  protocol    = string
                  from_port   = string
                  to_port     = string
                  description = string
                }))
              }))
            }))
            additional_propagated_vpcs   = list(string)
            additional_static_vpc_routes = list(string)
          }))
          default = {}
        }"#;

    c.bench_function("hcl::parse(&deeply_nested)", |b| {
        b.iter(|| hcl::parse(deeply_nested))
    });

    #[cfg(feature = "nom")]
    c.bench_function("hcl::parser::parse_spanned(deeply_nested)", |b| {
        b.iter(|| hcl::parser::parse_spanned(deeply_nested))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
