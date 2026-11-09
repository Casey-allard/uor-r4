# ORDER: cpu-training-pilot, deepseek-244 (ask-head CPU oracle)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b2caa330e`.

## Goal

Build an isolated CPU oracle for Claude-330's question-keyed ask-head geometry.
The oracle will use the sealed Claude-322 integer artifacts' shared writer
table and key rotations, reproduce the ask query's zero-offset equivalence to
a copy query, optimize a shared per-token ask offset toward the found-copy
target key, and report whether the target value position enters the kept set.

This is a CPU development-research support unit. It does not alter Claude's
float-training branch, define the final artifact schema, choose CUDA behavior,
or claim a B2 pass. Its purpose is to provide an exact local target for the ask
query and found-copy geometry before GPU training results are available.

## Cancel probe

- Claude-330 has an open draft ORDER for ask heads and a found-copy pull, but
  no implementation exists on `main`.
- The stable inputs to the oracle exist today: the sealed Claude-322 `UORLUT01`
  artifacts expose the writer table, Hamming key rotations, head margins and
  kept count.
- The existing CPU Hamming port already constructs the same sign codes and
  nearest-k selection. It does not yet expose the arbitrary-point ask query or
  the shared offset table.
- No CPU binary tests the question-keyed query formula independently of the
  float trainer or estimates how much a shared ask offset can reduce target-key
  Hamming distance across depths.

## Design

1. Add a CPU-only ask-head oracle around the existing integer Hamming tables.
   It will:
   - construct each ask query as the sign code of
     `rotate(key_rotation, token_point + ask_offset[token])`;
   - use the same earlier-position key codes and nearest-k tie rule as the
     served copy/type heads;
   - prove bit-for-bit copy-query equivalence at zero offset.
2. Add a deterministic shared-offset fitter for the found-copy target. For each
   Claude-330 query token `Q(K_j)`, minimize total Hamming bits to `K_j`'s key
   across every ask head and depth with bounded coordinate search. Report the
   starting and ending distances and iterations.
3. Build a bounded synthetic B2-shaped probe from real artifact points: one
   earlier `K_j, v_j` fact, filler positions, and `Q(K_j)` at the query row.
   Compare target-position kept membership before and after offset fitting.
4. Run both Claude-322 seed artifacts and all 16 question/key relation pairs.
   Record exact hashes, per-depth/per-head distances, fit cost and resources.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/hamming.rs`
- `crates/uor-r4-cpu-train/src/ask_head.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-ask-head-oracle.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-244/**`
- `docs/labs/cpu-training-pilot-ask-head-oracle-2026-11-08/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Unit tests cover zero-offset copy equivalence, exact key targeting,
   nearest-position retention, deterministic offset fitting, malformed input
   refusal and artifact hash checks.
2. Both sealed artifacts run over all 16 relation/query pairs without training
   a model or changing serving.
3. The REPORT records starting/ending key distances, retained target positions,
   fit iterations and resource cost without calling this a B2 or capability
   result.
4. REPORT, notebook row, report check, local policy, full gates and a fresh
   exact-head review pass.

## Stop and ask if

Do not modify Claude-owned files, infer an artifact field that Claude has not
defined, train a model, use a GPU, or claim that a synthetic retained position
proves B2. If the ask formula cannot be implemented from current public integer
tables, stop at the parity kernel and name the missing interface.
