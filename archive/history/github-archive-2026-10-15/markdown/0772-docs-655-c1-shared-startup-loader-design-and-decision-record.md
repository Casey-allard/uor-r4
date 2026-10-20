# #772 — docs(#655-C1): shared startup loader design and decision record

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T20:13:22Z
- **Updated:** 2026-08-17T20:35:35Z
- **Closed:** 2026-08-17T20:34:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/772
- **Merged:** 2026-08-17T20:34:30Z
- **Merge commit:** 42d7aa1af8332632a4a0c53467a69d626b58dd93
- **Base / Head:** main / UOR-Foundation:docs/655-c1-shared-loader-design
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Design/decision record for #655-C1 (the shared startup loader named in
#655's own A-F decomposition). Docs only — no server/CLI code changes.

Answers the three "Next open questions for #655-C1" left open by
`docs/SERVING_MODEL_DISCOVERY.md` (#655-C1a, PR #739):

- **Q1** — scope the loader to the R4G1 tier only; tiers 2-4 keep their
  current ad hoc discovery unchanged.
- **Q2** — keep `ModelStore` separate for this phase; migrating it to
  `ReleaseBundleManifest` is a breaking compatibility change for existing
  local `.uor-models` stores, deserving its own explicit decision
  (deferred to a future C1e, not decided here).
- **Q3** — the loader must tolerate a missing manifest as the *common*
  case: no shipped bundle produces one today. New finding while tracing
  this: `uor-r4-api::compile` (the pipeline that produces exactly the
  digest shape `ReleaseBundleManifest` wants) is currently exercised only
  by its own crate's test suite — not called from any live CLI or server
  path. Proposes an additive verify-if-present sidecar check layered on
  top of today's existing R4G1 discovery (`resolve_loadable_compiled_bundle_with_authority`),
  so the next slice (C1c) is buildable and unit-testable now, without
  blocking on or being blocked by #655-D.

Proposes a slice sequence for follow-up PRs: C1c (additive sidecar-verify
helper, no behavior change for any bundle that exists today), C1d (wire
`verified` into `/v1/models` once #655-D ships a real bundle), C1e
(`ModelStore` reconciliation, explicitly deferred as its own issue).

## Verification

- `cargo fmt --check` — clean (docs-only change, no Rust files touched)
- Full `cargo build --workspace` / `clippy --all-targets --all-features -D
  warnings` / `cargo test --workspace --offline` already re-verified clean
  against this exact `origin/main` base commit (`eb75cd50`) in the
  sibling `chore/add-fetch-simple-wiki-script` PR (#771), landed minutes
  before this one — not re-run here since this change touches no Rust
  source.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

