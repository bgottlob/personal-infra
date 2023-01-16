variable "s3_access_key" {
  type = string
  description = "Access key for Linode object storage"
  sensitive = true
}

variable "s3_secret_key" {
  type = string
  description = "Secret key for Linode object storage"
  sensitive = true
}
