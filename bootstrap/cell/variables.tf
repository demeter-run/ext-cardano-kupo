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

// Instances
variable "instances" {
  type = map(object({
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
}
