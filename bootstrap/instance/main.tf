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

variable "defer_indexes" {
  default = false
}

variable "tolerations" {
  description = "List of tolerations for the node"
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
      value    = "disk-intensive"
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
