# This creates a network acl for the public subnets of the VPC.
resource "aws_network_acl" "public" {
  vpc_id = aws_vpc.vpc.id

  subnet_ids = aws_subnet.public.*.id

  tags = {
    Name        = "public.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# ---------------------------------------------------------------------------
# INGRESS ACLS
# These control what kind of traffic is allowed to enter the public subnets.
# ---------------------------------------------------------------------------

# Allow all TCP traffic from whitelisted IP addresses
resource "aws_network_acl_rule" "allow-tcp-whitelisted-external-cidr-blocks" {
  count          = length(var.allowed_external_cidr_blocks)
  network_acl_id = aws_network_acl.public.id
  rule_number    = count.index + 50
  egress         = false
  protocol       = "tcp"
  rule_action    = "allow"
  cidr_block     = var.allowed_external_cidr_blocks[count.index]
  from_port      = 0
  to_port        = 65535
}

# Allow ICMP traffic
resource "aws_network_acl_rule" "allow-icmp-public" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 100
  egress         = false
  protocol       = "icmp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  icmp_type      = -1
  icmp_code      = -1
}

# Allow all traffic from public ip of nat gateway to this group.
resource "aws_network_acl_rule" "allow-public-all-nat" {
  count          = var.allow_nat_traffic ? var.availability_zone_count : 0
  network_acl_id = aws_network_acl.public.id
  rule_number    = count.index + 111
  egress         = false
  protocol       = "-1"
  rule_action    = "allow"
  cidr_block     = "${aws_eip.nat[count.index].public_ip}/32"
  from_port      = 0
  to_port        = 65535
}

# In case we open a connection from VPC->Internet, we get a random unprivileged
# from-port. This ports need to be able to receive answers (e.g. syn-ack).
resource "aws_network_acl_rule" "allow-tcp-unpriv-public" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 150
  egress         = false
  protocol       = "tcp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 1024
  to_port        = 65535
}

# Needed to allow udp answer back to nat gateway so that it can foward that
# into private subnets.
resource "aws_network_acl_rule" "allow-udp-unpriv-public" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 175
  egress         = false
  protocol       = "udp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 1024
  to_port        = 65535
}

# Allow http traffic if `var.allow_http_traffic` is enabled.
resource "aws_network_acl_rule" "allow-http-public" {
  count          = var.allow_http_traffic ? 1 : 0
  network_acl_id = aws_network_acl.public.id
  rule_number    = 300
  egress         = false
  protocol       = "tcp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 80
  to_port        = 80
}

# Allow https traffic if `var.allow_https_traffic` is enabled.
resource "aws_network_acl_rule" "allow-https-public" {
  count          = var.allow_https_traffic ? 1 : 0
  network_acl_id = aws_network_acl.public.id
  rule_number    = 400
  egress         = false
  protocol       = "tcp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 443
  to_port        = 443
}

# ---------------------------------------------------------------------------
# VPC INTERNAL ACLS
# These control what kind of traffic is allowed within the VPC.
# ---------------------------------------------------------------------------

# Allow all VPC internal traffic by default.
resource "aws_network_acl_rule" "allow-all-protocols-in-vpc" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 700
  egress         = false
  protocol       = "-1"
  rule_action    = "allow"
  cidr_block     = aws_vpc.vpc.cidr_block
}

# ---------------------------------------------------------------------------
# EGRESS ACLS
# These control what kind of traffic is allowed to leave the public subnets.
# ---------------------------------------------------------------------------

# Allow outgoing TCP traffic.
resource "aws_network_acl_rule" "allow-tcp-out-vpc" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 100
  egress         = true
  protocol       = "tcp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 0
  to_port        = 65535
}

# Allow outgoing UDP traffic.
resource "aws_network_acl_rule" "allow-udp-out-vpc" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 200
  egress         = true
  protocol       = "udp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  from_port      = 0
  to_port        = 65535
}

