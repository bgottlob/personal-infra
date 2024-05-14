local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local cm = import 'github.com/jsonnet-libs/cert-manager-libsonnet/1.10/main.libsonnet';
local util = import 'util/main.libsonnet';

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

{
  local addNamespace() = {
    metadata+: { namespace: 'planka' },
  },

  all(pgUrl, secretKey, adminName, adminEmail, adminUsername, adminPassword, clusterIssuer='letsencrypt-prod', ingressClass='nginx', hostname='planka.bgottlob.com'):
    helm.template('planka', './charts/planka', {
      namespace: 'planka',
      values: {
        secretkey: secretKey,

        admin_name: adminName,
        admin_email: adminEmail,
        admin_username: adminUsername,
        admin_password: adminPassword,

        dburl: pgUrl + '/planka?sslmode=disable',

        postgresql: { enabled: false },
        persistence: { enabled: false },

        ingress: {
          enabled: 'true',
          annotations: {
            'cert-manager.io/cluster-issuer': clusterIssuer,
          },
          className: ingressClass,
          hosts: [{
            host: hostname,
            paths: [{
              path: '/',
              pathType: 'Prefix',
            }],
          }],
          tls: [{
            secretName: 'planka-tls',
            hosts: [hostname],
          }],
        },
      },
    })
    + { deployment_planka+: addNamespace() }
    + { pod_planka_test_connection+: addNamespace() }
    + { service_account_planka+: addNamespace() }
    + { service_planka+: addNamespace() }
    + { ingress_planka+: addNamespace() },
}
