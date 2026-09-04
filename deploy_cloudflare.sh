#!/usr/bin/env bash
# ==============================================================================
# Cloudflare Deployment Entrypoint Wrapper (Delegates to standardized deploy.sh)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
chmod +x "$SCRIPT_DIR/deploy.sh" 2>/dev/null || true
exec bash "$SCRIPT_DIR/deploy.sh" "$@"
