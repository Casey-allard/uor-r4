# Completion scoring readiness — 14 October 2026

This is a receipt-only rehearsal against the preserved A2 pilot, plus source review
of frozen D5. No model initialization, carry, forward, evaluation, training, GPU
context, pod, owner record or release is performed. No model source, numeric
instrument, registered bar, initializer or schedule changes. Claude owns the
separate qualified completion build; this record does not qualify that binary.

Line: evaluation integrity; count 1/3 unchanged; no new model result or headline.
The owner assignment is completion-scale scoring readiness without model compute.

## What gets scored and when

The final **zero-based batch index is 16,403**. After that batch the saved cursor
is **16,404 completed batches**, and the final evaluation is
`evaluation-16404.json`. There is no registered `evaluation-16403.json`.

| Completed batches | Last processed batch index | Vector attempts | Route attempts | DEV rows |
|---:|---:|---:|---:|---:|
| 4,101 | 4,100 | 4,101 | 0 | 14,287 |
| 8,202 | 8,201 | 8,202 | 0 | 14,287 |
| 12,303 | 12,302 | 12,303 | 0 | 14,287 |
| 16,404 | 16,403 | 16,384 | 20 | 14,287 |

D5 saves the checkpoint before scoring each registered cursor. The final report
reuses that final evaluation instead of running the DEV panel again. Every
`PreparedDriver::evaluate` starts DEV traversal at zero: 223 batches of 64 plus
a final batch of 15, independent of the training cursor. The 16,404 training
batches do not turn into 16,404 DEV batches. Seed 1 preserves its original pilot
checkpoint-zero baseline; seed 2 measures its own baseline under its own scope.

Source reviewed: `crates/uor-r4-discrete/src/completion.rs` (CHECKPOINTS,
score/read_evaluation and run), `training.rs` (PreparedDriver::evaluate),
`corpus.rs` (batch boundaries), `device.rs` (receipt serialization/aggregation),
and `discrete.cu` (evaluation and checked integer accumulation). Discrete source
must remain byte-identical to Claude-approved `33564b75e`.

## B3 and guard accounting

Each row already contains the necessary GPU measurements:

```
target_admitted = admitted_cells - address_rivals  // 0 or 1
strict_win = token_rivals > 0 && token_hinge < 32
admitted_strict_win = strict_win && target_admitted == 1
```

The scorer checks the row, batch and entire-panel sums and the nesting:
`admitted strict wins <= top-1 hits <= top-8 hits <= target-admitted rows`.
The strict-win guard itself includes omitted targets and is not a writer-hit
count. The frozen loss is `8 * token_hinge + 32 * address_hinge`, with denominator
256 per row. Rank bars and guard comparisons use integer/rational arithmetic.
Intermediate and single-seed diagnostics do not manufacture a two-seed KEEP or
fresh-transfer result.

**Avoided scoring defect:** the pilot occupied enough cells to select eight in
every row. A completion may legally occupy fewer cells. The general scorer must
validate actual cell counts bounded by K=8, not require `admitted_cells == 8`.
This changes no model behavior and avoids misclassifying a lawful changed codebook
as an instrument failure.

## Artifact and recovery boundary

The original pilot report is SHA-256
`e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`
and contains complete baseline and final per-row receipts. Its checkpoint-1640 is
`46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440`.
Both come from the MD5-verified archive
`icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar`.
No completion-trained artifact exists in this rehearsal, and no pilot measurement
is relabeled as a completion checkpoint.

D5's production recovery checks exact immutable evaluation and checkpoint FilePins,
cursor/epoch, lineage, count reference and sensitivity counters before resumed
apply. The final report contains links to the four quarter results, not their
inline payloads. **Keep the complete run-state and every referenced attempt
checkpoint together at their pinned paths**, and retain the original seed-1
parent; moving only `report.json` is not a self-contained recovery archive.
The prior first-save edge remains: an interruption before a latest pointer exists
can require a new run-state/plan/personal release. Claude recorded it as
non-blocking because only the pre-run baseline is exposed.

Stored per-batch receipts omit their in-memory epoch/binding fields, so the
external scorer can validate the evaluation-level epoch and source bindings but
cannot independently recover an omitted per-batch epoch. Stored rows have no
explicit target IDs: row pairing follows the pinned deterministic DEV traversal.
These are evidence limits, not silently invented provenance.

## Scope of readiness

The standalone Rust scorer reads saved JSON and performs integer receipt arithmetic.
The frozen crate's default-feature metadata tests exercise its actual metadata
validators. **Neither executes D5's CUDA-gated score/run branch.** Source continuity,
metadata rehearsal and prior pilot GPU evaluation do not substitute for Claude's
new CUDA build/identity qualification or measure future occupancy-dependent GPU
latency. No arbitrary latency threshold is used as a model gate.

