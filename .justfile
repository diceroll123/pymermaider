# PyMermaider Justfile

# Default recipe to display help
default:
    @just --list

# Build WASM module for web
build-wasm:
    cd web && npm run build:wasm

# Install web dependencies
web-install:
    cd web && npm install

# Run web dev server (requires WASM to be built first)
web-dev port="3000":
    cd web && npm run dev -- -p {{port}}

# Build web app for production (static export)
web-build:
    cd web && npm run build
    @echo "Static site generated in web/out/"

# Serve the built site locally (for testing)
web-serve:
    @echo "Serving at http://localhost:8000"
    @echo "Press Ctrl+C to stop"
    cd web/out && python3 -m http.server 8000

# Setup web app (install deps + build WASM)
web-setup: web-install build-wasm

# Full web dev workflow (setup + dev server)
web port="3000": web-setup
    @just web-dev {{port}}

# Build CLI in release mode
build-cli:
    cargo build --release --features cli

# Run CLI
run-cli path:
    cargo run --release --features cli -- {{path}}

# Check WASM compilation
check-wasm:
    cargo check --lib --target wasm32-unknown-unknown --no-default-features

# Check CLI compilation
check-cli:
    cargo check --features cli

# Run Rust tests
test:
    cargo test --lib --no-default-features

# Run Rust linting
lint:
    cargo fmt --check
    cargo clippy --all-targets --all-features

# Run web linting
lint-web:
    cd web && npm ci && npm run lint

# Format Rust code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean
    rm -rf web/.next
    rm -rf web/public/wasm
    rm -rf target

# Clean and rebuild everything
rebuild: clean web-setup

# Add wasm32 target if not present
add-wasm-target:
    rustup target add wasm32-unknown-unknown

# Setup development environment
setup: add-wasm-target web-install
    @echo "✅ Development environment ready!"

# Verify setup
verify:
    @echo "Checking Rust..."
    @cargo --version
    @echo "Checking wasm32 target..."
    @rustup target list | grep "wasm32-unknown-unknown (installed)"
    @echo "Checking Node.js..."
    @node --version
    @echo "✅ All checks passed!"
