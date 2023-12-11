resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = "operator"
    labels = {
      role = "operator"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        role = "operator"
      }
    }

    template {
      metadata {
        labels = {
          role = "operator"
        }
      }

      spec {
        container {
          image   = "ghcr.io/demeter-run/ext-cardano-kupo-operator:${var.operator_image_tag}"
          name    = "main"
          command = ["npm", "run", "start"]

          env {
            name  = "PORT"
            value = 9946
          }

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "PROMETHEUS_QUERY_ENDPOINT"
            value = "http://prometheus-operated.default.svc.cluster.local:9090"
          }

          env {
            name  = "SCRAPE_INTERVAL_S"
            value = var.scrape_interval
          }

          env {
            name  = "PER_MIN_DCUS_MAINNET"
            value = var.per_min_dcus["mainnet"]
          }

          env {
            name  = "PER_MIN_DCUS_DEFAULT"
            value = var.per_min_dcus["default"]
          }

          env {
            name  = "TRACK_DCU_USAGE"
            value = var.track_dcu_usage
          }

          env {
            name  = "API_KEY_SALT"
            value = var.api_key_salt
          }

          env {
            name  = "NAMESPACE"
            value = var.namespace
          }

          env {
            name  = "INGRESS_CLASS"
            value = var.ingress_class
          }

          env {
            name = "DNS_ZONE"
            value = var.dns_zone
          }

          resources {
            limits = {
              memory = "256Mi"
            }
            requests = {
              cpu    = "50m"
              memory = "256Mi"
            }
          }

          port {
            name           = "metrics"
            container_port = 9946
            protocol       = "TCP"
          }
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Equal"
          value    = "general-purpose"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Exists"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/availability-sla"
          operator = "Equal"
          value    = "consistent"
        }
      }
    }
  }
}

