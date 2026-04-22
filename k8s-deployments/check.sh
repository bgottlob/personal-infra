#!/usr/bin/env bash
# Verifies that all yoke deployments build and render manifests successfully.
# Uses `just debug` (yoke takeoff -stdout) which renders without deploying.
# Intended for CI — requires cargo with wasm32-wasip1 target and yoke in PATH.
set -uo pipefail

K8S_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -t 1 ]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    GREEN='' RED='' BOLD='' RESET=''
fi

PASSED=()
FAILED=()
FAILED_OUTPUTS=()  # parallel to FAILED: temp file path holding captured output
PLACEHOLDERS=()
TMPFILES=()

cleanup() {
    for f in "${PLACEHOLDERS[@]+"${PLACEHOLDERS[@]}"}"; do rm -f "$f"; done
    for f in "${TMPFILES[@]+"${TMPFILES[@]}"}"; do rm -f "$f"; done
}
trap 'cleanup; printf "\nInterrupted\n"; exit 130' INT TERM

# Discover deployment dirs: justfiles that import shared_yoke.just or define yoke
# commands directly (covers both the standard pattern and custom ones like cnpg-database)
mapfile -t DIRS < <(
    find "$K8S_DIR" -maxdepth 2 -name "justfile" \
        -exec grep -l "shared_yoke\|yoke takeoff" {} \; \
    | xargs -I{} dirname {} \
    | sort
)

run_check() {
    local dir="$1" recipe="$2" exit_code=0
    local label="$(basename "$dir"):$recipe"
    local sealed_file="$dir/sealed-secrets.json"
    local template_file="$dir/secrets-template.yaml"
    local placeholder_created=false
    local output_file
    output_file=$(mktemp)
    TMPFILES+=("$output_file")

    # If a secrets template exists but no sealed file, create a placeholder so
    # _maybe_seal skips re-sealing (which requires sops and a live secrets.yaml).
    if [ -f "$template_file" ] && [ ! -f "$sealed_file" ]; then
        echo '{"sealed-secrets": {}}' > "$sealed_file"
        PLACEHOLDERS+=("$sealed_file")
        placeholder_created=true
    fi

    (cd "$dir" && just "$recipe") > "$output_file" 2>&1 || exit_code=$?

    if [ "$placeholder_created" = true ]; then
        rm -f "$sealed_file"
        local i
        for i in "${!PLACEHOLDERS[@]}"; do
            if [ "${PLACEHOLDERS[$i]}" = "$sealed_file" ]; then
                unset 'PLACEHOLDERS[$i]'
                break
            fi
        done
    fi

    if [ $exit_code -eq 0 ]; then
        PASSED+=("$label")
        printf "${GREEN}PASS${RESET}: %s\n" "$label"
    else
        FAILED+=("$label")
        FAILED_OUTPUTS+=("$output_file")
        printf "${RED}FAIL${RESET}: %s\n" "$label"
    fi
}

for dir in "${DIRS[@]}"; do
    run_check "$dir" "debug"
    # Also run debug-restore for deployments that define it (e.g. cnpg-database)
    if grep -q "^debug-restore" "$dir/justfile"; then
        run_check "$dir" "debug-restore"
    fi
done

printf "\n${BOLD}Results: %d passed, %d failed${RESET}\n" "${#PASSED[@]}" "${#FAILED[@]}"

if [ ${#PASSED[@]} -gt 0 ]; then
    printf "\n${GREEN}Passed:${RESET}\n"
    for label in "${PASSED[@]}"; do
        printf "  %s\n" "$label"
    done
fi

if [ ${#FAILED[@]} -gt 0 ]; then
    printf "\n${RED}Failed:${RESET}\n"
    for i in "${!FAILED[@]}"; do
        printf "\n  %s\n" "${FAILED[$i]}"
        sed 's/^/    /' "${FAILED_OUTPUTS[$i]}"
    done
    cleanup
    exit 1
fi

cleanup
