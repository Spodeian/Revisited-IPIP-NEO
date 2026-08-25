#!/usr/bin/env bash
# ==============================================================================
# Cloudflare Serverless Deployment Build Pipeline Script (Pages & Workers)
# ==============================================================================
set -euo pipefail

echo "=== Initializing Cloudflare Serverless Build Pipeline ==="

# 1. Environment & PATH Setup
export PATH="$HOME/.cargo/bin:$PATH"
mkdir -p "$HOME/.cargo/bin"

# 2. Rust Toolchain & Target Verification
if ! command -v rustup &> /dev/null && [ ! -f "$HOME/.cargo/bin/rustup" ]; then
    echo "Rust compiler not detected. Installing Rust stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --target wasm32-unknown-unknown
    . "$HOME/.cargo/env"
else
    if [ -f "$HOME/.cargo/bin/env" ]; then
        . "$HOME/.cargo/bin/env"
    elif [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi
    echo "Rust toolchain detected: $(rustc --version || echo 'Active')"
    if command -v rustup &> /dev/null; then
        rustup target add wasm32-unknown-unknown
    else
        "$HOME/.cargo/bin/rustup" target add wasm32-unknown-unknown
    fi
fi

# 3. Trunk Asset Bundler Installation
if ! command -v trunk &> /dev/null; then
    echo "Installing Trunk asset bundler..."
    wget -qO- https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf - -C "$HOME/.cargo/bin"
    chmod +x "$HOME/.cargo/bin/trunk"
    TRUNK_BIN="trunk"
else
    echo "Trunk detected: $(trunk --version)"
    TRUNK_BIN="trunk"
fi

# 4. Binaryen (wasm-opt) v132 Installation
BINARYEN_VERSION="version_132"
WASM_OPT_BIN="$HOME/.cargo/bin/wasm-opt"

install_wasm_opt() {
    echo "Downloading and installing Binaryen wasm-opt (${BINARYEN_VERSION})..."
    local temp_tar="/tmp/binaryen-${BINARYEN_VERSION}.tar.gz"
    wget -qO "$temp_tar" "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz"
    tar -xzf "$temp_tar" -C /tmp
    mv "/tmp/binaryen-${BINARYEN_VERSION}/bin/wasm-opt" "$WASM_OPT_BIN"
    chmod +x "$WASM_OPT_BIN"
    rm -rf "$temp_tar" "/tmp/binaryen-${BINARYEN_VERSION}"
}

if [ ! -f "$WASM_OPT_BIN" ]; then
    install_wasm_opt
fi

echo "Active wasm-opt version: $($WASM_OPT_BIN --version || echo 'Installed')"

# 5. Clean & Build Web Application
echo "Purging previous build distribution caches..."
rm -rf crates/web/dist dist

echo "Compiling and bundling web application for release..."
$TRUNK_BIN clean
$TRUNK_BIN build --release --public-url "/"

# 6. Production Asset Minification (HTML, CSS, JS)
DIST_DIR="crates/web/dist"
if [ ! -d "$DIST_DIR" ] && [ -d "dist" ]; then
    DIST_DIR="dist"
fi

if [ -d "$DIST_DIR" ]; then
    echo "=== Running Production Asset Minification (HTML, CSS, JS) for '$DIST_DIR' ==="

    if command -v npx &> /dev/null; then
        echo "Minifying JavaScript and CSS assets using esbuild..."
        for js_file in "$DIST_DIR"/*.js; do
            if [ -f "$js_file" ]; then
                echo "  Minifying JS: $js_file"
                npx --yes esbuild "$js_file" --minify --allow-overwrite --outfile="$js_file" || true
            fi
        done
        for css_file in "$DIST_DIR"/*.css; do
            if [ -f "$css_file" ]; then
                echo "  Minifying CSS: $css_file"
                npx --yes esbuild "$css_file" --minify --allow-overwrite --outfile="$css_file" || true
            fi
        done
        if [ -f "$DIST_DIR/index.html" ]; then
            echo "  Minifying HTML: $DIST_DIR/index.html"
            npx --yes html-minifier-terser --collapse-whitespace --remove-comments --remove-redundant-attributes --remove-script-type-attributes --remove-style-link-type-attributes --use-short-doctype --minify-css true --minify-js true -o "$DIST_DIR/index.html" "$DIST_DIR/index.html" || true
        fi
    elif command -v python3 &> /dev/null; then
        echo "Node/npx not available. Using Python minification engine fallback..."
        python3 -c '
import os, re, sys, glob

dist_dir = sys.argv[1]

def minify_css(content):
    content = re.sub(r"/\*[\s\S]*?\*/", "", content)
    content = re.sub(r"\s+", " ", content)
    content = re.sub(r"\s*([\{\}:;,])\s*", r"\1", content)
    content = content.replace(";}", "}")
    return content.strip()

def minify_js(content):
    lines = []
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("//") and not stripped.startswith("///"):
            continue
        lines.append(line)
    content = "\n".join(lines)
    content = re.sub(r"/\*[\s\S]*?\*/", "", content)
    content = re.sub(r"[ \t]+", " ", content)
    content = re.sub(r"\n\s*", "\n", content)
    content = re.sub(r"\s*([=+\-*/%&|!<>?:,;{}()[\]])\s*", r"\1", content)
    return content.strip()

for fpath in glob.glob(os.path.join(dist_dir, "*.css")):
    try:
        with open(fpath, "r", encoding="utf-8") as f:
            c = f.read()
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(minify_css(c))
        print(f"  Minified CSS: {fpath}")
    except Exception as e:
        print(f"  Error minifying {fpath}: {e}")

for fpath in glob.glob(os.path.join(dist_dir, "*.js")):
    try:
        with open(fpath, "r", encoding="utf-8") as f:
            c = f.read()
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(minify_js(c))
        print(f"  Minified JS: {fpath}")
    except Exception as e:
        print(f"  Error minifying {fpath}: {e}")

html_path = os.path.join(dist_dir, "index.html")
if os.path.exists(html_path):
    try:
        with open(html_path, "r", encoding="utf-8") as f:
            html = f.read()
        html = re.sub(r"<!--(?!\[if)[\s\S]*?-->", "", html)
        html = re.sub(r"<style[^>]*>([\s\S]*?)</style>", lambda m: f"<style>{minify_css(m.group(1))}</style>", html, flags=re.IGNORECASE)
        html = re.sub(r">\s+<", "><", html)
        html = re.sub(r"[ \t]+", " ", html)
        with open(html_path, "w", encoding="utf-8") as f:
            f.write(html.strip())
        print(f"  Minified HTML: {html_path}")
    except Exception as e:
        print(f"  Error processing {html_path}: {e}")
' "$DIST_DIR"
    fi
fi

echo "=== Build Completed Successfully! Static assets are ready in: '$DIST_DIR' ==="

# 7. Deployment Context Router
if [ "${CLOUDFLARE_WORKER_DEPLOY:-false}" = "true" ]; then
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
    echo "Pages / Static CDN deployment context detected. Build ready for publishing."
fi
