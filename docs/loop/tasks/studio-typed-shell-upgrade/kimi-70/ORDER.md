# ORDER: share the mounted Studio runtime across web, native runner, and Tauri, kimi-70

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `000f4b457ee020013654ffbc65f9da5061c2317a`, tree `fed1361391027ca2ef8f9d3bd2041451014eef70`; original merge base is the same.

## Findings to repair

- Native runner downloads and serves a pinned legacy UI archive rather than the reviewed mounted Preact build.
- Tauri stages that legacy page, then constructs `WebviewUrl::App("index.html")`; no typed mount or shared `dist` packaging exists.
- Runtime allowlisting and package verification understand legacy source files, not hashed Vite assets or the early migration runtime.
- Packaged-window acceptance found that the typed Changes/Terminal panels construct a second desktop adapter without the declared Tauri `invoke`, falsely reporting browser-only capability while native folder access works. Repair the existing factory default and its regression coverage without changing host approval or adding a service.
- Exact-head acceptance then failed to build `uor-chat` after the desktop verifier had used the same target: unhashed mixed-workspace artifacts produced duplicate tokenizer/serde types even after package cleaning. A fresh isolated target succeeded. Namespace the local runner's generated target and prove the desktop-before-native order without changing dependency versions or model code.

## Selected approach and cancel probe

Use one generated Vite `dist` layout for static web preview, a local-only native-runner path, and Tauri's existing `WebviewUrl::App`. Native paths use `index.html?typed=1` so the early singleton migration and mounted bootstrap activate before legacy readers. No second desktop UI or extra loopback service is added.

The cancel probe is the current legacy archive/runtime, absent typed allowlist, and absence of a local native-runner path or packaged typed-window evidence. Red-first tests must fail on those missing contracts before implementation.

## Files in scope

- `scripts/studio/runtime-files.sh`, `native-runner.sh`, `prepare-tauri-runtime.sh`, `verify-tauri-app.sh`, `catalog.mjs`, `scripts/stage-studio-site.sh`, and directly related runtime tests.
- `web/studio/src/stage-dist.mjs`, `web/studio/assets/js/desktop_actions.js` only for the declared-host factory default, its focused adapter tests, `web/studio/README.md`, `docs/products/web-client.md`, `docs/integration/current-state.md`, the approved design, roadmap/notebook evidence, and this task folder.
- `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs` only for the typed App URL and package assertions, `src-tauri/tests/**` only for focused runtime evidence. `src-tauri/src/main.rs` is unchanged.

No new dependency, external test endpoint/credential, model/engine/worker/tokenizer/`studio_api.js` change, release/publication, production cutover, or Phase B compute. Preserve existing release defaults and the unsigned-development-only boundary.

## Done when

- Typed allowlist staging includes the exact Vite `dist` assets, generated pkg/tokenizer/model files, and no legacy source/development files or unexpected outputs.
- `native-runner.sh --local` uses the same local build and sidecar, prints `/?typed=1`, and performs no release download; its mounted workflow returns an actual UOR response or explicit unsupported state with no external requests.
- Tauri stages and loads `index.html?typed=1` from the same bytes. Packaged app/shared-UI/native-host smoke checks prove the mounted application and real backend contract, with capability differences explicit and no external page-load requests.
- Full product/legacy/repository checks, fresh exact-head approval, protected merge, and fresh-main verification pass. Production publication and final full acceptance remain separate.

## Stop and report

Stop before weakening a check, using a legacy archive as shared-UI evidence, hiding a typed mount failure, adding a dependency/service, touching real browser data/credentials, or publishing. Continue only with the shared runtime and actual packaged/native checks green and freshly approved.
