locals {
  name = var.name
  role = "proxy-${var.network}"

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

variable "network" {
  type = string
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

variable "extra_annotations" {
  description = "Extra annotations to add to the proxy services"
  type        = map(string)
  default     = {}
}

variable "dns_names" {
  description = "URL that will hit this proxies, used to create TLS certificates"
  type        = list(string)
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

variable "cert_secret_name" {
  type    = string
  default = null
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
