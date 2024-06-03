# The bgottlob Domain in Linode
resource "linode_domain" "bgottlob" {
  domain = "bgottlob.com"
  type = "master"
  soa_email = "login+dns@bgottlob.com"
}
