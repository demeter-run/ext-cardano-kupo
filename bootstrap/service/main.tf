variable "namespace" {}
variable "network" {
  type = string
}
variable "salt" {}
variable "prune" {
  type = bool
}
variable "instance_name" {
  type = string
}

locals {
  instance     = "${var.instance_name}-${var.salt}"
  service_name = var.prune ? "kupo-${var.network}-pruned" : "kupo-${var.network}"
}

resource "kubernetes_service_v1" "well_known_service" {
  metadata {
    name      = local.service_name
    namespace = var.namespace
  }

  spec {
    port {
      name     = "http"
      protocol = "TCP"
      port     = 1442
    }

    selector = {
      "cardano.demeter.run/network" = var.network
      "cardano.demeter.run/kupo-pruned" = var.prune ? "true" : "false"
    }

    type = "ClusterIP"
  }
}
