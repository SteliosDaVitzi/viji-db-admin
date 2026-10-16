#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --help ]]; then
  echo "Usage: $0 dev|prod [--no-cache] [migrate]"
  echo 'SQLite saved-connection storage initializes automatically. Configure WEBDB_VAULT_KEY in .env.'
  exit 0
fi
source "$(dirname "$0")/common.sh"
select_environment "${1:-}"; shift
no_cache=false
for arg in "$@"; do
  case "$arg" in
    --no-cache) no_cache=true ;;
    migrate) echo 'The SQLite connection-store schema initializes automatically at backend startup.' ;;
    *) echo "Unknown argument: $arg" >&2; exit 1 ;;
  esac
done
if $no_cache; then compose build --no-cache; fi
compose up -d --build
