#!/usr/bin/env bash
# Verifies that all yoke deployments build and render manifests successfully.
# Uses `just debug` (yoke takeoff -stdout) which renders without deploying.
# Pass --diff to run `just diff` instead, comparing each deployment against the
# live cluster. Deployments with diffs exit non-zero, so a user can follow up
# with `just <deployment> takeoff`.
# Intended for CI — requires cargo with wasm32-wasip1 target and yoke in PATH.
set -uo pipefail

K8S_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DIFF_MODE=false
for arg in "$@"; do
    case "$arg" in
        --diff) DIFF_MODE=true ;;
        *) printf "Unknown flag: %s\n" "$arg" >&2; exit 1 ;;
    esac
done

if [ "$DIFF_MODE" = true ]; then
    PRIMARY_RECIPE="diff"
    RESTORE_RECIPE="diff-restore"
else
    PRIMARY_RECIPE="debug"
    RESTORE_RECIPE="debug-restore"
fi

if [ -t 1 ]; then
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    RED='\033[0;31m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    GREEN='' YELLOW='' RED='' BOLD='' RESET=''
fi

PASSED=()
DIFFED=()
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
        echo '{"sealed-secrets": []}' > "$sealed_file"
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

    if [ $exit_code -ne 0 ]; then
        FAILED+=("$label")
        FAILED_OUTPUTS+=("$output_file")
        printf "${RED}FAIL${RESET}: %s\n" "$label"
    elif [ "$DIFF_MODE" = true ] && grep -q '^---' "$output_file"; then
        DIFFED+=("$label")
        printf "${YELLOW}DIFF${RESET}: %s\n" "$label"
    else
        PASSED+=("$label")
        local pass_label
        pass_label="$( [ "$DIFF_MODE" = true ] && echo "NO DIFF" || echo "PASS" )"
        printf "${GREEN}%s${RESET}: %s\n" "$pass_label" "$label"
    fi
}

for dir in "${DIRS[@]}"; do
    run_check "$dir" "$PRIMARY_RECIPE"
    # Also run the restore variant for deployments that define it (e.g. cnpg-database).
    # Only in debug mode — diff mode compares against the live cluster where the
    # restore release typically does not exist.
    if [ "$DIFF_MODE" = false ] && grep -q "^debug-restore" "$dir/justfile"; then
        run_check "$dir" "$RESTORE_RECIPE"
    fi
done

if [ "$DIFF_MODE" = true ]; then
    printf "\n${BOLD}Results: %d no diff, %d with diffs, %d failed${RESET}\n" "${#PASSED[@]}" "${#DIFFED[@]}" "${#FAILED[@]}"
else
    printf "\n${BOLD}Results: %d passed, %d failed${RESET}\n" "${#PASSED[@]}" "${#FAILED[@]}"
fi

if [ ${#PASSED[@]} -gt 0 ]; then
    printf "\n${GREEN}%s:${RESET}\n" "$( [ "$DIFF_MODE" = true ] && echo "No diff" || echo "Passed" )"
    for label in "${PASSED[@]}"; do
        printf "  %s\n" "$label"
    done
fi

if [ ${#DIFFED[@]} -gt 0 ]; then
    printf "\n${YELLOW}Diffs:${RESET}\n"
    for label in "${DIFFED[@]}"; do
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
