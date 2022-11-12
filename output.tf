output "k8s_admin_token" {
  sensitive = true
  value = [for user in yamldecode(
    base64decode(linode_lke_cluster.personal.kubeconfig)
  ).users : user.user.token if user.name == "lke${linode_lke_cluster.personal.id}-admin"][0]
}
