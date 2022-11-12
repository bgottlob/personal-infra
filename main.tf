terraform {
  required_providers {
    linode = {
      source  = "linode/linode"
      version = "1.29.4"
    }
  }
}

resource "linode_lke_cluster" "personal" {
  label = "bgottlob-personal"
  k8s_version = "1.23"
  region = "us-east"

  control_plane {
    high_availability = false
  }

  pool {
    type = "g6-standard-1"
    count = 1
  }
}
