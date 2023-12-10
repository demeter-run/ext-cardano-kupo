locals {
  rate_limiting_tiers = {
    "kupo-tier-0" = {
      "minute" = 10
    },
    "kupo-tier-1" = {
      "minute" = 100
    },
    "kupo-tier-2" = {
      "minute" = 1000
    },
  }
}

resource "kubernetes_manifest" "rate_limiting_cluster_plugin" {
  for_each = local.rate_limiting_tiers
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongClusterPlugin"
    "metadata" = {
      "name" = "rate-limiting-${each.key}"
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
      "labels": {
        "global": "false"
      }
    }
    "config" = {
      "minute" = each.value.minute
      "policy" = "local"
    }
    "plugin" = "rate-limiting"
  }
}