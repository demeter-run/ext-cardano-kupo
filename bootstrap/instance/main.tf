variable "namespace" {
  default = "ftr-kupo-v1"
}

variable "network" {
  type = string
}

variable "pruned" {
  default = false
}

variable "image_tag" {
  type = string
}

variable "n2n_endpoint" {
  type = string
}

variable "db_volume_claim" {
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
    limits = {
      cpu    = "1",
      memory = "1Gi"
    }
    requests = {
      cpu    = "500m",
      memory = "1Gi"
    }
  }
}

variable "suffix" {
  default = ""
}

