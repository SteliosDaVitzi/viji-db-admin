#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
select_environment() {
  case "${1:-}" in
    dev|prod) COMPOSE="$ROOT/docker-compose.$1.yml" ;;
    *) echo "Choose an environment: dev | prod" >&2; exit 1 ;;
  esac
}
compose() { docker compose --project-directory "$ROOT" -f "$COMPOSE" "$@"; }
