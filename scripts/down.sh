#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --help ]]; then echo "Usage: $0 dev|prod [volumes] [images]"; exit 0; fi
source "$(dirname "$0")/common.sh"
select_environment "${1:-}"; shift
flags=()
for arg in "$@"; do
  case "$arg" in
    volumes) flags+=(--volumes) ;;
    images) flags+=(--rmi local) ;;
    *) echo "Unknown argument: $arg" >&2; exit 1 ;;
  esac
done
compose down "${flags[@]}"
