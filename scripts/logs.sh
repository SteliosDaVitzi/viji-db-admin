#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == --help ]]; then echo "Usage: $0 dev|prod [backend|frontend]"; exit 0; fi
source "$(dirname "$0")/common.sh"
select_environment "${1:-}"; shift
[[ $# -le 1 ]] || { echo 'Too many arguments.' >&2; exit 1; }
case "${1:-}" in
  '') compose logs --follow --tail 100 ;;
  backend|frontend) compose logs --follow --tail 100 "$1" ;;
  *) echo 'Choose backend or frontend.' >&2; exit 1 ;;
esac
