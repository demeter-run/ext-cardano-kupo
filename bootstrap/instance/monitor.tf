resource "kubernetes_manifest" "instance_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = local.instance_name
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          "demeter.run/instance"            = local.instance_name
          "cardano.demeter.run/network"     = var.network
          "cardano.demeter.run/kupo-pruned" = var.pruned ? "true" : "false"
        }
      }
      podMetricsEndpoints = [
        {
          port = "http",
          path = "/health"
        }
      ]
    }
  }
}
