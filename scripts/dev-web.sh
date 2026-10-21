#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
wasm-pack build --target web --out-dir web/pkg --no-pack
python3 -m http.server 8000 --directory web
