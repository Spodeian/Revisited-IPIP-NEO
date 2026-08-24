#!/usr/bin/env bash
# Cloudflare Serverless Deployment Build Pipeline Script (Pages & Workers Hybrid)
set -e

echo "=== Initializing Cloudflare Serverless Build Pipeline ==="

# 1. Ensure Rust stable cargo bin path is in global environment
export PATH="$HOME/.cargo/bin:$PATH"

# 2. Check if rustup is installed, if not, install it quietly
if ! command -v rustup &> /dev/null && [ ! -f "$HOME/.cargo/bin/rustup" ]; then
    echo "Rust compiler not detected. Installing Rust stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Sourcing cargo environment is required to bind rustup on fresh installs
    . "$HOME/.cargo/env"
else
    if [ -f "$HOME/.cargo/bin/env" ]; then
        . "$HOME/.cargo/bin/env"
    elif [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi
    echo "Rust compiler toolchain detected: $(rustc --version || echo 'Local install active')"
fi

# 3. Add WASM compile target
echo "Adding WebAssembly compile target (wasm32-unknown-unknown)..."
if command -v rustup &> /dev/null; then
    rustup target add wasm32-unknown-unknown
else
    $HOME/.cargo/bin/rustup target add wasm32-unknown-unknown
fi

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

# 4b. Install updated wasm-opt (binaryen) with bulk memory operations support
BINARYEN_VERSION="version_118"
echo "Installing/updating wasm-opt (${BINARYEN_VERSION}) to support bulk memory operations..."
mkdir -p "$HOME/.cargo/bin"
wget -qO- "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz" | tar -xzf -
mv "binaryen-${BINARYEN_VERSION}/bin/wasm-opt" "$HOME/.cargo/bin/wasm-opt"
chmod +x "$HOME/.cargo/bin/wasm-opt"
rm -rf "binaryen-${BINARYEN_VERSION}"

echo "wasm-opt active version: $(wasm-opt --version)"

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
$TRUNK_BIN build --release --public-url "/"

echo "=== Build Completed Successfully! Static assets are ready in: 'crates/web/dist' ==="

# 6. Dynamic Deployment context router
# If CLOUDFLARE_WORKER_DEPLOY is explicitly set to true, run wrangler deployment.
# Otherwise, we default to Pages native CDN publishing which requires no worker scripts.
if [ "$CLOUDFLARE_WORKER_DEPLOY" = "true" ]; then
    echo "Wrangler Worker deployment context detected."
    if ! command -v wrangler &> /dev/null; then
        if command -v npm &> /dev/null; then
            echo "Installing Cloudflare Wrangler globally via npm..."
            npm install -g wrangler
        else
            echo "ERROR: npm is required to install Wrangler for Worker deployments."
            exit 1
        fi
    fi
    echo "Executing Wrangler Deploy..."
    wrangler deploy
else
    echo "Pages / Static CDN deployment context detected. Bypassing Wrangler deploy."
fi
