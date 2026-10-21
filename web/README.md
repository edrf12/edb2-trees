# Tree Study

This is a static web app. The AVL and red-black tree operations run in the Rust
WASM module; the page uses plain HTML, CSS, and JavaScript to display the
resulting snapshots and operation frames.

## Local development

Install `wasm-pack`, add the `wasm32-unknown-unknown` Rust target, then run from
the repository root:

```sh
./scripts/dev-web.sh
```

This builds the WASM package and serves the page at <http://localhost:8000>.
The local server is only for development; the app makes no API calls and needs
no network once its static files are served.

## GitHub Pages

The Pages workflow builds the WASM package and deploys the `web/` directory on
push to `main`. Enable GitHub Pages for the repository and select **GitHub
Actions** as the build and deployment source.
