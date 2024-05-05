local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local cm = import 'github.com/jsonnet-libs/cert-manager-libsonnet/1.10/main.libsonnet';

local certificate = cm.nogroup.v1.certificate;
local ingress = k.networking.v1.ingress;
local ingressRule = k.networking.v1.ingressRule;
local secret = k.core.v1.secret;

{
  tlsIngress(name, namespace, hostname, issuerName='letsencrypt-prod', issuerKind='ClusterIssuer', ingressClass='nginx'): {
    local tlsName = '%s-tls' % name,

    ingress: ingress.new(
               name
             )
             + ingress.metadata.withNamespace(namespace)
             + ingress.metadata.withAnnotations({
               'cert-manager.io/cluster-issuer': 'letsencrypt-prod',
             })
             + ingress.spec.withIngressClassName(ingressClass)
             + ingress.spec.withTls({ hosts: [hostname], secretName: tlsName })
             + ingress.spec.withRules([
               ingressRule.withHost(hostname) + {
                 http: {
                   paths: [{
                     pathType: 'Prefix',
                     path: '/',
                     backend: {
                       service: {
                         name: name,
                         port: { number: 80 },
                       },
                     },
                   }],
                 },
               },
             ]),

    certificate: certificate.new(
                   tlsName
                 ) + certificate.metadata.withNamespace(namespace)
                 + certificate.spec.withSecretName(tlsName)
                 + certificate.spec.issuerRef.withName(issuerName)
                 + certificate.spec.issuerRef.withKind(issuerKind)
                 + certificate.spec.withDnsNames(hostname),
  },

  secretStringData(name, namespace, stringData): {
    secret: secret.new(name, {})
            + secret.metadata.withNamespace(namespace)
            + secret.withStringData(stringData),
  },
}
