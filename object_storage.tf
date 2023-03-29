data "linode_object_storage_cluster" "primary" {
  id = "us-east-1"
}

resource "linode_object_storage_bucket" "public" {
  cluster = data.linode_object_storage_cluster.primary.id
  label = "bgottlob-public"
  acl = "public-read"
}

resource "linode_object_storage_key" "public" {
  label = "public-access"
  bucket_access {
    cluster = data.linode_object_storage_cluster.primary.id
    bucket_name = linode_object_storage_bucket.public.label
    permissions = "read_write"
  }
}

resource "linode_object_storage_bucket" "registry" {
  cluster = data.linode_object_storage_cluster.primary.id
  label = "bgottlob-registry"
  acl = "private"
}

resource "linode_object_storage_key" "registry" {
  label = "registry-access"
  bucket_access {
    cluster = data.linode_object_storage_cluster.primary.id
    bucket_name = linode_object_storage_bucket.registry.label
    permissions = "read_write"
  }
}
