# #95 — Hidden-state trace lane for cover induction (issue #71)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T07:27:19Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-07-23T07:45:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/95
- **Merged:** 2026-07-23T07:45:33Z
- **Merge commit:** 3b5e015d9909992c1dbc8bbfe98b397f87dc4019
- **Base / Head:** main / UOR-Foundation:issue-71-hidden-state-lane
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What was built

Issue #71: the `TeacherOracle::hidden_state()` trace surface is wired as a second cover-induction observation lane beside the deterministic context-bundle lane.

- **`crates/uor-r4-core/src/transformerless/trace_lane.rs`** (new, compiler-side only, `#[cfg(not(target_arch = "wasm32"))]` like its neighbors — nothing touches `runtime.rs`; the P-4 machine-checked scan still passes):
  - `build_trace_observations`: replays the corpus through the oracle in story order (`reset` at story boundaries, one `step` per position) and builds `Observation` rows whose `position` / `sample` / `next` are **identical** to `cover::build_observations` (split, entropy gating, recall, and Gate C align 1:1 across lanes). `vector` = L2-normalized final hidden state; `sig` = sign bits vs 0.0 (the `binarize_prototype` convention — for hidden states there is no thresholds centering; sign vs 0.0 IS the centering; documented in the module docs). **Dimension gate**: v1 requires `source_dimension() == compiler::D` (288, the legacy stories15M teacher); other widths (e.g. the raw 576-dim HF teacher) get a plain error — no silent truncation.
  - `compare_lanes` / `run_compare`: runs ONE fixed region-budget point — the confirmed default `k0=8 / entropy-gain 0.25 / budget 256` (the 42-region #70 default) — on **both** lanes through the lane-agnostic `cover_sweep::run_point` (Gate C routes with each lane's own sigs; the bundle-hardcoded `score --cover` CLI is not used). Writes `lane_compare.json` (schema documented in the module docs) with per-lane rows, the shared TLA3 baseline row (labeled inherently bundle-lane), a recorded lane choice with the adoption rule and justifying deltas, and the macOS-pinned libm-sensitive determinism note.
  - `lane-compare` CLI subcommand (registered in `command.rs` dispatch + usage text).
- **`crates/uor-r4-core/tests/trace_lane.rs`** (new): FakeOracle with planted hidden-state geometry (the 5-group 288-dim sphere pattern) — `induce_cover` recovers the planted regions from trace observations (4 depth-1 regions + the G2 split, ≥0.95 per-group recall); 1:1 alignment with the bundle lane asserted; dimension-gate rejection; byte-identical double-run of the comparison report; adoption-rule edge cases; a checkpoint-gated `#[ignore]` fixture end-to-end test (skip-if-absent convention).
- Minimal supporting change: `#[derive(Clone)]` on `compiler::Corpus` and `compiler::Compiled` (the trace lane's `SweepInputs` shares the loaded inputs).

## The measurement (release binary, fixture corpus, 150k positions / 754 stories, 119,964 train / 30,036 held-out)

Command: `cargo run --release --offline --bin r4 -- transformerless lane-compare --corpus-meta crates/uor-r4-core/tests/fixtures/c_meta.bin --corpus-recs crates/uor-r4-core/tests/fixtures/c_recs.bin --artifacts crates/uor-r4-core/tests/fixtures/tless_artifacts.bin --out /tmp/lane-compare-71` (~10 min under CPU contention).

Per-depth reference recall and frontier width (held-out):

| lane | depth | evaluated | ref-top1 | ref-topM | frontier |
|---|---|---|---|---|---|
| bundle | 1 | 30036 | 78.9% | 93.7% | 2.41/3 |
| bundle | 2 | 30036 | 69.8% | 82.8% | 2.39/3 |
| bundle | 3 | 30036 | 58.9% | 72.4% | 2.44/3 |
| hidden | 1 | 30036 | 80.4% | 92.8% | 2.52/3 |
| hidden | 2 | 30036 | 66.9% | 82.8% | 2.64/3 |
| hidden | 3 | 30036 | 60.5% | 76.2% | 2.54/3 |

Gate C (held-out, Rule 1+2 precedence):

| lane | regions | bytes | R1+2 top1 | bits/token |
|---|---|---|---|---|
| bundle | 42 | 10,867,179 | 31.7% | 9.8612 |
| hidden | 42 | 10,868,395 | **2.8%** | **16.3302** |
| tla3 (shared baseline, bundle-lane class codes) | - | - | 31.7% | 11.8781 |

## Recorded lane choice: **rejected — no gain**

Rule (recorded in `lane_compare.json`): adopt the hidden lane iff its Gate C Rule 1+2 top-1 agreement strictly exceeds the bundle lane's at the same fixed point. The hidden lane's region *geometry* is comparable (deepest-depth reference top-1 +1.6 pts), but its Gate C distortion collapses (top-1 2.8% vs 31.7%, Δ −28.9 pts; bits/token +6.47) — the hidden-state sign codes route the scored graph far worse than the bundle codes. The comparison table above is the rejection evidence.

## Acceptance boxes

- [x] Cover induction runs end-to-end over final hidden states for a fixed observation sample (full pinned fixture corpus, both lanes, same sample).
- [x] Per-lane reference recall and Gate C numbers compared in one table (console table + `lane_compare.json`).
- [ ] Quantized i8 spill for hidden-state vectors — **intentionally unticked**: the lane was rejected by the measurement, so no spill is built (per the issue: "If rejected: the comparison table showing no gain is attached").
- [x] Rejection: the comparison table showing no gain is attached (above, and recorded in the report JSON).

## Gates (all run in the worktree)

- `cargo fmt --check` — clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean
- `cargo nextest run --workspace -E 'not binary(bdd)'` — 289 passed, 11 skipped
- `cargo test --test bdd` — 13 scenarios passed
- `cargo test --doc --workspace` — clean

Determinism note: same-machine double runs are byte-identical (asserted synthetically in `tests/trace_lane.rs`); the f32 hidden states are macOS-pinned / libm-sensitive cross-platform — the inherited status of the κ baseline (D2 resolves cross-platform byte equality later).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

