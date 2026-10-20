# #761 — Probe the R4G1 graph path in the live quality gate (#750)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T07:09:47Z
- **Updated:** 2026-08-17T07:46:18Z
- **Closed:** 2026-08-17T07:30:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/761
- **Merged:** 2026-08-17T07:30:46Z
- **Merge commit:** 611b767be0cae5710e814dc506365246ec803b94
- **Base / Head:** main / UOR-Foundation:issue-750-r4g1-quality-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Closes the gap #750 named: `evaluate_live_quality` (the `#744` live quality
gate) always built its probe engine with `r4g1_bytes: None`, so `r4 import`
only ever exercised the plain TLA/TLS1 generation path -- even though
`ask`/`chat` transparently prefer a manifest-name-keyed `compiled.r4g1`
graph over the plain path at serving time whenever one exists. The two
paths have been observed to diverge sharply on the same underlying bytes
(one bundle: single-token degenerate repetition under R4G1 vs. word-salad
on plain, both bad but not identically bad), so a manifest could pass the
gate on the strength of one path while still serving degenerate output
through the other.

## Changes

- `chat::engine_from_bytes_with_r4g1(artifact_bytes, store_bytes,
  tokenizer_bytes, r4g1_bytes: Option<&[u8]>, max_tokens)` splits out of
  `engine_from_bytes` (now a thin wrapper), so the gate can build an
  engine bound to either path on demand.
- `model::evaluate_live_quality` gains an `r4g1_bytes: Option<&[u8]>`
  parameter. When supplied, the fixed 8-question probe set runs a second
  time through the R4G1 path via a new `run_live_quality_probe_set`
  helper, and a new `combine_path_quality(plain, r4g1)` function requires
  **both** probed paths to independently pass before
  `instruction_eval_passed` is true -- silently averaging or taking the
  better path would let a bundle degenerate on one path still pass.
- `QualityAttestation` gains `r4g1_grounded_answer_rate` /
  `r4g1_repetition_rate: Option<f32>` fields, `#[serde(default)]` so
  existing manifests deserialize unchanged (no `MANIFEST_SCHEMA` bump
  needed). `None` means "not probed" (no R4G1 graph supplied at import
  time), distinct from a probed-and-degenerate `Some(0.0)`.
- `r4 import` gains an optional `--r4g1 <path/to/compiled.r4g1>` flag
  threading the graph bytes through to the gate. `validate_for_chat` now
  also range-checks the new fields when present.
- `docs/MODEL_LIFECYCLE.md`'s step-6 "Known limitation" paragraph rewritten
  to describe the fix instead of the gap, and its `import` example updated
  to show `--r4g1`.

## Verification

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo run -p xtask -- audit-limits`
- `cargo test --workspace --offline` -- full suite, no failures
- Real-bundle check against `smollm2-1-7b-instruct` (a real local bundle
  carrying a `compiled.r4g1`, using a scratch `UOR_MODEL_STORE` so nothing
  touched the checkout): `r4 import --r4g1 <path>` and a plain-only
  `r4 import` of the identical bytes both correctly reject the bundle via
  `ModelError::QualityGateFailed` -- confirms the gate now catches a
  known-divergent bundle regardless of which path is probed. (This
  particular bundle turned out to already fail on the plain path too, per
  `#745`'s prior word-salad finding, so it doesn't showcase the
  plain-passes/R4G1-fails split specifically -- that exact scenario is
  what the new synthetic falsifier test below covers directly.)

New tests:
- `chat::tests::engine_from_bytes_with_r4g1_actually_threads_the_graph_bytes_through`
  -- confirms the new function actually binds/validates supplied graph
  bytes into the resulting `ChatEngine`, and rejects structurally invalid
  ones.
- `model::tests::combine_path_quality_requires_both_probed_paths_to_pass`
  -- the falsifier this issue's acceptance criteria calls for: a synthetic
  plain-healthy + R4G1-degenerate case must not pass overall; plain-only
  (no R4G1 supplied) passes on the plain verdict alone; both-healthy
  passes; plain-degenerate + R4G1-healthy still fails.

Closes #750

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

