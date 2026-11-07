# ORDER: repair Studio loading and share exact arithmetic

Lab: kimi. Item: [m7-lexlean-studio-tools](../../../../roadmap/items/m7-lexlean-studio-tools.md) (#3704). Initial base: `ed9b5b9aa196f086387e8afd6564430adb793716`. The `3a9c2f11d26991a70a22f88758cfb4589a1cf14d` base was ORDER-time history, including registration PR #3730; `195c217783fbecd58db70dd842129e90cff2a69c` and `d43e339502eabf5351c49677929e14c6a8fc5b51` were intermediate merged bases. Current `origin/main` base: `596bf74532beefc8d307936e8bbb35af03a7cb94`, including all non-overlapping main deliveries through that base.

## Owner request and measured decision

Inspect LexLean-R4, use it as the default if possible, and port what works to the web and Tauri clients. Research and an independent architecture review are complete. Repository commit `23436bdcd69e5238fcda51ef2529bbd4a5dc1e73` and deployed tree `563a7549e432fc035bc603504bb3a162eb332212` identify the inspected source. The full hosted 180 MB pack correctly computes `17 + 25 = 42` but answers self-description with a Hunger Games review and the previous-fact question with Fibonacci code. Its older `/8` memory layout, sigmoid tables, heuristic corpus and lack of an OpenAI endpoint make a default/runtime swap incompatible with the current product and charter. This unit therefore selects the narrow repair/shared-tool route; it does not change the default model, product milestone, charter or serving bar.

## Files in scope

- `crates/uor-r4-integer/src/m6_exact_arithmetic.rs` and `src/wasm_bindgen_exports.rs`.
- `src-tauri/src/lib.rs` and focused tests under `src-tauri/tests/`.
- `web/studio/index.html`, `web/studio/assets/js/uor_model_worker.js`, `desktop_openai.js`, `studio_api.js`, `studio_api_streaming.test.mjs`, `model_sources.test.mjs`, and `desktop_openai.test.mjs`.
- `web/studio/assets/js/model_sources.js` — only the existing source-load operation/selection-epoch gating (operation IDs, stale-completion rejection, API-selection invalidation, progress operation IDs and stale-error suppression) exercised by the already-authorized `web/studio/assets/js/model_sources.test.mjs`.
- `web/studio/src/adapters/modelSourceAdapter.ts` and `modelSourceAdapter.test.ts`.
- `web/studio/src/adapters/workflow.ts` (type-only exact-tool marker).
- `web/studio/verify_studio_worker.mjs`, `scripts/studio/verify-tauri-app.sh`, and, only if a command contract changes, existing focused runtime/package tests.
- `docs/products/web-client.md`, `docs/products/desktop-client.md`, `docs/research/notebook/m4-serving.md`, the owning roadmap item, this task folder, and `docs/superpowers/plans/2026-11-05-lexlean-studio-tools.md`.

Scope correction: fresh D68 reviews approved exactly `web/studio/index.html`, `web/studio/assets/js/studio_api.js`, `web/studio/assets/js/studio_api_streaming.test.mjs`, `scripts/studio/verify-tauri-app.sh`, and `web/studio/src/adapters/workflow.ts` as minimal history/package authorization, with the workflow change type-only. The item remains `ready` and the PR uses `References #3704`; no M7, bar, cap, owner-file or broader-path decision is made.

Scope correction: an independent D68 review approved `web/studio/assets/js/model_sources.js` only for the existing source-load operation/selection-epoch gating covered by `web/studio/assets/js/model_sources.test.mjs`: operation IDs, stale-completion rejection, API-selection invalidation, progress operation IDs and stale-error suppression. The ORDER's explicit file list had omitted this module even though the item's outer scope includes `web/studio/**`; the correction adds it explicitly. Callers that omit operation metadata retain prior behavior. This adds no new feature, model runtime, dependency, catalog/default, milestone/bar/cap, owner-file change or broader refactor, and does not implement or alter the exact-arithmetic tool. The item remains `ready`; the PR uses `References #3704` and does not close it. This is a scope decision only, not exact-head or merge approval.

No other paths are authorized. In particular, do not touch `.github/**`, model weights/catalogs, `openai.rs`, `loopback.rs`, sidecar binaries, `Cargo.lock`, or other labs' current training files.

## Done when

- New tests fail first for the intended missing behavior, then pass.
- Strict bounded decimal addition/subtraction produces an exact answer and actual trace through the existing Rust engine. Ordinary text falls through; invalid/underflow cases are explicit.
- The browser worker and Tauri host return the same tool answer with zero model completion tokens; ordinary native and browser generation still work.
- Worker identity mismatches become visible errors; the no-storage synthetic default and display-only catalog primary behavior have regressions.
- Real browser and packaged Tauri runs demonstrate arithmetic and ordinary synthetic responses. Full existing Studio/Node/Rust/runtime/harness/package checks pass.
- Product docs/notebook/REPORT state exact source and negative scope. Protected exact-head review, merge, fresh-main verification and Pages publication complete; generated outputs are removed.

## Cancel probe

Before implementation, run the new shared-tool test against the unmodified code and confirm it fails because no public prompt-tool function exists. If the existing arithmetic engine cannot reproduce the expected values or the Tauri fallback breaks normal streaming, stop and re-scope instead of copying LexLean code.

## Stop and report conditions

No LexLean default, second runtime/model format, external WASM, heuristic corpus, table-form squashing, new dependency, committed binary/corpus, paid compute, remote tunnel, owner-file change, fake token usage, useful-conversation/visual-understanding/M3/M7/M8 claim, or silent weakening of an existing check. If any out-of-scope file is needed, STOP and re-scope first. STOP and report when this unit is delivered; do not start a model-retraining campaign.
