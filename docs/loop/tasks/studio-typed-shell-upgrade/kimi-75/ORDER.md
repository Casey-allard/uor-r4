# ORDER: repair acceptance identity and source serving, kimi-75

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base/reviewed head: `73f7529675aa45885ee32d698ec3cccf6d452e20`. Fresh reviewer agent-59 returned CHANGES; PR #3879 remains draft.

## Findings and decision review

The limited host implementation has evidence but cannot yet be delivered honestly. Top-level records must bind the sole GUI acceptance to `42bc46f81`/`c8ddeb4039…`, not the delivery head. Reopen must assert exact session identity and complete transcript. Loopback evidence must bind HEAD/tree/host/dist hashes. The active report must name executed checks without stale pending/final claims. TypeScript, TSX and source maps must never be served from any static root; explicit `--studio-root` needs a present canonical directory, not silent fallback.

## Files in scope

- `src-tauri/src/lib.rs` only for static source/map refusal and focused tests; `src-tauri/src/main.rs` only for explicit studio-root validation and focused tests; `src-tauri/tests/desktop_client_controls.rs` for root/refusal regressions.
- `web/studio/verify_studio_loopback.mjs` for exact session/transcript and source/host/dist evidence. Do not edit engine/worker/tokenizer/model/API implementations.
- `docs/products/{desktop-client,web-client}.md`, `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `web/studio/README.md` for exact identity and current measured counts.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-75/**` and ignored evidence under `local/kimi/**`.

No dependencies, application/model/API behavior beyond the named source-serving guard, real endpoints/profiles/credentials, production publish, package rebuild, or full-parity claim. Earlier reports remain history.

## Done when

- Regression tests fail before the serving repair and pass after it, including renamed roots, default source root, missing/invalid `--studio-root`, TS/TSX/maps, and valid assets. Complete default/GUI suites pass.
- Reopen evidence compares active session ID and the full message array; head/tree/host-binary and deterministic dist hashes are recorded beside a fresh passing loopback run. Old evidence remains byte-identical.
- Each named record distinguishes historical GUI evidence from the current source checks and later contract-only package builds. The active REPORT records exact source/test identities, all failures, and remaining delivery steps.
- Typed/plain-JS/build/runtime checks, full repository gates, local policy, fresh exact-head review, GitHub checks, protected merge, and fresh-main verification complete without claiming full parity, production, signed release, or useful model behavior.

## Stop and report

Stop before weakening a security boundary or existing assertion, editing the R4 API/model, treating a substring as exact transcript evidence, or hiding an old/new artifact mismatch. Minor review notes about CLI approval ordering, notebook historical wording, the separate read-only browser query, and later package-build presentation are deferred unless required by the critical/important repairs and must remain in the REPORT.
