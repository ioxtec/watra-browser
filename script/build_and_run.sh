#!/usr/bin/env bash
set -euo pipefail
# Keep the desktop Run action pointed at the official Watra app.
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/run-watra.sh" "$@"
