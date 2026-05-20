secrets_file        := env_var("SECRETS_FILE")
template_file       := deployment_dir + "/secrets-template.yaml"
sealed_secrets_file := deployment_dir + "/sealed-secrets.json"

# Re-seal only when secrets.yaml or the template is newer than sealed-secrets.json
_maybe_seal:
    #!/usr/bin/env bash
    set -euo pipefail
    TEMPLATE="{{template_file}}"
    if [ ! -f "$TEMPLATE" ]; then
        exit 0
    fi
    SEALED="{{sealed_secrets_file}}"
    SECRETS="{{secrets_file}}"
    if [ ! -f "$SEALED" ] || [ "$SECRETS" -nt "$SEALED" ] || [ "$TEMPLATE" -nt "$SEALED" ]; then
        echo "Sealing secrets..."
        sops-seal --format json --secrets "$SECRETS" "$TEMPLATE" | jq '{"sealed-secrets": .}' > "$SEALED.tmp" && mv "$SEALED.tmp" "$SEALED"
    fi

# Build and deploy to the cluster
takeoff extra_flags="": _maybe_seal
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --target=wasm32-wasip1 --release
    if [ -f "{{sealed_secrets_file}}" ]; then
        yoke takeoff {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/release/{{wasm_name}}.wasm < {{sealed_secrets_file}}
    else
        yoke takeoff {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/release/{{wasm_name}}.wasm
    fi

# Build and print rendered manifests to stdout without deploying
debug extra_flags="": _maybe_seal
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --target=wasm32-wasip1
    if [ -f "{{sealed_secrets_file}}" ]; then
        yoke takeoff -stdout {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{wasm_name}}.wasm < {{sealed_secrets_file}}
    else
        yoke takeoff -stdout {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{wasm_name}}.wasm
    fi

# Build and show a diff of changes without deploying
diff extra_flags="": _maybe_seal
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --target=wasm32-wasip1
    if [ -f "{{sealed_secrets_file}}" ]; then
        yoke takeoff -diff-only {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{wasm_name}}.wasm < {{sealed_secrets_file}}
    else
        yoke takeoff -diff-only {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{wasm_name}}.wasm
    fi
