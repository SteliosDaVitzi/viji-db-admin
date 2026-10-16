#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --help ]]; then
  echo "Usage: $0 dev|prod [--no-cache] [migrate] [seed]"
  echo 'This viewer has no application database or migrations. Sample data is opt-in in development.'
  exit 0
fi
source "$(dirname "$0")/common.sh"
select_environment "${1:-}"; shift
no_cache=false
seed=false
for arg in "$@"; do
  case "$arg" in
    --no-cache) no_cache=true ;;
    migrate) echo 'No migrations: WebDB does not own or modify connected databases.' ;;
    seed) seed=true ;;
    *) echo "Unknown argument: $arg" >&2; exit 1 ;;
  esac
done
if $seed; then
  [[ "$COMPOSE" == *dev.yml ]] || { echo 'Sample database is available in development only.' >&2; exit 1; }
  [[ -f "$ROOT/.env" || -n "${POSTGRES_PASSWORD:-}" ]] || { echo 'Set POSTGRES_PASSWORD in .env or your environment first.' >&2; exit 1; }
  compose --profile sample-db up -d postgres
fi
if $no_cache; then compose build --no-cache; fi
compose up -d --build
