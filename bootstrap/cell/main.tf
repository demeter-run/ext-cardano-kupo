// Each cell of the kupo extension containes 1 PVC and an amount of indexers
// (commonly 3, one per network).
locals {
  pvc_name = "db-local-pv-${var.salt}"
}

module "kupo_pvc" {
  source       = "../pvc"
  namespace    = var.namespace
  volume_name  = var.volume_name
  storage_size = var.storage_size
  name         = local.pvc_name
}

module "kupo_instances" {
  source   = "../instance"
  for_each = var.instances

  namespace       = var.namespace
  image_tag       = each.value.image_tag
  network         = each.value.network
  pruned          = each.value.pruned
  defer_indexes   = coalesce(each.value.defer_indexes, false)
  n2n_endpoint    = each.value.n2n_endpoint
  db_volume_claim = local.pvc_name
  suffix          = coalesce(each.value.suffix, var.salt)

  resources = coalesce(each.value.resources, {
    limits = {
      cpu    = "1",
      memory = "1Gi"
    }
    requests = {
      cpu    = "500m",
      memory = "1Gi"
    }
  })
}
