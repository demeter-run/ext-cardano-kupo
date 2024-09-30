variable "namespace" {
  description = "The namespace where the resources will be created."
}

variable "volume_name" {
  description = "The name of the volume. If not specified, the volume will be dynamically provisioned."
  type        = string
  default     = null
}

variable "name" {
  description = "The name of the PersistentVolumeClaim (PVC)."
}

variable "storage_size" {
  description = "The size of the volume."
}

variable "storage_class_name" {
  description = "The name of the storage class to use."
  default     = "nvme"
}

variable "access_mode" {
  description = "The access mode for the volume."
  type        = string
  default     = "ReadWriteMany"
}

resource "kubernetes_persistent_volume_claim" "shared_disk" {
  wait_until_bound = false

  metadata {
    name      = var.name
    namespace = var.namespace
  }

  spec {
    access_modes = [var.access_mode]

    resources {
      requests = {
        storage = var.storage_size
      }
    }

    storage_class_name = var.storage_class_name
    volume_name        = var.volume_name != null ? var.volume_name : null
  }
}
