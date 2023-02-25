#!/usr/bin/env bash
set -e

NODE_DETAILS=$(kubectl get node -o json)
jq '.items[0].status.addresses[] | select(.type == "ExternalIP")' <<< "${NODE_DETAILS}"
