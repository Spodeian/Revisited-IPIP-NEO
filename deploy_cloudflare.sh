#!/usr/bin/env bash
# ==============================================================================
# Cloudflare Deployment Entrypoint Wrapper (Delegates to standardized deploy.sh)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/deploy.sh" "$@"
