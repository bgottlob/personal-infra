# The bgottlob Domain in Linode
resource "linode_domain" "bgottlob" {
  domain = "bgottlob.com"
  type = "master"
  soa_email = "login+dns@bgottlob.com"
}

# Subdomains used in an Ingress to the k8s cluster
resource "linode_domain_record" "bgottlob_blog" {
  depends_on = [linode_lke_cluster.personal]
  domain_id = linode_domain.bgottlob.id
  record_type = "A"
  name = "blog"
  target = local.ingress_external_ip
}

resource "linode_domain_record" "bgottlob_wallabag" {
  depends_on = [linode_lke_cluster.personal]
  domain_id = linode_domain.bgottlob.id
  record_type = "A"
  name = "wallabag"
  target = local.ingress_external_ip
}

resource "linode_domain_record" "bgottlob_miniflux" {
  depends_on = [linode_lke_cluster.personal]
  domain_id = linode_domain.bgottlob.id
  record_type = "A"
  target = local.ingress_external_ip
  name = "miniflux"
}

resource "linode_domain_record" "bgottlob_registry" {
  depends_on = [linode_lke_cluster.personal]
  domain_id = linode_domain.bgottlob.id
  record_type = "A"
  name = "registry"
  target = local.ingress_external_ip
}
