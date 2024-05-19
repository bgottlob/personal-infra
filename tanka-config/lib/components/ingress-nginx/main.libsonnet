local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

{
  all(namespace='default'):
    helm.template('nginx-ingress-controller', './charts/ingress-nginx', {
      namespace: namespace,
    }),
}
