variable "namespace" {
  default = "ftr-kupo-v1"
}

variable "operator_image_tag" {}


variable "metrics_delay" {
  description = "the inverval for polling metrics data (in seconds)"
  default     = "60"
}

variable "per_min_dcus" {
  default = {
    "mainnet" : 36,
    "default" : 16,
  }
}

variable "per_request_dcus" {
  default = {
    "mainnet" : 10,
    "default" : 5,
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

variable "extension_subdomain" {
  type = string
}

variable "dns_zone" {
  default = "demeter.run"
}

output "namespace" {
  value = var.namespace
}

variable "resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits : {
      cpu : "300m",
      memory : "1Gi"
    }
    requests : {
      cpu : "150m",
      memory : "250Mi"
    }
  }
}
