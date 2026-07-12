default:
    @just --list

# Check that all yoke deployments build and render manifests successfully.
# Pass --diff to compare each deployment against the live cluster instead.
check *args:
    @"{{ justfile_directory() }}/k8s-deployments/check.sh" {{ args }}

mod blog                        'k8s-deployments/blog'
mod cert-manager                'k8s-deployments/cert-manager'
mod cnpg-database               'k8s-deployments/cnpg-database'
mod cnpg-operator               'k8s-deployments/cnpg-operator'
mod csi-driver-linode           'k8s-deployments/csi-driver-linode'
mod envoy-gateway               'k8s-deployments/envoy-gateway'
mod external-dns                'k8s-deployments/external-dns'
mod grafana                     'k8s-deployments/grafana'
mod kavita                      'k8s-deployments/kavita'
mod kube-state-metrics          'k8s-deployments/kube-state-metrics'
mod metrics-server              'k8s-deployments/metrics-server'
mod miniflux                    'k8s-deployments/miniflux'
mod registry                    'k8s-deployments/registry'
mod rmfakecloud                 'k8s-deployments/rmfakecloud'
mod sealed-secrets              'k8s-deployments/sealed-secrets'
mod tailscale                   'k8s-deployments/tailscale'
mod umami                       'k8s-deployments/umami'
mod velero                      'k8s-deployments/velero'
mod victoria-metrics            'k8s-deployments/victoria-metrics'
mod victoria-metrics-operator   'k8s-deployments/victoria-metrics-operator'
mod vikunja                     'k8s-deployments/vikunja'
mod wallabag                    'k8s-deployments/wallabag'
