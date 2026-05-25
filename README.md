# personal-infra

Infrastructure-as-code for my self-hosted Kubernetes cluster. It runs a handful
of personal services as well as my [blog](https://bgottlob.com).

Like any self-respecting software engineer would, I use my own infrastructure to
experiment with tooling that isn't common in enterprise environments.

> ⚠️ This is tailored to my specific setup and preferences. Feel free to peruse
> it for ideas and use pieces from it, but don't expect things to work for you
> out of the box.

## Stack

The stack used to build and host the infrastructure of this repo includes:

- **[Linode Kubernetes
  Engine](https://techdocs.akamai.com/cloud-computing/docs/linode-kubernetes-engine)**
  - a relatively cheap managed Kubernetes control plane that doesn't force
    enterprise features on me that I don't need
- **[OpenTofu](https://opentofu.org/)** - provisions the LKE cluster, object
  storage buckets, and DNS records on Linode, with remote state in a Linode
  object storage bucket.
- **[SOPS](https://github.com/getsops/sops)** - encrypts a secrets file (set
  with the `SECRETS_FILE` environment variable) that is decrypted and consumed
  by OpenTofu and Kubernetes deployments.
- **[Yoke](https://yokecd.github.io/)** - defines Kubernetes configuration in
  WebAssembly modules (in my case, written in Rust) for more flexibility over
  tools like Helm and Kustomize

These are the core Kubernetes components that support the services running:

- **[cert-manager](https://cert-manager.io/)** - provision TLS certificates for
  endpoints to services
- **[Envoy Gateway](https://gateway.envoyproxy.io/)** - Internet-facing public
  gateway with ingress points to services hosted in Kubernetes
- **[external-dns](https://kubernetes-sigs.github.io/external-dns/)** - syncs
  Gateway/HTTPRoute hostnames into the Linode domain.
- **[Tailscale Operator](https://tailscale.com/kb/1236/kubernetes-operator)** -
  exposes private services to my tailnet without punching holes in the public
  Gateway
- **[CloudNativePG](https://cloudnative-pg.io/)** - Postgres clusters for
  stateful apps, with continuous backups to object storage
- **[Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)** -
  cluster-native encrypted secrets, written from SOPS secrets file by a small
  custom CLI (`sops-seal`)
- **[Velero](https://velero.io/)** - cluster snapshots/backups to Linode object
  storage
- **[VictoriaMetrics](https://victoriametrics.com/)** - metrics collection
- **[Grafana](https://grafana.com)** - metric dashboards

Some of the services that are actually running in this repo include:

- **[bgottlob.com](https://bgottlob.com)** - my static blog site
- **[Miniflux](https://miniflux.app/)** - RSS feed reader
- **[Wallabag](https://wallabag.org/)** - read-later app
- **[Vikunja](https://vikunja.io/)** - task manager app
- **[Kavita](https://www.kavitareader.com/)** - ebook library
- **[rmfakecloud](https://github.com/ddvk/rmfakecloud)** - self-hosted
  reMarkable cloud alternative

---

## Repository Layout

```
.
├── tofu/                 # OpenTofu: LKE cluster, object storage, DNS
├── k8s-deployments/      # Yoke k8s deployments and library crates
│   ├── kube_builder/     #   shared typed builders over k8s-openapi
│   ├── helm/             #   thin Rust wrapper around `helm pull`/`template`
│   ├── sops-seal/        #   SOPS → SealedSecret translator
│   └── <app>/            #   each deployment crate
├── templates/            # cargo-generate templates for new Yoke k8s deployments
├── flake.nix             # pinned dev shell (kubectl, yoke, tofu, sops, etc.)
├── .envrc                # direnv: loads the flake, exports TOFU_ROOT, sops creds
└── justfile              # top-level task runner
```

## Local Development and Deployment

A Nix flake defines the local development environment, with automatic loading
capabilities with `direnv`. These are some key commands for local development
and deployment:

```sh
just                    # list every per-app recipe
just <app> debug        # render manifests to stdout (no cluster contact)
just <app> diff         # diff rendered manifests against the cluster
just <app> takeoff      # build WASM + apply to the cluster
just check              # render every deployment (CI sanity check)
just check --diff       # diff every deployment against the cluster
tofu plan               # always runs in ./tofu/ regardless of CWD
```

> The flake modifies `tofu` by injecting `-chdir=$TOFU_ROOT`, so the command
> works from anywhere in the repo.

Secrets are decrypted from a SOPS-encrypted file pointed to by `SECRETS_FILE`
(set in a local, gitignored `.env`). Without that file you can still render
deployments - sealed-secret material just falls back to placeholders.

## Contributing

I appreciate that you have taken interest in this repo, but I'm not taking any
contributions at this time since it is opinionated and tuned to my preferences.
However, I may generalize and release some portions of this repo in the future.

<!-- vim: set textwidth=80 formatoptions+=t: -->
