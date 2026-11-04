# REPORT: conversation-first Studio and working native connection

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Fresh main base: `2833c451`. Delivery is through the protected source PR; the Pages publish and its live identifiers are recorded only after they exist.

## What did not work

- The public model remains a synthetic protocol fixture. Its responses are not useful conversation, and no visual understanding, production readiness, M7/M8 completion or trained-model quality is claimed.
- GitHub Pages cannot host an API. HTTPS Pages must not call insecure HTTP loopback, and CORS alone is not a security or connectivity fix. No public tunnel, paid provider, plugin requirement, token persistence or admission/context weakening was added.
- A native browser prompt needing 69 positions against the synthetic stack's 64-position context returned HTTP 413 without truncation. A fresh short session was used for the successful stream.
- Browser inspection caught legacy responsive and ID-level rules hiding Connections/New chat and a dead mobile Menu. Focused regressions were added and the cascade corrected before packaging; no green claim rests only on source inspection.
- The current source/release evidence is not yet a new live-Pages claim. The existing `233b97fb9` publish remains the live reference until the redesigned page is pushed and verified.

## What changed

- `web/studio/index.html`, `index.css`, `assets/js/model_sources.js`, `studio_ui.js` and `studio_api.js`: one system-type dark conversation column, default-only bundled boot with stale-load protection, real offline/model/native actions, visible native guidance in the regrouped Connections view, rounded messages and floating composer, compact mobile controls, and actionable missing-runner, insecure-loopback, authentication and unsupported-endpoint errors. Existing keyboard, history, long-paste, attachment, image, session and API paths remain intact.
- New focused Studio/Node regressions cover the empty state, persisted-WASM boot gate, visible guide DOM, connection error call paths, panel-free layout, visible controls and mobile typography. Existing model cards and `models.json` are preserved for publish.
- `scripts/studio/native-runner.sh`: side-effect-free `--check`, pinned HTTPS downloads and SHA-256 verification, safe runtime extraction, Apple silicon validation, and loopback-only `uor-chat --studio-root … --token-stdin`. The token stays in memory and is not printed. The verified archive supplies both synthetic artifacts, so the runner has no mutable Pages dependency. Research-only FBS filenames are listed as not downloaded.
- [Native runner prerelease](https://github.com/UOR-Foundation/uor-r4/releases/tag/studio-native-runner-v0.1.0): `native-runner.sh`, `uor-chat-arm64-apple-darwin`, runtime-only `studio-local.tar.gz`, and `SHA256SUMS`. The archive contains no source library, tests, package/build configuration, declarations, nested verifier or README.
- Independent exact-head review returned CHANGES for hidden native guidance, persisted-WASM boot overwrite, raw connection errors, publish hygiene and retained evidence. The code findings were fixed RED→GREEN and the actual Native setup action was rechecked in the browser. The publish uses an explicit runtime allowlist, not the older copy-all staging helper.

## Measurements

- Full Studio suite: **289/289**, plus typecheck and Vite build. Node API suites: **45/45**. Native loopback: **15/15**. Vision: **9/9**. xtask: **222/222**.
- Worker: `ALL STUDIO WORKER CHECKS PASSED`, save/restore identity and `remoteRequests: 0`. Static and headless runtime page traps pass with `external: []`; planted external detection fires. WASM audit: **7/7**, zero float/unclassified instructions in the declared scope.
- Offline browser at 1440×900: automatic demo ready, Enter sent a real prompt, 32 completion tokens with live tok/s and finish `length`, and ArrowUp restored the exact prompt. At 390×852 the compact controls and composer remain visible with system typography. Native setup visibly displays the Pages/API and local-runner guidance.
- Native browser: authenticated runner-served same-origin stream returned **16 completion tokens**, exact prompt/completion/total usage **19/16/35**, measured **21,697.4 tok/s**, TTFT **0.366 ms**, elapsed **0.737 ms**, finish `length`. These are one local synthetic run's measurements, not a model benchmark.
- Release assets: binary SHA-256 `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`; Studio archive SHA-256 `bf78084e09d03736d3de5cfbe98040e632cc102946f6396434f12314b9ede04d`; synthetic stack `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`; tokenizer `6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b`.
- Fresh public `gh release download` verifies all three `SHA256SUMS` entries. A separate empty-cache launch of the downloaded runner reaches `/healthz` with `ready: true` and the expected stack hash. No token appears in agent logs.

## Checks run at the head

The full Studio/Node/native/vision/xtask suites, worker/static/runtime page traps, WASM audit, real offline/native browser streams, public release checksums, and empty-cache launch passed at the current source state. Repository gates and independent exact-head review are required before merge; their final outcomes and the protected merge are reported on the PR rather than preclaimed here.

## Skills used

Project research/delivery workflow, isolated worktree, test-driven development, systematic browser debugging, modern web forms/IME/dark-mode/color/accessibility guidance, real built-in-browser desktop/mobile verification, public artifact checksum verification, and protected PR review. No GPU, paid compute, new dependency, new trained model or publisher-code change was used.

## Still open

After protected merge and fresh-main verification, publish through the explicit runtime allowlist used for the verified release archive—not `scripts/stage-studio-site.sh`, which copies development files—restore the existing `models.json` byte-for-byte, and reject the publish if any source, test, package/build configuration, declaration, verifier or README path appears. Verify the hosted page and model cards, then update this report and live references with the actual gh-pages/Pages identifiers. Remove temporary Rust/build outputs and the delivered worktree while preserving evidence and the shared runtime. Useful conversation, trained visual understanding, M7/M8 and production readiness remain unestablished.

## Plain summary

The source Studio is now a simple dark chat interface that works immediately with the synthetic offline demo. The public prerelease gives Apple silicon users a verified local runner for the authenticated native API on the same loopback origin. GitHub Pages remains a static client, not an API host; no useful-chat or visual-understanding claim is made.
