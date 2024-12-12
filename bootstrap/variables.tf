variable "namespace" {
  type    = string
  default = "ftr-kupo-v1"
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview"]
}

variable "cloud_provider" {
  type    = string
  default = "aws"
}

variable "cluster_issuer" {
  type    = string
  default = "letsencrypt"
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

variable "operator_resources" {
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

variable "storage_class" {
  type    = string
  default = "nvme"
}

// Cells
variable "cells" {
  type = map(object({
    pvc = object({
      volume_name        = optional(string)
      storage_size       = string
      storage_class_name = string
      access_mode        = string
    })
    instances = map(object({
      image_tag     = string
      network       = string
      pruned        = bool
      defer_indexes = optional(bool)
      n2n_endpoint  = string
      suffix        = optional(string)
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
