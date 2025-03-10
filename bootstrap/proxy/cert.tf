locals {
  cert_secret_name = (
    var.cert_secret_name != null
    ? var.cert_secret_name
    : (
      var.environment != null
      ? "${var.extension_name}-${var.environment}-proxy-wildcard-tls"
      : "${var.extension_name}-proxy-wildcard-tls"
    )
  )
}

resource "kubernetes_manifest" "certificate_cluster_wildcard_tls" {
  manifest = {
    "apiVersion" = "cert-manager.io/v1"
    "kind"       = "Certificate"
    "metadata" = {
      "name"      = local.cert_secret_name
      "namespace" = var.namespace
    }
    "spec" = {
      "dnsNames" = var.dns_names

      "issuerRef" = {
        "kind" = "ClusterIssuer"
        "name" = var.cluster_issuer
      }
      "secretName" = local.cert_secret_name
    }
  }
}

