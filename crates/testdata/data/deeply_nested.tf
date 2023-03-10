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
}
