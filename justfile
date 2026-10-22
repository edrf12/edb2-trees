set shell := ["bash", "-euo", "pipefail", "-c"]

# Build the WebAssembly package and serve the static visualizer.
dev-web:
    wasm-pack build --target web --out-dir web/pkg --no-pack
    miniserve --port 8000 --index index.html web/
