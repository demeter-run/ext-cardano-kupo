# resource "kubernetes_manifest" "instance_monitor" {
#   manifest = {
#     apiVersion = "monitoring.coreos.com/v1"
#     kind       = "PodMonitor"
#     metadata = {
#       labels = {
#         "app.kubernetes.io/component" = "o11y"
#         "app.kubernetes.io/part-of"   = "demeter"
#       }
#       name      = local.instance
#       namespace = var.namespace
#     }
#     spec = {
#       selector = {
#         matchLabels = {
#           "demeter.run/instance"        = var.instance_name
#           "demeter.run/salt"            = var.salt
#           "cardano.demeter.run/network" = var.network
#         }
#       }
#       podMetricsEndpoints = [
#         {
#           port = "server",
#           path = "/metrics"
#         }
#       ]
#     }
#   }
# }
