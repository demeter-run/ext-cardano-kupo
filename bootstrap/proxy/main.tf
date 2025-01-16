locals {
  name = var.name
  role = "proxy"

  prometheus_port = 9187
  prometheus_addr = "0.0.0.0:${local.prometheus_port}"
  proxy_port      = 8080
  proxy_addr      = "0.0.0.0:${local.proxy_port}"
  proxy_labels    = var.environment != null ? { role = "${local.role}-${var.environment}" } : { role = local.role }
}

variable "name" {
  type    = string
  default = "proxy"
}

// blue - green
variable "environment" {
  default = null
}

variable "namespace" {
  type = string
}

variable "replicas" {
  type    = number
  default = 1
}

variable "proxy_image_tag" {
  type = string
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
      cpu : "50m",
      memory : "250Mi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
    }
  }
}

variable "kupo_port" {
  type    = number
  default = 1442
}


variable "extension_name" {
  type = string
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview"]
}

variable "versions" {
  type    = list(string)
  default = ["2"]
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}

variable "cloud_provider" {
  type    = string
  default = "aws"
}

variable "healthcheck_port" {
  type    = number
  default = null
}

variable "cluster_issuer" {
  type    = string
  default = "letsencrypt"
}

variable "tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Equal"
      value    = "general-purpose"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Equal"
      value    = "x86"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Equal"
      value    = "consistent"
    }
  ]
}
