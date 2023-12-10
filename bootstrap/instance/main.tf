variable "namespace" {}

variable "network" {
  type = string
}

variable "salt" {
  type = string
}

variable "instance_name" {
  type = string
}

variable "replicas" {
  default = 1
}

locals {
  instance = "${var.instance_name}-${var.salt}"
}

variable "operator_version" {
  type = string
}

variable "dolos_version" {
  type = string
  default = "v0.6.0"
}

variable "release" {
  default = "stable"
}

variable "resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
    storage = object({
      size  = string
      class = string
    })
  })
  default = {
    requests = {
      cpu    = "50m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "1000m"
      memory = "512Mi"
    }
    storage = {
      size  = "30Gi"
      class = "fast"
    }

  }
}

