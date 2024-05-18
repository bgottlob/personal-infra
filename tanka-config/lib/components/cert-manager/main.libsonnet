local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local cm = import 'github.com/jsonnet-libs/cert-manager-libsonnet/1.10/main.libsonnet';
local util = import 'util/main.libsonnet';

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

local ci = cm.nogroup.v1.clusterIssuer;

{
  _config:: {
    namespace: 'cert-manager',
  },

  all(clusterIssuer, ingressClass):
    helm.template('cert-manager', './charts/cert-manager', {
      namespace: $._config.namespace,
    }) + {
      clusterIssuer:
        ci.new(clusterIssuer)
        + ci.metadata.withNamespace($._config.namespace)
        + ci.spec.acme.withEmail('info@bgottlob.com')
        + ci.spec.acme.withServer('https://acme-v02.api.letsencrypt.org/directory')
        + ci.spec.acme.privateKeySecretRef.withName(clusterIssuer + '-secret')
        + {
          spec+:
            {
              acme+:
                {
                  solvers: [
                    ci.spec.acme.solvers.http01.ingress.withClass(ingressClass),
                  ],
                },
            },
        },
    },
}
