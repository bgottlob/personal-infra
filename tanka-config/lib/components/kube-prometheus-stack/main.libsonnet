local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

local util = import 'util/main.libsonnet';

{
  all(
    namespace = 'prometheus',
    grafanaAdminUsername,
    grafanaAdminPassword,
  ): {
    namespace: k.core.v1.namespace.new(namespace),

    chart: helm.template('kube-prometheus-stack', './charts/kube-prometheus-stack', {
      namespace: namespace,
      includeCrds: true,
      values: {
        grafana: {
          adminUser: grafanaAdminUsername,
          adminPassword: grafanaAdminPassword,
        }
      }
    }),
  },
}
