resource "kubernetes_manifest" "http_route" {
  for_each = var.networks

  manifest = {
    "apiVersion" = "gateway.networking.k8s.io/v1"
    "kind"       = "HTTPRoute"
    "metadata" = {
      "annotations" = {
        "konghq.com/plugins" = "kupo-key-to-header, kupo-auth, acl-kupo-${each.key}"
      }
      "labels" = {
        "demeter.run/kind"    = "http-route"
        "demeter.run/tenancy" = "proxy"
      }
      "name"      = "kupo-${each.key}"
      "namespace" = var.namespace
    }
    "spec" = {
      "hostnames" = [
        "${each.key}.kupo-v1.demeter.run",
        "*.${each.key}.kupo-v1.demeter.run"
      ]
      "parentRefs" = [
        {
          "group"     = "gateway.networking.k8s.io"
          "kind"      = "Gateway"
          "name"      = "kupo-v1"
          "namespace" = "ftr-kupo-v1"
        },
      ]
      "rules" = [
        {
          "backendRefs" = [
            {
              "group"  = ""
              "kind"   = "Service"
              "name"   = "kupo-${each.key}-pruned"
              "port"   = 1442
              "weight" = 1
            },
          ]
          "matches" = [
            {
              "path" = {
                "type"  = "PathPrefix"
                "value" = "/"
              }
            },
          ]
        },
      ]
    }
  }
}

resource "kubernetes_manifest" "kong_auth_plugin" {
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongPlugin"
    "metadata" = {
      "name"      = "kupo-auth"
      "namespace" = var.namespace
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
    }
    "config" = {
      "key_names" = [
        "dmtr-api-key"
      ]
    }
    "plugin" = "key-auth"
  }
}

resource "kubernetes_manifest" "kong_key_to_header_plugin" {
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongPlugin"
    "metadata" = {
      "name"      = "kupo-key-to-header"
      "namespace" = var.namespace
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
    }
    config   = {}
    "plugin" = "key-to-header"
  }
}

