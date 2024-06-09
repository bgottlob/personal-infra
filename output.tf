output "k8s_admin_token" {
  sensitive = true
  value = [for user in yamldecode(
    base64decode(linode_lke_cluster.personal.kubeconfig)
  ).users : user.user.token if user.name == "lke${linode_lke_cluster.personal.id}-admin"][0]
}

output "registry_access_bucket_keys" {
  sensitive = true
  value = {
    access_key = linode_object_storage_key.registry.access_key
    secret_key = linode_object_storage_key.registry.secret_key
  }
}

output "velero_access_bucket_keys" {
  sensitive = true
  value = {
    access_key = linode_object_storage_key.velero.access_key
    secret_key = linode_object_storage_key.velero.secret_key
  }
}
