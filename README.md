# AVL and Red-Black Trees

A Rust implementation and comparison of AVL trees and left-leaning red-black
trees (LLRB), with a local browser visualizer powered by WebAssembly and
Criterion benchmarks.

[Leia em português (Brasil)](README.pt-BR.md).

## Project contents

- `src/tree/avl`: AVL tree implementation.
- `src/tree/rbtree`: left-leaning red-black tree implementation.
- `src/wasm.rs`: WebAssembly interface used by the browser app.
- `web/`: static visualizer, built without a JavaScript framework.
- `benches/tree.rs`: Criterion benchmarks for insertion, removal, and search.

The visualizer accepts integer keys, shows tree metadata, and lets you inspect
searches, rotations, and color changes frame by frame. Its tree state stays in
memory in the browser.

## Requirements

- Stable Rust and Cargo (edition 2024).
- For the web app: `wasm-pack`, Python 3, and the Rust
  `wasm32-unknown-unknown` target.

## Build and verify

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Build the browser's WebAssembly package with:

```sh
wasm-pack build --target web --out-dir web/pkg --no-pack
```

## Run the web app locally

The development script builds the WASM package and serves the static page at
<http://localhost:8000>:

```sh
./scripts/dev-web.sh
```

The app has no backend and makes no network requests. A localhost server is
needed during development because browsers do not load WebAssembly modules
from `file://` pages.

GitHub Actions deploys the static site to GitHub Pages on pushes to `main`. To
use a custom domain, configure it in the repository's **Settings → Pages** and
add the domain's DNS records as GitHub Pages instructs. The page is static, so
it can also be hosted on another static-site host.

More web app details: [English web README](web/README.md) ·
[README em português](web/README.pt-BR.md).

## Criterion benchmarks

The benchmark suite compares AVL and red-black trees for insertion, removal,
and search. It uses tree sizes 100, 1,000, and 10,000; insertion and removal
orders are ascending, descending, and alternating. Search benchmarks include
hits and misses.

Run the full Criterion suite:

```sh
cargo bench --bench tree
```

For a quicker run with fewer samples:

```sh
cargo bench --bench tree -- --quick
```

Criterion reports are written under `target/criterion/`.

To compile the optimized benchmark executable without running it:

```sh
cargo bench --bench tree --no-run
```

Cargo prints the executable path, usually similar to
`target/release/deps/tree-<hash>`. Run that executable directly and pass
Criterion options after it:

```sh
./target/release/deps/tree-<hash> --quick
```

## Downloadable macOS benchmark

Pushing a version tag (`v*`) triggers the release workflow. It builds native
Apple Silicon and Intel macOS benchmark executables and attaches them to the
GitHub Release for that tag. Download the executable matching your Mac's
architecture and run it with optional Criterion flags, for example
`./tree-bench-macos-arm64 --quick`.

Create and push a release tag with:

```sh
git tag v0.1.0
git push origin v0.1.0
```
