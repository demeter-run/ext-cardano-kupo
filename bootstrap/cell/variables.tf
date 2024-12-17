variable "namespace" {
  type = string
}

variable "salt" {
  type        = string
  description = "Salt used to identify all components as part of the cell. Should be unique between cells."
}

// PVC
variable "volume_name" {
  type = string
}

variable "storage_size" {
  type = string
}

variable "storage_class_name" {
  type = string
}

variable "access_mode" {
  type = string
}

// Instances
variable "instances" {
  type = map(object({
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
    tolerations = optional(list(object({
      effect   = string
      key      = string
      operator = string
      value    = optional(string)
    })))
  }))
}
