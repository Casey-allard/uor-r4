# #625 — rebase(UOR-Atlas-UTQC): port κ-injectivity census discipline + template gates — no crate dependency

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T23:02:39Z
- **Updated:** 2026-08-12T00:43:07Z
- **Closed:** 2026-08-12T00:43:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/625
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 7: UOR-Atlas-UTQC — last, least directly relevant; hologram's substrate, evaluated per the tracker note).

## Audit result (measured 2026-08-11)

Not integrated. Upstream v0.1.3 = main HEAD (2026-07-19). Nine `tqc-*` crates realizing a modular-tensor-category on a content-addressed substrate. **Not standalone**: `tqc-substrate` (the only crate touching content addressing) drags `holospaces` + six `hologram-*` git crates plus crates.io `uor-addr 0.2.0`/`uor-foundation 0.5.2` — which would land under our #618/#619 `[patch.crates-io]` type-identity arrangement with unverified build compatibility. The MTC/braid/QASM crates (`tqc-mtc`, `tqc-algorithms`, `tqc-compiler`) have no r4 surface.

**Verdict: port the discipline, not the crates.** Atlas and r4 share the same repo-template lineage (r4's Justfile marks the #273 template adoption explicitly; cucumber 0.21, model/*.toml typed registries, honesty gates in both) — atlas is the *fuller* instance and the reference for finishing what #273 started (#273 currently on-hold, team Phase 0; this issue does not reopen it, it inventories what's portable so #273 has a concrete menu when the team activates it).

## Transplantable pieces (no new dependencies)

1. **κ-injectivity census discipline** (tqc-vv `exact.rs` ~2994–3042, ~3198–3420; this is upstream's HEAD-commit hardening): wherever r4 reports "N distinct X" by counting distinct κ (cover sweeps, region censuses, distinct-graph counts in graph-certify / cover_sweep.rs), count distinct **canonical bytes** as the primary number, compute κ separately, assert equality — the count becomes hash-independent and collision-freeness becomes a *verified property* instead of an assumption.
2. **Two-path κ certificate** (`canonical_kappa_certificate`, exact.rs ~3337): an independently-built canonical serialization must reproduce the production addresser's κ. Cheap second witness for our determinism invariant.
3. **Cheap gates for xtask/Justfile**: `anti-hardcode` (grep-gate forbidding literal verdict fields and `assert!(true`), `oracle-verify` + `PROVENANCE.md` for our pinned teacher/corpus κ artifacts, `portability` as an explicit gate (we already build wasm32), workspace `[workspace.lints]` deny posture (`unwrap_used`/`expect_used`/`panic`/`todo`).

## Deferred / rejected (recorded)

Any crate-level atlas dependency (esp. `tqc-substrate` — hologram git deps are outside deny.toml's source allowlist; if we want uor-addr composition on κ we call uor-addr directly at our pinned rev); `tqc-model`/`tqc-core` adoption (repo-model already owns our vocabulary); the LaTeX `paper-check` chain and the **online** `atlas-pin-check` gate (would add a network dependency to CI); blake3 transitive question (atlas lock has 1.5.5 vs our `=1.5.0`) — moot unless a crate dep is ever taken, resolve before not after.

## Deliverables

- [ ] κ-injectivity + two-path certificate retrofit filed as a concrete implementation issue against graph-certify/cover_sweep census sites (assign on start per board discipline)
- [ ] Gate menu (anti-hardcode / oracle-verify / portability / lints) posted to #273 as input for the team's Phase 0 template work — Alex/Ari governance, filed unassigned
- [ ] This issue closes with the port/defer record; no atlas crates enter the graph

## Verification

Any retrofit PR: four gates + suite green; census numbers unchanged (the retrofit adds an assert, not a new count); no dependency-graph change.


## Comments (1)

### comment 5260728834 — 2026-08-12T00:43:06Z — Casey-allard

## Deliverables complete — closing with record (2026-08-12)

**Verdict from the audit, executed: port the discipline, not the crates.** No atlas crate enters r4's graph — `tqc-substrate` (the only content-addressing surface) drags `holospaces` + six `hologram-*` git dependencies (outside deny.toml's source allowlist) plus crates.io `uor-addr`/`uor-foundation` that would land under the #619 `[patch.crates-io]` type-identity arrangement with unverified build compatibility; the MTC/braid/QASM crates have no r4 surface. If r4 ever wants uor-addr composition on κ, it calls uor-addr directly at our pinned rev. The blake3 question (atlas lock 1.5.5 vs our `=1.5.0`) is moot with no crate dep and is recorded here for the day one is proposed.

### Delivered

1. **κ-injectivity census retrofit filed as #628** (linked as a sub-issue here; unassigned per board rules — assign on start): canonical-bytes counts primary + κ-equality asserted + two-path κ certificate, with the must-be-able-to-fail verification requirement in the body.
2. **Template gate menu posted to #273** (Alex/Ari governance, filed as input not decision): anti-hardcode grep gate, oracle-verify + PROVENANCE.md, portability-as-a-gate, workspace lints deny posture; paper-check and the online atlas-pin-check explicitly not recommended for r4 CI.
3. Version/relevance record: atlas v0.1.3 = upstream HEAD (2026-07-19); relevant to r4 as a discipline reference and template exemplar, not as a dependency — consistent with the #589 tracker's original note that it "may not necessarily be completely relevant."

No code changed under this issue; gates unaffected; zero-matmul runtime untouched. Deliverables check: retrofit issue filed ✓ · gate menu to #273 ✓ · closes with port/defer record, no atlas crates in the graph ✓.


---
_Comment/review ids in this file: 5260728834_

