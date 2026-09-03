#!/usr/bin/env bash
# ==============================================================================
# Cloudflare Serverless Deployment Build Pipeline Script (Pages & Workers)
# ==============================================================================
set -euo pipefail

echo "=== Initializing Cloudflare Serverless Build Pipeline ==="

# 1. Persistent Environment & PATH Setup
export NODE_ENV="production"

if [ -d "/opt/buildhome" ]; then
    export CARGO_HOME="/opt/buildhome/.cargo"
    export RUSTUP_HOME="/opt/buildhome/.rustup"
else
    export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
fi

export PATH="$CARGO_HOME/bin:$HOME/.cargo/bin:/opt/buildhome/.cargo/bin:$PATH"
mkdir -p "$CARGO_HOME/bin"

if [ -f "$CARGO_HOME/env" ]; then
    . "$CARGO_HOME/env"
elif [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

# 2. Rust Toolchain & Target Verification (Resolves channel from rust-toolchain.toml)
RUST_TOOLCHAIN="stable"
if [ -f "rust-toolchain.toml" ]; then
    DETECTED_CHANNEL=$(grep -E '^\s*channel\s*=' rust-toolchain.toml | head -n 1 | cut -d '"' -f 2 | tr -d ' ' || true)
    if [ -n "$DETECTED_CHANNEL" ]; then
        RUST_TOOLCHAIN="$DETECTED_CHANNEL"
        echo "Detected Rust toolchain from rust-toolchain.toml: $RUST_TOOLCHAIN"
    fi
fi

if ! command -v rustup &> /dev/null && [ ! -f "$CARGO_HOME/bin/rustup" ]; then
    echo "Rust compiler not detected. Installing Rust ($RUST_TOOLCHAIN) toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$RUST_TOOLCHAIN" --target wasm32-unknown-unknown
    if [ -f "$CARGO_HOME/env" ]; then
        . "$CARGO_HOME/env"
    fi
else
    echo "Rust toolchain detected: $(rustc --version || echo 'Active')"
    if command -v rustup &> /dev/null; then
        rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --target wasm32-unknown-unknown 2>/dev/null || true
        rustup default "$RUST_TOOLCHAIN" 2>/dev/null || true
        rustup target add wasm32-unknown-unknown 2>/dev/null || true
    elif [ -f "$CARGO_HOME/bin/rustup" ]; then
        "$CARGO_HOME/bin/rustup" target add wasm32-unknown-unknown 2>/dev/null || true
    fi
fi

# 3. Trunk Asset Bundler (Check Cache & Validate Execution)
TRUNK_BIN=""
if command -v trunk &> /dev/null && trunk --version &> /dev/null; then
    TRUNK_BIN="trunk"
    echo "Cached Trunk binary detected: $(trunk --version)"
elif [ -x "$CARGO_HOME/bin/trunk" ] && "$CARGO_HOME/bin/trunk" --version &> /dev/null; then
    TRUNK_BIN="$CARGO_HOME/bin/trunk"
    echo "Cached Trunk binary detected: $("$TRUNK_BIN" --version)"
else
    echo "Downloading and caching latest Trunk asset bundler..."
    wget -qO- https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf - -C "$CARGO_HOME/bin"
    chmod +x "$CARGO_HOME/bin/trunk"
    TRUNK_BIN="$CARGO_HOME/bin/trunk"
    echo "Trunk installed: $("$TRUNK_BIN" --version)"
fi

# 4. Binaryen (wasm-opt) (Check Cache & Validate Execution)
WASM_OPT_BIN="$CARGO_HOME/bin/wasm-opt"

if [ -x "$WASM_OPT_BIN" ] && "$WASM_OPT_BIN" --version &> /dev/null; then
    echo "Cached wasm-opt detected: $("$WASM_OPT_BIN" --version)"
elif command -v wasm-opt &> /dev/null && wasm-opt --version &> /dev/null; then
    echo "System wasm-opt detected: $(wasm-opt --version)"
    WASM_OPT_BIN="wasm-opt"
else
    echo "Downloading and caching Binaryen wasm-opt..."
    BINARYEN_VERSION="version_122"
    temp_tar="/tmp/binaryen-${BINARYEN_VERSION}.tar.gz"
    wget -qO "$temp_tar" "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz" || \
    wget -qO "$temp_tar" "https://github.com/WebAssembly/binaryen/releases/latest/download/binaryen-x86_64-linux.tar.gz"
    tar -xzf "$temp_tar" -C /tmp
    find /tmp -name "wasm-opt" -type f -exec mv {} "$CARGO_HOME/bin/wasm-opt" \;
    chmod +x "$CARGO_HOME/bin/wasm-opt"
    rm -rf "$temp_tar" /tmp/binaryen*
    WASM_OPT_BIN="$CARGO_HOME/bin/wasm-opt"
    echo "wasm-opt installed: $("$WASM_OPT_BIN" --version || echo 'Ready')"
fi

# 5. Clean & Build Web Application
echo "Purging previous build distribution caches..."
rm -rf crates/web/dist dist

export RUSTFLAGS="-C target-feature=+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext,+reference-types,+multivalue -C link-arg=-zstack-size=2097152 -C target-cpu=generic ${RUSTFLAGS:-}"

echo "Compiling and bundling web application for release..."
"$TRUNK_BIN" clean
"$TRUNK_BIN" build --release --public-url "/"

DIST_DIR="crates/web/dist"
if [ ! -d "$DIST_DIR" ] && [ -d "dist" ]; then
    DIST_DIR="dist"
fi

# Run wasm-opt pass on generated wasm artifact with bulk memory, reference-types, and performance optimizations
WASM_OPT_FLAGS=(
    "-Oz"
    "--enable-bulk-memory"
    "--enable-bulk-memory-opt"
    "--enable-mutable-globals"
    "--enable-sign-ext"
    "--enable-nontrapping-float-to-int"
    "--enable-reference-types"
    "--enable-multivalue"
)

if [ -x "$WASM_OPT_BIN" ] || command -v wasm-opt &> /dev/null; then
    for wasm_file in "$DIST_DIR"/*.wasm; do
        if [ -f "$wasm_file" ]; then
            echo "Optimizing WASM with wasm-opt (reference-types, bulk-memory, fast math): $wasm_file"
            "$WASM_OPT_BIN" "${WASM_OPT_FLAGS[@]}" "$wasm_file" -o "$wasm_file" || "$WASM_OPT_BIN" -Oz "$wasm_file" -o "$wasm_file" || true
        fi
    done
fi

# 6. Deployment Cache Invalidation & Version Stamping
BUILD_ID=$(git rev-parse --short HEAD 2>/dev/null || date +%s)
echo "=== Stamping Deployment Cache Invalidation with BUILD_ID: $BUILD_ID ==="
if [ -f "$DIST_DIR/sw.js" ]; then
    sed -i "s/CACHE_NAME = '.*'/CACHE_NAME = 'ipip-neo-cache-${BUILD_ID}'/g" "$DIST_DIR/sw.js" 2>/dev/null || true
    sed -i "s/?v=[a-zA-Z0-9_-]*/?v=${BUILD_ID}/g" "$DIST_DIR/sw.js" 2>/dev/null || true
fi
if [ -f "$DIST_DIR/index.html" ]; then
    sed -i "s/?v=[a-zA-Z0-9_-]*/?v=${BUILD_ID}/g" "$DIST_DIR/index.html" 2>/dev/null || true
fi

# 7. Production Asset Minification (HTML, CSS, JS)
if [ -d "$DIST_DIR" ]; then
    echo "=== Running Production Asset Minification (HTML, CSS, JS) for '$DIST_DIR' ==="

    if command -v npx &> /dev/null; then
        echo "Minifying JavaScript and CSS assets using esbuild..."
        for js_file in "$DIST_DIR"/*.js; do
            if [ -f "$js_file" ]; then
                echo "  Minifying JS: $js_file"
                npx --yes esbuild "$js_file" --minify --allow-overwrite --outfile="$js_file" 2>/dev/null || true
            fi
        done
        for css_file in "$DIST_DIR"/*.css; do
            if [ -f "$css_file" ]; then
                echo "  Minifying CSS: $css_file"
                npx --yes esbuild "$css_file" --minify --allow-overwrite --outfile="$css_file" 2>/dev/null || true
            fi
        done
        if [ -f "$DIST_DIR/index.html" ]; then
            echo "  Minifying HTML: $DIST_DIR/index.html"
            npx --yes html-minifier-terser --collapse-whitespace --remove-comments --remove-redundant-attributes --remove-script-type-attributes --remove-style-link-type-attributes --use-short-doctype --minify-css true --minify-js true -o "$DIST_DIR/index.html" "$DIST_DIR/index.html" 2>/dev/null || true
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

    # 8. High-Ratio Asset Pre-Compression (Brotli Level 11 + Gzip Level 9)
    echo "=== Generating Pre-Compressed Brotli (.br) & Gzip (.gz) Assets ==="
    if command -v python3 &> /dev/null; then
        python3 -c '
import os, sys, gzip, glob

dist_dir = sys.argv[1]
extensions = ("*.wasm", "*.js", "*.css", "*.html", "*.json", "*.svg")
target_files = []
for root, _, _ in os.walk(dist_dir):
    for ext in extensions:
        target_files.extend(glob.glob(os.path.join(root, ext)))

# 1. Gzip Level 9
for fpath in target_files:
    if fpath.endswith(".gz") or fpath.endswith(".br"): continue
    gz_path = fpath + ".gz"
    try:
        with open(fpath, "rb") as f_in, gzip.open(gz_path, "wb", compresslevel=9) as f_out:
            f_out.write(f_in.read())
    except Exception as e:
        print(f"  Gzip failed for {fpath}: {e}")

# 2. Brotli Level 11 (if brotli module is available)
try:
    import brotli
    for fpath in target_files:
        if fpath.endswith(".gz") or fpath.endswith(".br"): continue
        br_path = fpath + ".br"
        with open(fpath, "rb") as f_in:
            data = f_in.read()
        compressed = brotli.compress(data, quality=11, mode=brotli.MODE_GENERIC)
        with open(br_path, "wb") as f_out:
            f_out.write(compressed)
    print("  Successfully pre-compressed assets with Brotli (q11) & Gzip (level 9)")
except ImportError:
    print("  Pre-compressed assets with Gzip (level 9). Brotli CLI check...")
' "$DIST_DIR" || true
        if command -v brotli &> /dev/null; then
            find "$DIST_DIR" -type f \( -name "*.wasm" -o -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.json" -o -name "*.svg" \) -exec brotli -f -k -q 11 {} + 2>/dev/null || true
        fi
    fi

    # 9. Ensure Cloudflare configuration files are guaranteed present in output distribution
    cp -f crates/web/_headers "$DIST_DIR/_headers" 2>/dev/null || true
    cp -f crates/web/_redirects "$DIST_DIR/_redirects" 2>/dev/null || true
fi

echo "=== Build Completed Successfully! Static assets are ready in: '$DIST_DIR' ==="

# 10. Deployment Context Router
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
