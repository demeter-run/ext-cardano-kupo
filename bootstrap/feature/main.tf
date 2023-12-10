variable "namespace" {
  default = "ftr-kupo-v1"
}

variable "operator_image_tag" {}


variable "scrape_interval" {
  description = "the inverval for polling metrics data (in seconds)"
  default     = "30"
}

variable "per_min_dcus" {
  default = {
    "mainnet" : 36,
    "default" : 16,
  }
}

variable "track_dcu_usage" {
  default = "true"
}

variable "api_key_salt" {
  type = string
}

variable "ingress_class" {
  type = string
}

variable "dns_zone" {
  default = "demeter.run"
}

output "namespace" {
  value = var.namespace
}

