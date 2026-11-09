# ORDER: cpu-training-pilot, deepseek-245 (trained ask-offset audit)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `e883f7281`.

## Goal

Prepare CPU-side support for Claude-330's question-keyed read without waiting for
or modifying its float-training branch.

1. Add a read-only checkpoint audit that consumes a Claude-330 checkpoint's
   `model.safetensors` and `hamming_read.safetensors`, validates the per-depth
   ask tables, and measures for all 16 relation/query pairs whether the trained
   offsets reduce Hamming distance to the target key and make that key the
   nearest of the 16 relation keys.
2. Record a concrete ask-head export/serving contract: representation,
   invariants, refusal behavior, CPU/integer parity and acceptance gates. This
   is a design gate only; it does not add an artifact field or serving path.

The purpose is to diagnose a completed B2 run immediately on the workstation
and identify the downstream serving work before authorizing schema changes.

## Cancel probe

- Claude-330 adds `d{depth}.ask` tensors of shape `(vocab, width)` to
  `hamming_read.safetensors`; its current draft also refuses export because the
  integer engine has no ask-head representation.
- No CPU audit reads those tensors, compares trained and zero offsets against
  target keys, or reports relation-key ranking.
- No registered ask-head artifact or serving contract exists on `main`.

## Design

- Parse checkpoint tensors directly with `safetensors`; bind both input file
  hashes and write a report only to a new path.
- Require ask tables and key rotations at every ask depth, a consistent
  vocabulary/width/head count, even heads of at most 64 quaternion-multiple
  coordinates, and finite offset values.
- For each ask head, depth and relation:
  - query with zero and trained offsets;
  - key with the shared writer point and unchanged key rotation;
  - report target distance, improvement, rank among the 16 relation keys and
    whether the target is nearest.
- Aggregate per depth/head and per depth, including exact code matches, total
  bit improvement, target-nearest counts and changed offset coordinates.
- Document the future integer representation and parity gates. Do not modify
  `fbs_export`, `uor-r4-integer`, serving defaults or Claude files.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-offset-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-245/**`
- `docs/labs/cpu-training-pilot-ask-offset-audit-2026-11-09/**`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Unit tests cover a known offset improvement/regression, nearest-key ranking,
   malformed/missing tensors, hash checks and deterministic output.
2. The release binary builds and a synthetic checkpoint produces a retained
   audit report.
3. The design names the proposed integer representation, fail-closed behavior,
   CPU/integer parity controls and the B2 result required before implementation.
4. REPORT, report check, local policy, full gates and a fresh exact-head review
   pass.

## Stop and ask if

Do not modify Claude-owned files, export an ask-head artifact, add an integer
schema field, run a GPU, train a model, or claim that an offset audit proves B2
or serving readiness.
