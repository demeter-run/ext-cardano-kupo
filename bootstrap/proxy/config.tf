// each replica enforces these limits on its own, so by default the tier quota
// is split across replicas
locals {
  config_map_name = var.environment != null ? "${var.environment}-proxy-config" : "proxy-config"
  quota_divisor   = var.split_quota_across_replicas ? max(var.replicas, 1) : 1

  tiers = [
    {
      "name" = "0",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(5 * 60 / local.quota_divisor)
        },
        {
          "interval" = "1d",
          "limit"    = floor(430000 / local.quota_divisor)
        }
      ]
    },
    {
      "name" = "1",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(20 * 60 / local.quota_divisor)
        },
        {
          "interval" = "1d",
          "limit"    = floor(1700000 / local.quota_divisor)
        }
      ]
    },
    {
      "name" = "2",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(100 * 60 / local.quota_divisor)
        },
        {
          "interval" = "1d",
          "limit"    = floor(8600000 / local.quota_divisor)
        }
      ]
    },
    {
      "name" = "3",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(300 * 60 / local.quota_divisor)
        },
        {
          "interval" = "1d",
          "limit"    = floor(26000000 / local.quota_divisor)
        }
      ]
    }
  ]
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = local.config_map_name
  }

  data = {
    "tiers.toml" = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
  }
}
