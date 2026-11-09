# ORDER: cpu-training-pilot, deepseek-246 (CPU ask-offset solver)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `dfb2a5d3b`.

## Goal

Train Claude-330 ask offsets offline on the CPU from a checkpoint's shared
writer table and key rotations. Given a deterministic corpus of
`(question token, target key token, weight)` pulls, optimize one ask row per
depth and token against the exact rotated sign-code Hamming objective.

Emit a checkpoint-compatible `hamming_read.safetensors` prototype containing
all original Hamming tensors plus `d{depth}.ask`. This is an optional
initializer experiment, not an artifact-schema or serving change.

## Cancel probe

- Claude-330 makes `d{depth}.ask` a per-token `(vocab, width)` F32 table.
- Found-copy pulls depend only on token ids, the detached writer table and key
  rotation; no trunk forward is required to optimize them.
- The CPU checkpoint audit exists on `main`, but no CPU trainer produces ask
  rows or a complete candidate Hamming sidecar.
- The default corpus is the fixed B2 relation `K -> Q(K)` mapping; arbitrary
  pull corpora may include repeated and multiple targets per question.

## Design

- Parse `embedding.weight`, `heads`, and every
  `d{depth}.key_rotation` from a Claude-330 checkpoint.
- Read pulls from an optional JSON schema; default to the 16 B2 relations.
- For each question/depth, evaluate:
  1. zero offset;
  2. exact offset to every distinct target key;
  3. a weighted coordinate median toward all targets;
  4. deterministic bounded coordinate search from the best few starts.
- Objective: weighted sum of Hamming distance to each target key across ask
  heads, with stable token/coordinate tie order.
- Reject empty/invalid pulls, zero widths, wrong shapes, partial depths,
  non-finite values, non-finite sums/rotations and output replacement.
- Preserve all original Hamming tensors and metadata; add/replace only ask
  tables. Report before/after weighted bits and output hashes.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/ask_offset_solver.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-offset-audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-offset-solver.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-246/**`
- `docs/labs/cpu-training-pilot-ask-offset-solver-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Unit tests cover exact solving, multiple targets, deterministic output,
   malformed pulls/checkpoints and non-finite failures.
2. A synthetic checkpoint and B2 mapping produce a candidate Hamming sidecar.
   Its report and hashes are retained; the generated binary is not committed.
3. The existing ask-offset audit accepts the candidate and shows the expected
   distance change.
4. REPORT, report check, local policy, full gates and a fresh exact-head review
   pass.

## Stop and ask if

Do not modify the checkpoint in place, Claude-owned files, artifact serving,
training defaults or any GPU path. Do not claim the solved table is a trained
ask-head model or B2 result.