# Allow outgoing ICMP traffic.
resource "aws_network_acl_rule" "allow-icmp-out-vpc" {
  network_acl_id = aws_network_acl.public.id
  rule_number    = 300
  egress         = true
  protocol       = "icmp"
  rule_action    = "allow"
  cidr_block     = "0.0.0.0/0"
  icmp_type      = -1
  icmp_code      = -1
}
# This creates an internal dns zone for each configured availability zone.
resource "aws_route53_zone" "internal" {
  count = var.availability_zone_count

  name = "${local.vpc_azs[count.index]}.${local.vpc_name}.${var.dns_domain}"

  vpc {
    vpc_id     = aws_vpc.vpc.id
    vpc_region = local.vpc_region
  }

  tags = {
    Name        = "${local.vpc_azs[count.index]}.${local.vpc_name}.${var.dns_domain}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# One internet gateway for all hosts in public subnets.
resource "aws_internet_gateway" "internet_gateway" {
  vpc_id = aws_vpc.vpc.id

  tags = {
    Name        = "igw.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# One elastic ip per nat gateway.
resource "aws_eip" "nat" {
  count = var.availability_zone_count

  tags = {
    Name        = "eip.nat.${local.vpc_azs[count.index]}.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# One nat gateway per availability zone.
resource "aws_nat_gateway" "nat_gateway" {
  count = var.availability_zone_count

  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  # Internet gateways cannot be deleted if there are still nat gateways around,
  # so we force terraform to delete the nat gateways first.
  depends_on = [aws_internet_gateway.internet_gateway]

  tags = {
    Name        = "nat.${local.vpc_azs[count.index]}.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

/**
 * This module creates a VPC with public and private subnets, internet/NAT
 * gateways, internal DNS zones and network ACLs. The sizing of the VPC and
 * subnets can be configured. The number of public subnets is equal to the
 * number of configured availability zones, whereas the number of private
 * subnets can be configured separately.
 */

# ---------------------------------------------------------------------------
# REQUIRED PARAMETERS
# You must provide a value for each of these parameters.
# ---------------------------------------------------------------------------

variable "account" {
  description = "The AWS account name where the VPC is deployed (e.g. globalservices). This will show up in internal DNS entries and tags for resources associated with the network."
  type        = string
}

variable "vpc_address" {
  description = "The base address of the VPC e.g. 10.64.0.0."
  type        = string
}

# ---------------------------------------------------------------------------
# OPTIONAL PARAMETERS
# These parameters have reasonable defaults.
# ---------------------------------------------------------------------------

variable "additional_public_subnet_tags" {
  default     = {}
  description = "Additional tags that should be added to public subnets (e.g. kubernetes.io/cluster/<cluster-id>=shared)."
  type        = map(string)
}

variable "additional_private_subnet_tags" {
  default     = {}
  description = "Additional tags that should be added to private subnets (e.g. kubernetes.io/role/elb=1)."
  type        = map(string)
}

variable "allow_http_traffic" {
  default     = false
  description = "If set to true, all http traffic to public subnets will be allowed."
  type        = bool
}

variable "allow_https_traffic" {
  default     = true
  description = "If set to true, all https traffic to public subnets will be allowed."
  type        = bool
}

variable "allow_nat_traffic" {
  default     = false
  description = "If set to true, all traffic coming via the nat gateway IPs to public subnets will be allowed within the VPC."
  type        = bool
}

variable "allowed_external_cidr_blocks" {
  default     = []
  description = "Whitelist of external CIDR blocks that are allowed to access public subnets."
  type        = list(any)
}

variable "availability_zone_count" {
  default     = 2
  description = "Number of availability zones to span the network."
  type        = number
}

variable "dns_domain" {
  default     = "bonial.lan"
  description = "The DNS domain to use as a base for internal zones."
  type        = string
}

variable "env" {
  default     = "prod"
  description = "The environment of the VPC (e.g. prod). This will show up in internal DNS entries and tags for resources associated with the network."
  type        = string
}

variable "individual_private_subnet_tags" {
  default     = []
  description = "Individual tags that should be added to certain private subnets. This is list of maps of tags where the list index corresponds to the n-th subnet and allows tagging individual subnets differently."
  type        = list(map(string))
}

variable "network_name" {
  default     = "generic"
  description = "The name of the network. This will show up in internal DNS entries and tags for resources associated with the network."
  type        = string
}

variable "private_subnet_count" {
  default     = 2
  description = "Number of private subnets to create."
  type        = number
}

variable "subnet_size" {
  default     = 20
  description = "The CIDR block size of each subnet, e.g. 20 for /20 subnets."
  type        = number
}

variable "vpc_size" {
  default     = 16
  description = "The CIDR block size of the VPC, e.g. 16 for a /16 VPC."
  type        = number
}

output "internal_dns_zone_ids" {
  value       = aws_route53_zone.internal.*.zone_id
  description = "A list of the IDs of the internal DNS zones of the network."
}

output "internal_dns_zone_name" {
  value       = aws_route53_zone.internal.*.name
  description = "A list of the names of the internal DNS zones of the network."
}

output "nat_gateway_ips" {
  value       = aws_eip.nat.*.public_ip
  description = "A list of the IP addresses of the NAT gateways of the VPC."
}

output "public_routing_table_id" {
  value       = aws_route_table.public.id
  description = "The ID of the public routing table."
}

output "public_subnet_cidr_blocks" {
  value       = aws_subnet.public.*.cidr_block
  description = "A list of the CIDR blocks of the public subnets."
}

output "public_subnet_ids" {
  value       = aws_subnet.public.*.id
  description = "A list of the IDs of the public subnets."
}

output "private_routing_table_ids" {
  value       = aws_route_table.private.*.id
  description = "A list of the IDs of the private routing tables."
}

output "private_subnet_cidr_blocks" {
  value       = aws_subnet.private.*.cidr_block
  description = "A list of the CIDR blocks of the private subnets."
}

output "private_subnet_ids" {
  value       = aws_subnet.private.*.id
  description = "A list of the IDs of the private subnets."
}

output "vpc_cidr_block" {
  value       = aws_vpc.vpc.cidr_block
  description = "The CIDR block of the VPC."
}

output "vpc_id" {
  value       = aws_vpc.vpc.id
  description = "The ID of the VPC."
}

output "vpc_name" {
  value       = local.vpc_name
  description = "The full name of the VPC including network, env and brand."
}

# We only need one global public routing table for all public subnets in the
# VPC.
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.vpc.id

  tags = {
    Name        = "rt.public.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# One internet gateway route for all public subnets.
resource "aws_route" "internet_gateway" {
  route_table_id         = aws_route_table.public.id
  destination_cidr_block = "0.0.0.0/0"
  gateway_id             = aws_internet_gateway.internet_gateway.id
}

# Associate public subnets with the public routing table.
resource "aws_route_table_association" "public" {
  count = var.availability_zone_count

  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

# We need one private routing table per availability zone. All private subnets
# within an availability zone will be associated with this routing table.
resource "aws_route_table" "private" {
  count = var.availability_zone_count

  vpc_id = aws_vpc.vpc.id

  tags = {
    Name        = "rt.private.${local.vpc_azs[count.index]}.${local.vpc_name}"
    Application = "network-stack"
    VPC         = local.vpc_name
    Environment = var.env
    Managed     = "terraform"
  }
}

# One nat gateway route per availability zone.
resource "aws_route" "nat_gateway" {
  count = var.availability_zone_count

  route_table_id         = aws_route_table.private[count.index].id
  destination_cidr_block = "0.0.0.0/0"
  nat_gateway_id         = aws_nat_gateway.nat_gateway[count.index].id
}

# Associate private subnets with the private routing table in their
# availability zone.
resource "aws_route_table_association" "private" {
  count = var.private_subnet_count

  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id
}

# We create one public subnet per availability zone. The public subnets
# allocate the first `var.availability_zone_count` CIDR blocks in the VPC.
resource "aws_subnet" "public" {
  count = var.availability_zone_count

  vpc_id = aws_vpc.vpc.id
  cidr_block = cidrsubnet(
    aws_vpc.vpc.cidr_block,
    var.subnet_size - var.vpc_size,
    count.index,
  )
  availability_zone = local.vpc_azs[count.index]

  # Do not automatically assign public ips to EC2 instances launched in public
  # subnets.
  map_public_ip_on_launch = false

  # We have to manage tags for public subnets manually because kubernetes
  # requires public subnets to be tagged with `kubernetes.io/cluster/<cluster-id>`
  # in order to be able to automatically create ELBs in these subnets. However,
  # unlike private subnets, these tags are not added automatically by EKS so we
  # have to manage them ourselves for now. This is painful and should be automated.
  tags = merge(
    var.additional_public_subnet_tags,
    {
      "Name"                   = "public${count.index + 1}.${local.vpc_azs[count.index]}.${local.vpc_name}"
      "Application"            = "network-stack"
      "VPC"                    = local.vpc_name
      "Environment"            = var.env
      "Managed"                = "terraform"
      "kubernetes.io/role/elb" = "1"
    },
  )
}

# We create a configurable number of private subnets and spread them across
# the availability zones. Given we configured 3 private subnets and 2
# availability zones the spreading is done like this:
#
#   Subnet 1 => AZ 1
#   Subnet 2 => AZ 2
#   Subnet 3 => AZ 1
#
# The private subnets allocate `var.private_subnet_count` CIDR blocks in the
# VPC. The blocks start `var.availability_zone_count` blocks after the base
# address of the VPC (first `var.availability_zone_count` blocks are allocated
# by the public subnets).
resource "aws_subnet" "private" {
  count = var.private_subnet_count

  vpc_id = aws_vpc.vpc.id
  cidr_block = cidrsubnet(
    aws_vpc.vpc.cidr_block,
    var.subnet_size - var.vpc_size,
    count.index + var.availability_zone_count,
  )
  availability_zone = local.vpc_azs[count.index % var.availability_zone_count]

  # Do not automatically assign public ips to EC2 instances launched in
  # private subnets.
  map_public_ip_on_launch = false

  tags = merge(
    var.additional_private_subnet_tags,
    try(var.individual_private_subnet_tags[count.index], {}),
    {
      Name                              = "private${count.index + 1}.${local.vpc_azs[count.index % var.availability_zone_count]}.${local.vpc_name}"
      Application                       = "network-stack"
      VPC                               = local.vpc_name
      Environment                       = var.env
      Managed                           = "terraform"
      "kubernetes.io/role/internal-elb" = "1"
    }
  )
}

terraform {
  required_version = "~> 1.2.2"
}
locals {
  # The VPC name is a DNS friendly name composed of the name of the network,
  # the environment and brand. It will be used in internal DNS names and as a
  # suffix for resource names.
  vpc_name = "${var.network_name}.${var.env}.${var.account}"

  # These locals for vpc_azs and vpc_region are just here to have shorter names
  # for frequently used values.
  vpc_azs = data.aws_availability_zones.available.names

  vpc_region = data.aws_region.current.name
}

data "aws_availability_zones" "available" {
}

data "aws_region" "current" {
}

# This will create a VPC with base address `var.vpc_address` and size
# `var.vpc_size`. The VPC will house `var.availability_zone_count` public
# subnets and `var.private_subnet_count` private subnets of size
# `var.subnet_size`.
resource "aws_vpc" "vpc" {
  cidr_block           = "${var.vpc_address}/${var.vpc_size}"
  enable_dns_hostnames = true
  enable_dns_support   = true
  instance_tenancy     = "default"

  tags = {
    Name        = local.vpc_name
    Application = "network-stack"
    Environment = var.env
    Managed     = "terraform"
  }

  lifecycle {
    # Some resources may add additional tags to the VPC for discovery purposes
    # (e.g. EKS). To avoid managing these, we just ignore changes to the
    # inital tags.
    ignore_changes = [tags]
  }
}

module "default-security-group" {
  source = "../default-security-group"

  vpc_id = aws_vpc.vpc.id
}
