resource "kubernetes_default_service_account_v1" "default" {
  metadata {
    namespace = var.namespace
  }

  automount_service_account_token = false
}

resource "kubernetes_service_account_v1" "operator" {
  metadata {
    namespace = var.namespace
    name      = "operator"
  }

  automount_service_account_token = true
}

resource "kubernetes_service_account_v1" "proxy" {
  metadata {
    namespace = var.namespace
    name      = "proxy"
  }

  automount_service_account_token = true
}
