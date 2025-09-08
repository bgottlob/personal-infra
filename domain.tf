# The bgottlob Domain in Linode
resource "linode_domain" "bgottlob" {
  domain = "bgottlob.com"
  type = "master"
  soa_email = "login+dns@bgottlob.com"
}

# Records for Protonmail
resource "linode_domain_record" "protonmail_txt" {
  domain_id = linode_domain.bgottlob.id
  record_type = "TXT"
  target = "protonmail-verification=b167eb1b100d5d1b6a6a59165f5b336a77cbb235"
}

resource "linode_domain_record" "protonmail_mx_1" {
  domain_id = linode_domain.bgottlob.id
  record_type = "MX"
  target = "in1-smtp.messagingengine.com"
  priority = 10
}

resource "linode_domain_record" "protonmail_mx_2" {
  domain_id = linode_domain.bgottlob.id
  record_type = "MX"
  target = "in2-smtp.messagingengine.com"
  priority = 20
}

resource "linode_domain_record" "protonmail_spf" {
  domain_id = linode_domain.bgottlob.id
  record_type = "TXT"
  target = "v=spf1 include:spf.messagingengine.com ?all"
}

resource "linode_domain_record" "protonmail_dkim_1" {
  domain_id = linode_domain.bgottlob.id
  name = "fm1._domainkey"
  record_type = "CNAME"
  target = "fm1.bgottlob.com.dkim.fmhosted.com"
}

resource "linode_domain_record" "protonmail_dkim_2" {
  domain_id = linode_domain.bgottlob.id
  name = "fm2._domainkey"
  record_type = "CNAME"
  target = "fm2.bgottlob.com.dkim.fmhosted.com"
}

resource "linode_domain_record" "protonmail_dkim_3" {
  domain_id = linode_domain.bgottlob.id
  name = "fm3._domainkey"
  record_type = "CNAME"
  target = "fm3.bgottlob.com.dkim.fmhosted.com"
}

resource "linode_domain_record" "protonmail_dmarc" {
  domain_id = linode_domain.bgottlob.id
  name = "_dmarc"
  record_type = "TXT"
  target = "v=DMARC1; p=quarantine"
}
