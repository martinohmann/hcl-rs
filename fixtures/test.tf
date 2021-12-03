resource "aws_eks_cluster" "this" {
  count = var.create_eks ? 1 : 0

  name                      = var.cluster_name
  enabled_cluster_log_types = var.cluster_enabled_log_types
  role_arn                  = local.cluster_iam_role_arn
  version                   = var.cluster_version

  vpc_config {
    security_group_ids = compact([local.cluster_security_group_id])
    subnet_ids         = var.subnets
  }

  kubernetes_network_config {
    service_ipv4_cidr = var.cluster_service_ipv4_cidr
  }

  dynamic "encryption_config" {
    for_each = toset(var.cluster_encryption_config)

    content {
      provider {
        key_arn = encryption_config.value["provider_key_arn"]
      }
      resources = encryption_config.value["resources"]
    }
  }

  tags = merge(
    var.tags,
    var.cluster_tags,
  )

  depends_on = [
    aws_cloudwatch_log_group.this
  ]
}
