data "kubernetes_service" "nginx_ingress" {
  metadata {
    namespace = "default"
    name = "nginx-ingress-controller-ingress-nginx-controller"
  }
}

locals {
  ingress_external_ip = data.kubernetes_service.nginx_ingress.status.0.load_balancer.0.ingress.0.ip
}
