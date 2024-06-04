variable "namespace" {
  type = string
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview"]
}

// Feature
variable "operator_image_tag" {
  type = string
}

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

// Proxies
variable "proxy_green_image_tag" {
  type = string
}

variable "proxy_green_replicas" {
  type    = number
  default = 1
}

variable "proxy_blue_image_tag" {
  type = string
}

variable "proxy_blue_replicas" {
  type    = number
  default = 1
}

variable "proxy_resources" {
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

// Cells
variable "cells" {
  type = map(object({
    pvc = object({
      volume_name  = string
      storage_size = string
    })
    instances = map(object({
      image_tag    = string
      network      = string
      pruned       = bool
      n2n_endpoint = string
      suffix       = optional(string)
      resources = optional(object({
        limits = object({
          cpu    = string
          memory = string
        })
        requests = object({
          cpu    = string
          memory = string
        })
      }))
    }))
  }))
}
