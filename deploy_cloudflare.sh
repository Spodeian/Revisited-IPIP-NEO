#!/usr/bin/env bash
# Cloudflare Pages Serverless Deployment Build Pipeline Script
set -e

echo "=== Initializing Cloudflare Serverless Build Pipeline ==="

# 1. Ensure Rust stable cargo bin path is in global environment
export PATH="$HOME/.cargo/bin:$PATH"

# 2. Check if rustup is installed, if not, install it quietly
if ! command -v rustup &> /dev/null; then
    echo "Rust compiler not detected. Installing Rust stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
    echo "Rust compiler toolchain detected: $(rustc --version)"
fi

# 3. Add WASM compile target
echo "Adding WebAssembly compile target (wasm32-unknown-unknown)..."
rustup target add wasm32-unknown-unknown

# 4. Install Trunk if missing
if ! command -v trunk &> /dev/null; then
    echo "Installing Trunk asset bundler..."
    # Download compiled trunk release matching target environment architectures
    wget -qO- https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-
    TRUNK_BIN="./trunk"
else
    echo "Trunk detected: $(trunk --version)"
    TRUNK_BIN="trunk"
fi

# 5. Build static production assets
echo "Purging old Trunk build caches for a guaranteed up-to-date compile..."
if [ -d "crates/web/dist" ]; then
    rm -rf crates/web/dist
fi
if [ -d "dist" ]; then
    rm -rf dist
fi

echo "Compiling and bundling web application to distribution path..."
$TRUNK_BIN clean
$TRUNK_BIN build --release

echo "=== Build Completed Successfully! Static assets are ready in: 'crates/web/dist' ==="
