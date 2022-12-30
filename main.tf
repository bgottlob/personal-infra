terraform {
  backend "s3" {
    bucket = "bgottlob-terraform-state"
    key = "personal-cloud.tfstate"
    region = "us-east-1"
    endpoint = "us-east-1.linodeobjects.com"
    skip_credentials_validation = true
  }

  required_providers {
    linode = {
      source  = "linode/linode"
      version = "1.29.4"
    }
  }
}

resource "linode_lke_cluster" "personal" {
  label = "bgottlob-personal"
  k8s_version = "1.24"
  region = "us-east"

  control_plane {
    high_availability = false
  }

  pool {
    type = "g6-standard-1"
    count = 1
  }
}
