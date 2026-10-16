#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --help ]]; then echo "Usage: $0 dev"; echo 'Opens the explicitly enabled sample PostgreSQL database.'; exit 0; fi
source "$(dirname "$0")/common.sh"
[[ $# -eq 1 && "$1" == dev ]] || { echo 'Only the development sample database is managed by WebDB.' >&2; exit 1; }
select_environment dev
compose exec postgres sh -c 'exec psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"'
