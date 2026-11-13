# PyMermaider Web

Web interface for generating Mermaid diagrams from Python code. Built with Next.js and Chakra UI v3, runs entirely in your browser using WebAssembly.

## Development

```bash
# Full dev workflow
just web              # Setup + dev server

# Or step by step
just web-setup        # First time only
just build-wasm       # Build WASM module
just web-dev          # Run dev server
```

## Testing Production Build

```bash
just web-build        # Build static site
just web-serve        # Serve at localhost:8000
```

**Important:** Do NOT open `index.html` directly in a browser (file://). JavaScript modules require an HTTP server.

## Deployment

GitHub Actions automatically builds WASM and deploys to GitHub Pages on every push to main.

**One-time setup:**
1. Go to repository Settings → Pages
2. Under "Build and deployment", select "GitHub Actions" as the source
3. Push to main branch

## Inference Limits

The web app runs type inference entirely in WebAssembly on a single in-memory file. Imports from other modules (and third-party packages) cannot be resolved, so those types are treated as `Any` unless they are explicitly annotated. The native CLI resolves them project-wide, including from your virtual environment. See the root README, "Inference & environments".

## Tech Stack

- Next.js 16
- Chakra UI v3
- Rust/WASM
- Mermaid.js
