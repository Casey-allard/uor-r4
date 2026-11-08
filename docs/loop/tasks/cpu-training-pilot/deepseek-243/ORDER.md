# ORDER: cpu-training-pilot, deepseek-243 (first-reply Hamming weight trace)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `229739cf9`.

## Goal

Extend the merged first-reply Hamming trace from positional membership to exact
selection strength. For every kept position, record its Hamming distance, its
head's Q16 margin, and its Q31 margin weight. Aggregate those values over the
asked and paired-other value spans so the result distinguishes positions that
are merely retained from positions that contribute nonzero weight to a head.

This is a read-only CPU diagnostic supporting Claude's query-recall track. It
does not train or modify an artifact, change serving arithmetic, touch Claude's
open branch, or claim that one retained position caused the final token.

## Cancel probe

- Deepseek-242 established that every read/head retains at least one position
  from both value spans in 873-992/1,024 rows, but discarded the candidate
  distances returned by `nearest` and the head margins returned by the artifact.
- `crates/uor-r4-cpu-train/src/hamming.rs` already computes the exact serving
  weight with `copy_weight_q31(margin_q16, distance)` for both copy and type
  reads. It currently exposes neither the kept distance nor the resulting
  weight outside the private mean calculation.
- `CpuFbsForward` already executes the exact integer Hamming path and can be
  instrumented without changing its output arithmetic.
- The item's broad scope overlaps an unrelated stale same-lab worktree only at
  `cpu-point-transfer-driver.rs`. This unit does not touch that file; its
  narrower files are disjoint.

## Design

1. Add an opt-in Hamming read trace to `CpuHamming` and `CpuFbsForward`.
   Recording is off by default. Each traced head reports its kind, Q16 margin,
   and every kept position with distance and Q31 margin weight.
2. Invoke tracing only on the final prompt-token step after reply start. Compare
   its read/head/kept-position shape with the merged integer-session trace and
   fail closed on any difference before reporting weights.
3. Extend `cpu-first-reply-hamming-trace` schema to v2. Preserve the existing
   chosen-token, span-membership and exact-hash controls, and add per-row
   distance/margin/weight evidence plus per-head/per-stratum aggregates.
4. Report rows and positions with asked-only, other-only, both and neither
   nonzero-weight support; total asked/other weight; zero-weight kept positions;
   and the first-token subset.
5. Run both Claude-322 seeds under both registered renderings and record the
   same input, artifact, executable, determinism and resource receipts.

## Files in scope

- `crates/uor-r4-cpu-train/src/hamming.rs`
- `crates/uor-r4-cpu-train/src/fbs.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-first-reply-hamming-trace.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-243/**`
- `docs/labs/cpu-training-pilot-first-reply-hamming-weight-2026-11-08/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. The opt-in trace is exact against the integer session's kept positions and
   leaves untraced CPU forward outputs byte-identical. Tests cover copy and type
   distances, margins, zero and nonzero weights, trace isolation, fail-closed
   shape mismatches, span tallies and schema v2.
2. Both sealed artifacts run on all 256 panel rows under both registered
   renderings without training.
3. The REPORT states whether zero-weight retention explains the failed opening,
   with per-head and per-stratum evidence, and does not infer causality from a
   retained or weighted position.
4. REPORT, notebook row, report check, local policy, full gates and a fresh
   exact-head review pass.

## Stop and ask if

This is attribution evidence, not a capability result. Do not alter a forced
prefix, train a model, modify Claude-owned files, claim that a weighted position
proves selection, or substitute this trace for Claude-328's trained query-recall
probe.
