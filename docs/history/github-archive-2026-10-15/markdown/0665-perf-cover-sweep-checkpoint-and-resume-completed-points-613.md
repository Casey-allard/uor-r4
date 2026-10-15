# #665 — perf(cover-sweep): checkpoint and resume completed points (#613)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T16:14:45Z
- **Updated:** 2026-08-13T16:34:09Z
- **Closed:** 2026-08-13T16:34:09Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/665
- **Merged:** 2026-08-13T16:34:09Z
- **Merge commit:** 0c09dba8789d722ecb2de736bca718df7207bf1f
- **Base / Head:** main / UOR-Foundation:i613-checkpoint-resume
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
A long or interrupted sweep lost all progress on failure. This persists each completed point atomically so a re-run **reuses valid points** instead of recomputing them. Opt-in and off by default: with `R4_SWEEP_CHECKPOINT_DIR` unset the behavior is byte-identical to before.

## What's included
- Each completed point writes `point-NN.json` via a **tmp-file + atomic rename**, so a reader only ever sees a complete file and an interrupted write leaves a stray `.tmp` that resume ignores.
- Each file carries a **`CheckpointKey` manifest** — schema, artifact/corpus κ, scorer fingerprint, source revision (`R4_SWEEP_SOURCE_REV`), Gate C sampling (`R4_GATE_C_SAMPLE`) and skipped arms (`R4_GATE_C_SKIP_ARMS`) — plus the point label and its full cover-config identity. A point is reused **only** when the file fully deserializes **and** its key, label, and cover all match the current run; any mismatch, missing, or corrupt file recomputes the point.
- The final report is assembled only once every point is present (`run_sweep` needs all rows), so a half-finished dir never yields a report.
- Wired into `run_points_bounded`'s per-point closure, so resume composes with the #612 bounded parallelism (distinct per-point files, no write contention).

## Byte-identity fix (`float_roundtrip`)
serde_json serializes f64 with the shortest round-tripping decimal, but its **default parser is not correctly rounded** and can land 1 ULP off — so a metric written to a checkpoint and read back diverged a resumed report from a clean one by a single ULP (e.g. `reference_topm 0.9333040894861723 → …724`). Enabling serde_json's `float_roundtrip` feature makes `parse(serialize(x)) == x` bit-for-bit. **No new dependencies** (`Cargo.lock` unchanged).

## Acceptance criteria
- Resume without rerunning valid completed points ✔ (measured; reuse proven by a timing sentinel).
- Partial files cannot be mistaken for completed results ✔ (atomic rename; corrupt/`.tmp` ignored — unit test).
- Any input/scorer/schema/cover mismatch invalidates the checkpoint ✔ (unit test).
- Resumed output byte-identical to a clean serial run ✔ (measured — after the `float_roundtrip` fix).
- Final report emitted only after all required points complete ✔.
- Default behavior safe for existing callers ✔ (opt-in; default unset = pre-#613).

## Verification (measured, real fixtures)
- **Resume parity** — new `#[ignore]` test runs a clean sweep (populating all nine checkpoints), a **full** resume, and a **partial** resume (half the points copied). Each resumed report equals the clean run's **except** the timing block (`graph_kappa`, the artifact's blake3, is compared, so equal reports prove byte-identical artifacts). Reuse proven by a timing sentinel that only survives if the point was loaded, not recomputed. `clean == full-resume == partial-resume`.
- **Checkpoint unit test** (CI, no fixtures): reuse only under a matching manifest/label/cover; partial/corrupt/interrupted (`.tmp`) writes never trusted; round-trip is bit-exact.
- **Local CI parity, all green:** `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4 — nothing deferred), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Breaking changes
None. `R4_SWEEP_CHECKPOINT_DIR` unset (default) is byte-identical to pre-#613 behavior; `float_roundtrip` only makes float parsing more accurate.

Closes #613

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

