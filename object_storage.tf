locals {
  object_storage_region = "us-east"
}

resource "linode_object_storage_bucket" "public" {
  region = local.object_storage_region
  label = "bgottlob-public"
  acl = "public-read"
}

resource "linode_object_storage_key" "public" {
  label = "public-access"
  bucket_access {
    region = local.object_storage_region
    bucket_name = linode_object_storage_bucket.public.label
    permissions = "read_write"
  }
}

resource "linode_object_storage_bucket" "db_backup" {
  region = local.object_storage_region
  label = "bgottlob-db-backup"
  acl = "private"
}

resource "linode_object_storage_key" "db_backup" {
  label = "db-backup-access"
  bucket_access {
    region = local.object_storage_region
    bucket_name = linode_object_storage_bucket.db_backup.label
    permissions = "read_write"
  }
}

resource "linode_object_storage_bucket" "registry" {
  region = local.object_storage_region
  label = "bgottlob-registry"
  acl = "private"
}

resource "linode_object_storage_key" "registry" {
  label = "registry-access"
  bucket_access {
    region = local.object_storage_region
    bucket_name = linode_object_storage_bucket.registry.label
    permissions = "read_write"
  }
}

resource "linode_object_storage_bucket" "velero" {
  region = local.object_storage_region
  label = "bgottlob-velero-backups"
  acl = "private"
}

resource "linode_object_storage_key" "velero" {
  label = "velero-backups-access"
  bucket_access {
    region = local.object_storage_region
    bucket_name = linode_object_storage_bucket.velero.label
    permissions = "read_write"
  }
}
