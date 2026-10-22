# AVL and Red-Black Trees

A Rust implementation and comparison of AVL trees red-black trees, with a local browser visualizer powered by WebAssembly and Criterion benchmarks.

[Leia em português (Brasil)](README.pt-BR.md).

## Project contents

- `src/tree/avl`: AVL tree implementation.
- `src/tree/rbtree`: left-leaning red-black tree implementation.
- `src/wasm.rs`: WebAssembly interface used by the browser app.
- `web/`: static visualizer using the wasm code.
- `benches/tree.rs`: Criterion benchmarks for insertion, removal, and search.

The visualizer accepts integer keys, shows tree metadata, and lets you inspect
searches, rotations, and color changes frame by frame. Its tree state stays in
memory in the browser.

## Requirements

- Rust and Cargo.
- For the web app: `just`, `wasm-pack`, `miniserve`, and the Rust
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

Install `just` and the Rust static-file server:

```sh
cargo install --locked just
cargo install --locked miniserve
```
The `just dev-web` recipe builds the WASM package and serves the static page at
<http://localhost:8000>:

```sh
just dev-web
```

## Web visualizer

The visualizer is a static web app. Tree operations run in the Rust WebAssembly
module; the page uses plain HTML, CSS, and JavaScript to display tree snapshots
and operation frames. Choose AVL or red-black and insert, delete, or search
integer keys. Press Enter in the key field to insert. Use the timeline, arrow
buttons, or keyboard shortcuts to review frames. Playback continues through new
frames if an operation starts while it is playing. Rotation and recoloring
frames show changed snapshots, and unchanged snapshots are omitted.

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
./target/release/deps/tree-<hash> --bench --quick
```
