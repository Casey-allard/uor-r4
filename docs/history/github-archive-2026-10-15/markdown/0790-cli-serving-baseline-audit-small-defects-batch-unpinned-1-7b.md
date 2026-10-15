# #790 — cli/serving: baseline-audit small-defects batch — unpinned 1.7B rev, silent no-op subcommands, default-model resolution, dead serving code, store-path splits

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:47Z
- **Updated:** 2026-08-18T22:00:33Z
- **Closed:** 2026-08-18T22:00:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/790
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

Small, bounded defects found by the 2026-08-18 baseline audit, batched per the AGENTS batch-flow convention (one integration branch, one commit per item; the serving-semantics items get individual PRs per the same convention). Line refs at `aea30bae`.

1. **`uor-r4-cli` menu option 3 (1.7B) uses `HF_REV="main"`** — an unpinned revision the downloader refuses by design (40-hex rule, `src/model.rs:773-776`), so Stage 1 breaks on any fresh machine and violates the pinning policy. Pin the real revision (`uor-r4-cli:97-101`).
2. **`r4 transformerless compare|compare-report` (and ANY unknown `transformerless` subcommand) exit 0 as a silent no-op**, printing a usage banner that itself advertises `compare` (`crates/uor-r4-graph-cli/src/lib.rs:9741-9762`). Make unknown subcommands exit nonzero (the `graph` sub-dispatch already does, `:9673-9675`) and fix the banner. Verified live: exit 0, 2,462-byte banner.
3. **Bare `r4 ask` is broken by default-model resolution**: `latest_descriptor_name` picks the mtime-newest `models/*.json` (`src/model.rs:727-744`), currently the tokenizer-only `t5-base-tokenizer.json` → `compiled model manifest 't5-base-tokenizer' was not found` (verified live). Filter to chat-capable descriptors (has architecture/weights fields) or use a deterministic order.
4. **Dead serving code that the docs used to present as live:** `uor_r4_router::fallback::FallbackRouter` (+ `EngineResponse`, `run_cascade_with_policy`, `AbstainPolicy::Terminal`, the never-constructed `UnmappedRegion` status, and the names `r4g1-graph`/`transformerless-tla5`) has zero callers outside its own tests; `select_synthesis_engine` (`src/server.rs:3688-3698`) is called only by the BDD suite (`tests/bdd.rs:239`) and disagrees with the live `tier_for_engine_name` on `"transformerless"`. Remove both (or repoint the `r4g1_runtime.feature` scenarios at the live resolver so they stop asserting dead behavior).
5. **Unreachable wasm export:** `generate_r4g1_response` (`src/lib.rs:61-65`) is gated on a `wasm_module` global that neither frontend ever assigns (`index.html:2801,2803,3596`; `r4_worker.js:52,54`) — the dashboard's r4g1/transformerless selections always take the geometric fallback in static/WASM mode. Wire it or remove it.
6. **Store-path splits:** the ask-path R4G1 preference hardcodes `.uor-models/...` while the manifest came from `UOR_MODEL_STORE` (`src/chat.rs:198-203` vs `src/model.rs:480`); raw `manifest.name` is interpolated into the path while the manifest read applies `safe_name` (`chat.rs:200` vs `model.rs:715-725`); `last_engine.txt`/`engine_profile.txt` are CWD-relative in both server and chat (`server.rs:3770,:3827`). Resolve all against the store root; sanitize the name.
7. **Silent R4G1 downgrade:** a `compiled.r4g1` that parses at build time but fails at ask time falls through to the plain path with no log (`chat.rs:351`) — after import required both paths to pass. Log it and surface it in the answer metadata (pairs with the decode-path issue's witness item).
8. **`last_engine.txt` writes swallow errors and are non-atomic** (`let _ = fs::write`, `chat.rs:1370-1380, 1712, 2052-2054`) — the CLI reports success while nothing persisted. Check the result; write via temp-file + rename.

## DoD

Each item lands with its own test where behavior changes; items 4–7 as individual PRs (serving semantics); the batch closes with the usual per-item evidence list.

**Next action:** items 1–3 on one batch branch (pure CLI/orchestrator, no serving semantics).


## Comments (2)

### comment 5332682528 — 2026-08-18T18:56:46Z — Casey-allard

**Items 1-3 merged** (PR #796, one commit per item): the 1.7B revision is pinned (`31b70e2e…`, resolved 2026-08-18) in both `uor-r4-cli` and the in-client trigger; unknown `transformerless` subcommands exit nonzero naming the subcommand and the banner no longer advertises `compare`/`compare-report` (verified live: `compare` exit 1, `help` exit 0); bare `r4 ask` default-model discovery skips tokenizer-only descriptors, prefers a locally compiled bundle, and orders deterministically (verified live: bare `r4 ask` now serves smollm2-135m-instruct, exit 0). Falsifier tests for each.

**Item 7 is complete via #785-C3** (PR #797): the ask-time parse failure now warn-logs its downgrade AND the served answer carries a DecodeWitness naming the engine that actually ran — both halves of "log it and surface it in the answer metadata".

Remaining: items 4 (dead FallbackRouter/select_synthesis_engine), 5 (unreachable wasm export — wire-or-remove is a maintainer flavor call), 6 (store-path splits), 8 (atomic engine-file writes) — individual PRs per the convention. Working item 8 next.

### comment 5334580368 — 2026-08-18T22:00:33Z — Casey-allard

**Items 4–8 merged — all eight items done; closing as completed.** (Items 1–3: PR #796, evidence in the previous comment.) Per the DoD, each behavior change carries its own test, and the serving-semantics items shipped as individual PRs.

- **Item 4 — dead serving code removed** (PR #802, `772b68b8`). `uor_r4_router::fallback` rewritten without the zero-caller `FallbackRouter` pipeline (`EngineResponse`, `FallbackResult`, `run_cascade_with_policy`, `AbstainPolicy`, `SERVING_ABSTAIN_POLICY`, the never-constructed `UnmappedRegion`, and the stale `r4g1-graph`/`transformerless-tla5` names all gone); `run_cascade` now implements record-and-continue directly with exports trimmed to what serving actually uses. `select_synthesis_engine` removed from `src/server.rs`; the BDD scenarios repointed at the live resolver via `default_resolved_tier` — the suite stopped asserting dead behavior (the legacy expectation is now honestly `Some("transformerless")` against the live map).
- **Item 5 — wasm export WIRED** (maintainer decision: wire, not remove; PR #805, `5a4175b0`). New `#[wasm_bindgen] set_r4g1_bundle(graph, tokenizer)` export in `src/lib.rs`; both frontends now assign the `wasm_module` global at INIT (`r4_worker.js`, `index.html`); static/WASM mode attempts a real bundle install (`./graph/score.r4g1` + `./tokenizer.bin` → `set_r4g1_bundle`) and logs an honest geometric-fallback message when the bundle is absent — the dashboard's r4g1/transformerless selections reach the real engine instead of silently taking the geometric path.
- **Item 6 — one store root** (PR #803, `51e88efe`). `src/model.rs` gains `model_store_root()` (`UOR_MODEL_STORE` → `.uor-models`), `store_state_file()`, and `compiled_model_dir()` (applies `safe_name`); the ask-path R4G1 preference, the CLI state files, and the server state-file reads all resolve through them. The `chat.rs:200` raw-`manifest.name` interpolation is gone (sanitized like the manifest read that produced it). Tests pin one root + sanitized names and that tokenizer-only descriptors are skipped.
- **Item 7 — silent R4G1 downgrade named** (PR #797, `ab7b3bce`, alongside #785-C3). The compiled-graph parse failure at ask time now warns loudly and the served answer's `DecodeWitness` names the actual serving engine — the downgrade is visible in both the log and the answer metadata, exactly as the item asked (paired with the decode-witness work by design).
- **Item 8 — atomic, loud state-file writes** (PR #799, `8f560156`). `persist_state_file` writes temp-file + rename and propagates errors; the CLI no longer reports success while nothing persisted. Test covers atomicity and the loud-failure path.

Board effect: the audit's small-defect batch is fully cleared, including the audit's CWD-relative note for `last_engine.txt`/`engine_profile.txt` — both now resolve against the store root (`store_state_file`), so `UOR_MODEL_STORE` is honored end to end. (The stale CWD clause in `docs/CONFIGURATION.md`'s table is corrected in the #789 implementation PR, which touches that row anyway.)



---
_Comment/review ids in this file: 5332682528, 5334580368_

