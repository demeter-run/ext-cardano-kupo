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
          image = "ghcr.io/demeter-run/ext-cardano-kupo-operator:${var.operator_image_tag}"
          name  = "main"

          env {
            name  = "ADDR"
            value = "0.0.0.0:9946"
          }

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "PROMETHEUS_URL"
            value = "http://prometheus-operated.demeter-system.svc.cluster.local:9090/api/v1"
          }

          env {
            name  = "METRICS_DELAY"
            value = var.metrics_delay
          }

          env {
            name  = "DCU_PER_REQUEST_MAINNET"
            value = var.per_request_dcus["mainnet"]
          }

          env {
            name  = "DCU_PER_REQUEST_PREPROD"
            value = var.per_request_dcus["default"]
          }

          env {
            name  = "DCU_PER_REQUEST_PREVIEW"
            value = var.per_request_dcus["default"]
          }

          env {
            name  = "DCU_PER_REQUEST_SANCHONET"
            value = var.per_request_dcus["default"]
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
            name  = "EXTENSION_SUBDOMAIN"
            value = var.extension_subdomain
          }

          env {
            name  = "DNS_ZONE"
            value = var.dns_zone
          }

          resources {
            limits = {
              memory = "512Mi"
            }
            requests = {
              cpu    = "50m"
              memory = "512Mi"
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
          operator = "Equal"
          value    = "x86"
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
