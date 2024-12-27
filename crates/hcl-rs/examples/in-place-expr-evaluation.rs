use hcl::eval::{Context, Evaluate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(filename) = std::env::args().nth(1) else {
        eprintln!("filename argument required");
        std::process::exit(1);
    };

    let input = std::fs::read_to_string(filename)?;
    let mut body = hcl::parse(&input)?;
    let ctx = Context::new();

    // This will try to evaluate all expressions in `body` and updates it in-place, returning all
    // errors that occurred along the way.
    if let Err(errors) = body.evaluate_in_place(&ctx) {
        eprintln!("{errors}");
    }

    hcl::to_writer(std::io::stdout(), &body)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use hcl::eval::{FuncDef, ParamType};
    use hcl::value::Value;
    use hcl::Map;
    use pretty_assertions::assert_eq;

    #[test]
    fn exprs_are_evaluated_in_place() {
        let input = indoc::indoc! {r#"
            resource "aws_eks_cluster" "this" {
              count = var.create_eks ? 1 : 0

              name     = var.cluster_name
              role_arn = local.cluster_iam_role_arn
              version  = var.cluster_version

              vpc_config {
                security_group_ids = compact([local.cluster_security_group_id])
                subnet_ids         = var.subnets
              }

              kubernetes_network_config {
                service_ipv4_cidr = var.cluster_service_ipv4_cidr
              }

              tags = merge(
                var.tags,
                var.cluster_tags,
              )
            }
        "#};

        let mut body = hcl::parse(input).unwrap();
        let mut ctx = Context::new();
        ctx.declare_var(
            "var",
            hcl::value!({
                "create_eks" = true
                "cluster_name" = "mycluster"
                "cluster_tags" = {
                    "team" = "ops"
                }
                "cluster_version" = "1.27.0"
                "tags" = {
                    "environment" = "dev"
                }
            }),
        );

        ctx.declare_func(
            "merge",
            FuncDef::builder()
                .variadic_param(ParamType::Any)
                .build(|args| {
                    let mut map = Map::<String, Value>::new();
                    for arg in args.variadic_args() {
                        if let Some(object) = arg.as_object() {
                            map.extend(object.clone());
                        } else {
                            return Err(format!("Argument {:?} is not an object", arg));
                        }
                    }

                    Ok(Value::Object(map))
                }),
        );

        let res = body.evaluate_in_place(&ctx);
        assert!(res.is_err());

        let errors = res.unwrap_err();

        assert_eq!(errors.len(), 4);
        assert_eq!(
            errors.to_string(),
            indoc::indoc! {r#"
                4 errors occurred:
                - undefined variable `local` in expression `local.cluster_iam_role_arn`
                - undefined variable `local` in expression `local.cluster_security_group_id`
                - no such key: `subnets` in expression `var.subnets`
                - no such key: `cluster_service_ipv4_cidr` in expression `var.cluster_service_ipv4_cidr`
            "#}
        );

        let expected = indoc::indoc! {r#"
            resource "aws_eks_cluster" "this" {
              count = 1
              name = "mycluster"
              role_arn = local.cluster_iam_role_arn
              version = "1.27.0"

              vpc_config {
                security_group_ids = compact([local.cluster_security_group_id])
                subnet_ids = var.subnets
              }

              kubernetes_network_config {
                service_ipv4_cidr = var.cluster_service_ipv4_cidr
              }

              tags = {
                "environment" = "dev"
                "team" = "ops"
              }
            }
        "#};

        assert_eq!(hcl::to_string(&body).unwrap(), expected);
    }
}
