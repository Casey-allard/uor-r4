# web

`web/studio/` holds the Studio shell: the editor, chat and model-manager page (`index.html`, `index.css`, `assets/js/uor_model_worker.js`, `r4_worker.js`, `coi-serviceworker.js`) and the standalone geometry explorer (`geometric_prime_router_webapp.html`). It is the planned home of the web client, a local in-browser model; see [`docs/products/web-client.md`](../docs/products/web-client.md).

The published site is built by [`docs-site.yml`](../.github/workflows/docs-site.yml): the book is served at the root and the Studio at `/studio/`. The old `deploy.yml` job that copied these files is disabled (D36).

## Open the Studio locally

The repository had no written instructions for this. The Studio is static files, so serve `web/studio/` with any static file server and open it in a browser, for example:

```sh
ruby -run -ehttpd web/studio -p 8000   # then open http://localhost:8000/
```

The page imports `pkg/uor_r4_wasm_router.js`, a `wasm-pack` build output that is not committed (an engine retired on 15 October), so the engine does not load from a clean clone.
