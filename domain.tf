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
  target = "mail.protonmail.ch"
  priority = 10
}

resource "linode_domain_record" "protonmail_mx_2" {
  domain_id = linode_domain.bgottlob.id
  record_type = "MX"
  target = "mailsec.protonmail.ch"
  priority = 20
}

resource "linode_domain_record" "protonmail_spf" {
  domain_id = linode_domain.bgottlob.id
  record_type = "TXT"
  target = "v=spf1 include:_spf.protonmail.ch ~all"
}

resource "linode_domain_record" "protonmail_dkim_1" {
  domain_id = linode_domain.bgottlob.id
  name = "protonmail._domainkey"
  record_type = "CNAME"
  target = "protonmail.domainkey.drwtk2chdc5drbmwi3ywgiz6re3m3qwr54k5m344witsbpfsfthea.domains.proton.ch"
}

resource "linode_domain_record" "protonmail_dkim_2" {
  domain_id = linode_domain.bgottlob.id
  name = "protonmail2._domainkey"
  record_type = "CNAME"
  target = "protonmail2.domainkey.drwtk2chdc5drbmwi3ywgiz6re3m3qwr54k5m344witsbpfsfthea.domains.proton.ch"
}

resource "linode_domain_record" "protonmail_dkim_3" {
  domain_id = linode_domain.bgottlob.id
  name = "protonmail3._domainkey"
  record_type = "CNAME"
  target = "protonmail3.domainkey.drwtk2chdc5drbmwi3ywgiz6re3m3qwr54k5m344witsbpfsfthea.domains.proton.ch"
}

resource "linode_domain_record" "protonmail_dmarc" {
  domain_id = linode_domain.bgottlob.id
  name = "_dmarc"
  record_type = "TXT"
  target = "v=DMARC1; p=quarantine"
}
