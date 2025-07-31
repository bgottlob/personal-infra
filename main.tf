provider "sops" {}

data "sops_file" "secrets" {
  source_file = "secrets.yaml"
}

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
      version = "2.34.1"
    }

    sops = {
      source = "carlpett/sops"
      version = "1.1.1"
    }
  }
}

provider "linode" {
  token = data.sops_file.secrets.data["linode.token"]
}

resource "linode_lke_cluster" "personal" {
  label = "bgottlob-personal"
  k8s_version = "1.33"
  region = "us-east"

  control_plane {
    high_availability = false
  }

  pool {
    type = "g6-standard-1"
    count = 2
  }
}
