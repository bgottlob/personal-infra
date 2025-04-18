local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local util = import 'util/main.libsonnet';

local clusterRole = k.rbac.v1.clusterRole;
local clusterRoleBinding = k.rbac.v1.clusterRoleBinding;
local container = k.core.v1.container;
local deployment = k.apps.v1.deployment;
local policyRule = k.rbac.v1.policyRule;
local ns = k.core.v1.namespace;
local serviceAccount = k.core.v1.serviceAccount;
local subject = k.rbac.v1.subject;

{
  _config:: {
    name: 'external-dns',
    image: {
      name: 'registry.k8s.io/external-dns/external-dns',
      tag: 'v0.15.1',
    },
    matchLabels: {
      app: 'external-dns',
    },
  },

  all(namespace='external-dns', domainFilter='bgottlob.com', linodeToken, txtOwnerId='bgottlob-k8s', txtPrefix='external-dns-'): {
    local clusterRoleName = $._config.name,
    local serviceAccountName = $._config.name,
    local serviceAccountObj = serviceAccount.new(serviceAccountName)
                              + serviceAccount.metadata.withNamespace(namespace),
    local secretName = 'linode',

    namespace: ns.new(namespace),
    serviceAccount: serviceAccountObj,
    clusterRole:
      clusterRole.new(clusterRoleName)
      + clusterRole.metadata.withNamespace(namespace)
      + clusterRole.withRules([
        policyRule.withApiGroups([''])
        + policyRule.withResources(['services', 'endpoints', 'pods'])
        + policyRule.withVerbs(['get', 'watch', 'list']),

        policyRule.withApiGroups(['extensions', 'networking.k8s.io'])
        + policyRule.withResources(['ingresses'])
        + policyRule.withVerbs(['get', 'watch', 'list']),

        policyRule.withApiGroups([''])
        + policyRule.withResources(['nodes'])
        + policyRule.withVerbs(['list']),
      ]),

    clusterRoleBinding:
      clusterRoleBinding.new($._config.name + '-viewer')
      + clusterRoleBinding.metadata.withNamespace(namespace)
      + clusterRoleBinding.roleRef.withKind('ClusterRole')
      + clusterRoleBinding.roleRef.withName(clusterRoleName)
      + clusterRoleBinding.withSubjects([
        subject.fromServiceAccount(serviceAccountObj),
      ]),

    deployment:
      deployment.new(
        name=$._config.name,
        replicas=1,
        containers=[
          container.new(
            $._config.name,
            $._config.image.name + ':' + $._config.image.tag,
          )
          + container.withArgs([
            '--source=ingress',
            '--domain-filter=' + domainFilter,
            '--provider=linode',
            '--registry=txt',
            '--txt-owner-id=' + txtOwnerId,
            '--txt-prefix=' + txtPrefix,
          ])
          + container.withEnv([
            util.envValueFromSecret('LINODE_TOKEN', secretName, 'token'),
          ]),
        ]
      )
      + deployment.metadata.withNamespace(namespace)
      + deployment.spec.strategy.withType('Recreate')
      + deployment.spec.selector.withMatchLabels($._config.matchLabels)
      + deployment.spec.template.metadata.withLabels($._config.matchLabels)
      + deployment.spec.template.spec.withServiceAccountName(serviceAccountName),

    secret: util.secretStringData(
      name=secretName,
      namespace=namespace,
      stringData={
        token: linodeToken,
      },
    ),
  },
}
