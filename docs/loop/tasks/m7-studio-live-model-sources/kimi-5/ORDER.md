# ORDER: conversation-first Studio and working native connection

Lab: kimi. Item: [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md) (#3440). Owner rejects the panel-heavy interface and reports that the API is unreachable. Base: `3f7f8605bf69e7a1069cf23cf34fa33de580743e`, tree `74d2978656559a96e20a339d17a8bcbd672fec66`.

## Goal

Deliver a modern, quiet Apple-like dark conversation interface that works on first load, plus an honest runner-served local page for real local-model streaming. The static Pages site remains the client, never an API host. Do not claim useful conversation, trained visual understanding, M7/M8 completion, or a production-ready model.

## Design

[DESIGN.md](DESIGN.md). Offline demo first; secondary navigation and diagnostics move out of the main flow. The native runner serves the same UI and authenticated API on loopback only. A public tunnel is deferred because the current static bootstrap would expose its bearer token and fail Host validation. HTTPS Pages must not call insecure HTTP loopback. No plugin, remote model host, paid provider, or CORS-only workaround is presented as the fix.

## Files in scope

- `web/studio/index.html`, `index.css`, `assets/js/**`, focused `src/adapters/**` tests, `README.md`, and runtime-only generated package output.
- `scripts/studio/native-runner.sh` and `scripts/tests/test_native_runner.py`; no engine or workflow changes.
- This task folder, `docs/products/web-client.md`, `docs/roadmap/items/m7-studio-live-model-sources.md`, `docs/research/notebook/m4-serving.md`, and `web/studio/README.md` for live records.

## Done when

- A single conversation view has no default left/right panels, dense setup steps, or unreachable-default provider. It has legible system typography, rounded message/composer treatment, responsive mobile layout, and a working first-run offline demo. Existing Enter/history/long-paste/file/image/session behavior remains covered.
- Connection setup distinguishes offline demo and native runner-served local Studio. It explains why GitHub Pages cannot host an API and why HTTP loopback plus CORS is insufficient. No public tunnel or secret-logging transport is added.
- The runner downloads pinned public synthetic demo/tokenizer artifacts and a pinned macOS arm64 binary, verifies their hashes, and offers an explicit loopback-only mode. Binary release assets are verified. No secret is printed in agent logs or committed.
- New failing-first tests cover the simplified empty state, auto demo loading, runner contract, and error guidance. Existing Studio/native/vision/worker checks pass.
- Desktop and mobile browser inspection shows the redesigned interface. A real default-demo prompt works, and a real native OpenAI stream through runner-served loopback returns exact usage. Hosted publish is verified with no development-only files added.
- Repository gates, independent exact-head review, protected merge, fresh-main verification, and cleaned workspace pass.

## Stop and report conditions

Stop before claiming GitHub Pages can host an API, using HTTP loopback from HTTPS Pages, persisting image blobs or launch tokens in agent logs, adding a paid/external model provider, weakening existing admission/context bounds, publishing development files, or making a useful-chat/visual-understanding claim. Merge forward from fresh `main`; never force-push. Reuse existing Rust/npm runtimes and preserve unique evidence and at least 30 GiB free after cleanup.
