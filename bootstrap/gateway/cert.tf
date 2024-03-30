

locals {
  by_network = flatten([
    for network in var.networks : [
      for version in var.versions : "*.${network}-${version}.kupo-m1.${var.dns_zone}"
    ]
  ])

  # Add the extra URL to the list of generated URLs
  dns_names = concat(local.by_network, ["*.kupo-m1.${var.dns_zone}"])
  cert_secret_name = "kupo-m1-wildcard-tls"
}

resource "kubernetes_manifest" "certificate_cluster_wildcard_tls_m1" {
  manifest = {
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata" = {
      "name"      = "kupo-m1-wildcard-tls"
      "namespace" = var.namespace
    }
    "spec" = {
      "dnsNames" = local.dns_names

      "issuerRef" = {
        "kind" = "ClusterIssuer"
        "name" = "letsencrypt"
      }
      "secretName" = local.cert_secret_name
    }
  }
}
