# ORDER: cpu-training-pilot, deepseek-90

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e2470ec4d09e873352df128133116f73d3ababda`.

## Goal

Close the missing independent CPU read path for the public trained 19-table
M1 artifact. The existing CPU reader covers the four base tables and the
independent writer, but the 15 post-rotation joint-pointer tables are absent.
That leaves full CPU-versus-serving parity unavailable on the project's only
public trained full model.

## Cancel probe

1. The public `caseyallard/uor-r4-m1-model` artifact
   `t16-frozen-zero-seed1-trunk.d11` is downloadable without a credential and
   hashes to `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
   Its header names 19 tables, 4 blocks, Relation read, and context 192.
2. `CpuDiscreteProposalScorer` currently refuses this artifact because its
   parser accepts only four tables. `uor-r4-discrete::trunk` already contains
   the pure-integer recurrence and joint-pointer arithmetic needed for an
   independent CPU implementation.
3. This is a read/parity unit. It does not claim CPU training on the 19-table
   trunk or a new language result.

## Design

- Add a read-only CPU trunk reader over a validated
  `uor-r4.lut-discrete-trunk/1` artifact.
- Reuse the independent CPU Route/Flock/Relation base read, then compute the
  trained recurrence code, causal pointer successor, joint-pointer selection,
  Hamilton rotation, and post-rotation writer through the pure-integer
  `uor-r4-discrete::trunk` functions.
- Keep the existing training and writeback paths refusing the 19-table
  artifact; this unit must not silently mutate a model whose extra tables the
  trainer cannot rewrite.
- Extend `cpu-public-discrete-parity` to use the trunk reader on the pinned
  public artifact and compare complete query vectors, admitted cells, top-8
  and token decisions against `IntegerDiscreteModel`.

## Files in scope

- `crates/uor-r4-cpu-train/src/discrete_route.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-public-discrete-parity.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-90/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. A synthetic 19-table artifact exercises the CPU trunk reader and matches the
   integer serving evaluator on query, admitted cells, top-8 and token.
2. The reader refuses malformed table shape, recurrence candidates, sparse
   tables, codes and planted corruptions.
3. The public seed-1 artifact completes all 14,287 DEV rows with zero query,
   admitted-cell, top-8 and token mismatches; admitted/top-1/top-8 totals equal
   the published M1 readings.
4. Focused tests, the full `uor-r4-cpu-train` suite, strict Clippy, targeted
   formatting, `git diff --check` and the local PR gate pass.
5. The REPORT states the runtime, peak RSS and exact claim boundary, and keeps
   training/writeback of the 19-table trunk explicitly open.

## Stop if

- Independent CPU arithmetic cannot use `uor-r4-discrete::trunk` without
  copying banned dense operations or calling `IntegerDiscreteModel` for the
  decision.
- The public artifact fails its pinned SHA, its DEV inputs fail their pins, or
  the published target totals cannot be reproduced.
- A malformed artifact reaches runtime rather than failing closed at load.
