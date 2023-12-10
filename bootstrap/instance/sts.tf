resource "kubernetes_stateful_set_v1" "utxorpc" {
  wait_for_rollout = "false"

  metadata {
    name      = local.instance
    namespace = var.namespace
    labels = {
      "demeter.run/kind"            = "UtxoRpcInstance"
      "demeter.run/release"         = var.release
      "cardano.demeter.run/network" = var.network
      "demeter.run/instance"        = local.instance
    }
  }
  spec {
    replicas     = var.replicas
    service_name = "utxorpc"
    volume_claim_template {
      metadata {
        name = "data"
      }
      spec {
        access_modes = ["ReadWriteOnce"]
        resources {
          requests = {
            storage = var.resources.storage.size
          }
        }
        storage_class_name = var.resources.storage.class
      }
    }
    selector {
      match_labels = {
        "demeter.run/instance"        = local.instance
        "demeter.run/release"         = var.release
        "cardano.demeter.run/network" = var.network
      }
    }
    template {
      metadata {
        labels = {
          "demeter.run/instance"        = local.instance
          "demeter.run/release"         = var.release
          "cardano.demeter.run/network" = var.network
        }
      }
      spec {
        container {
          name  = local.instance
          image = "ghcr.io/txpipe/dolos:${var.dolos_version}"
          args = [
            "/etc/config/dolos.toml",
            "daemon"
          ]
          resources {
            limits   = var.resources.limits
            requests = var.resources.requests
          }

          port {
            name           = "grpc"
            container_port = 50051
            protocol       = "TCP"
          }

          port {
            name           = "ouroboros"
            container_port = 30013
            protocol       = "TCP"
          }

          volume_mount {
            name       = "data"
            mount_path = "/var/data"
          }
          volume_mount {
            name       = "config"
            mount_path = "/etc/config"
          }

        }

        volume {
          name = "config"
          config_map {
            name = "configs-${var.network}"
          }
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Exists"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Equal"
          value    = "x86"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/availability-sla"
          operator = "Equal"
          value    = "best-effort"
        }
      }
    }
  }
}