Working-storage planning remains approximately 0.8 GB of checkpoints per complete
seed, plus evaluations, steps and logs. The measured pilot report is 5,718,475 bytes
for two full evaluations; the checkpoint is 3,138,466 bytes. Repeated baseline data
compresses in archives but still occupies uncompressed pod working storage.

## Rehearsal result and cost

The retained [pilot rehearsal](pilot-rehearsal.json) checks both complete stored
evaluations, not a reduced sample. It reproduces top-1 **17 → 82**, top-8
**173 → 1,169**, target-admitted rows **5,680 → 2,180**, and strict wins
**105 → 149**. Those strict wins split as admitted **0 → 36** and omitted
**105 → 113**. The loss-numerator increase is **68,000**: token contribution
**−63,968**, address contribution **+131,968**. These remain archived pilot
observations, not completion results. The registered integer thresholds are
1,941 top-1 hits, 4,775 top-8 hits and, for seed 1, 525 strict wins.

One optimized replay on the local arm64 machine (Rust 1.97.1) took **1.04 s wall**,
0.19 s user and 0.03 s system time, with **76,349,440 bytes maximum RSS**. Internal
phase timings were parse 190,661,375 ns, validate 647,923,041 ns and write
6,068,750 ns. This is a single observed host receipt-processing cost, not a GPU
evaluation estimate, repeated-run benchmark or latency acceptance bar. The release
build took 5.21 s. No pod rental or model work was incurred.

Validation: **12 scorer tests passed** (2.19 s test time), plus **25 existing D5
default-feature metadata tests passed**. The additional completion-parser test
uses the actual archived checkpoint bytes in an explicitly synthetic in-memory
metadata wrapper; baseline, cursor, model checksum, epoch, counter and P2 mutations
are refused. It does not write a fabricated completion artifact. A repeated output
path is refused with exit 1, and its existing digest is unchanged. D23/D25 ratchets,
formatting and source/diff checks pass. The independent source review approves the
receipt-consistency scope; it is not a CUDA or TEST FITNESS verdict.

| Identity | SHA-256 |
|---|---|
| Scorer source | `0fe2bde7a4f24fbc2a4f45291a91e793c110a3c0626ddac40a57b46686b6919e` |
| Local optimized executable | `a33f77f3a0acd9249ce43f64b02963a36fad66792eba037c7ef9d5660f273276` |
| Retained pilot rehearsal JSON | `b543dbf48df839d523c65644c9c52869ac7e1f29a38c646f1ce4b1a1beafa03e` |

## Operator handoff

Build in an owned worktree; the scorer is a standalone Cargo package with only
JSON and hashing dependencies. It does not link the model or CUDA runtime.

```sh
export CARGO_TARGET_DIR="$PWD/local/scoring-target" CARGO_BUILD_JOBS=4
cargo build --release --locked \
  --manifest-path docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/Cargo.toml

# Rehearse existing evidence. Output must not exist.
"$CARGO_TARGET_DIR/release/completion-scoring-audit" pilot \
  "$PILOT_REPORT" "$NEW_REHEARSAL_JSON"

# Once the admitted GPU driver has produced a real saved evaluation:
"$CARGO_TARGET_DIR/release/completion-scoring-audit" completion \
  "$SAVED_EVALUATION_JSON" "$ORIGINAL_BASELINE_JSON" \
  "$CANONICAL_SEED_PLAN_JSON" "$NEW_SCORING_JSON"
```

For seed 1, `ORIGINAL_BASELINE_JSON` is the exact pilot report or checkpoint-1640
listed above (both retain checkpoint zero's evaluation). For seed 2, use that
seed's original baseline. `CANONICAL_SEED_PLAN_JSON` is the corresponding
`SEED1_PLAN.json` or `SEED2_PLAN.json` from
[`rung0-completion-release-2026-10-14`](../rung0-completion-release-2026-10-14/README.md).
The scorer checks the canonical numerical plan and permits its build identity to
be bound separately; it neither authenticates nor creates an owner release.
Run it where the evaluation's referenced checkpoint FilePin path is available.
Use `evaluation-4101.json`, `evaluation-8202.json`, `evaluation-12303.json`, then
`evaluation-16404.json`; do not substitute a prior quarter for the final result.

The scorer verifies the referenced checkpoint's byte digest and metadata, model
epoch/bindings and stored sorted-JSON model checksum. The production typed inner
checkpoint and evaluation snapshot hashes remain the production validator's
responsibility, as does independent authentication of seed 2's original baseline.
The JSON output states these limitations, `authorizes_model_compute: false`, and
whether it is archived-pilot, intermediate or single-seed-final evidence.

**Next:** Claude retains completion build/identity qualification and execution
ownership. Kimi can use this scorer and accounting definition when the real
registered checkpoint receipts arrive. No further Codex scoring review is pending;
this delivery changes no training authorization, sheet bar or mechanism successor.
