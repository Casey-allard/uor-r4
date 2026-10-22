# web

`web/studio/` holds the Studio shell: the editor, chat and model-manager page (`index.html`, `index.css`, `assets/js/uor_model_worker.js`, `r4_worker.js`, `verify_studio_worker.mjs`, `coi-serviceworker.js`) and the standalone geometry explorer (`geometric_prime_router_webapp.html`). It is the planned home of the web client, a local in-browser model; see [`docs/products/web-client.md`](../docs/products/web-client.md) and [`docs/labs/m7-studio-connect-wasm-engine/README.md`](../docs/labs/m7-studio-connect-wasm-engine/README.md).

The published site is built by [`docs-site.yml`](../.github/workflows/docs-site.yml): the book is served at the root and the Studio at `/studio/`. The old `deploy.yml` job that copied these files remains disabled (D36).

## Build the WASM bundle and verify the Studio worker locally

Build the `crates/uor-r4-integer` (`serving-wasm`) WebAssembly bundle into the gitignored `web/studio/pkg/` directory, generate the deterministic synthetic stack fixture (`synthetic-stack.lut`), and run the automated Studio worker verification and D11 instruction audit:

```sh
RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt --out-dir ../../web/studio/pkg -- --features serving-wasm
~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- 5 web/studio/pkg/synthetic-stack.lut
node web/studio/verify_studio_worker.mjs
python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap
```

Then serve `web/studio/` with any static file server and open it in a browser:

```sh
ruby -run -ehttpd web/studio -p 8000   # then open http://localhost:8000/
```
