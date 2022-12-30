# nginx Ingress controller
resource "helm_release" "nginx_ingress" {
  depends_on = [linode_lke_cluster.personal]
  name = "nginx-ingress-controller"
  repository = "https://kubernetes.github.io/ingress-nginx"
  chart = "ingress-nginx"
}

data "kubernetes_service" "nginx_ingress" {
  depends_on = [helm_release.nginx_ingress]
  metadata {
    namespace = "default"
    name = "nginx-ingress-controller-ingress-nginx-controller"
  }
}

resource "kubernetes_namespace" "cert_manager" {
  depends_on = [linode_lke_cluster.personal]
  metadata {
    name = "cert-manager"
  }
}

resource "helm_release" "cert_manager" {
  depends_on = [linode_lke_cluster.personal]
  name = "cert-manager"
  repository = "https://charts.jetstack.io"
  chart = "cert-manager"
  namespace = "cert-manager"

  set {
    name = "installCRDs"
    value = "true"
  }
}

locals {
  ingress_external_ip = data.kubernetes_service.nginx_ingress.status.0.load_balancer.0.ingress.0.ip
}
