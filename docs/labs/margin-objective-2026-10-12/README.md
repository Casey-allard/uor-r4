# The rank-margin objective: D23's first softmax-free training objective, with its CUDA kernel

References #2029 (M1; pre-registered rung 0 in [#2029 comment 6124773996](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6124773996), amended in [#2029 comment 6126881953](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6126881953)).
Lab: Kimi, session `kimi/s1`, 2026-10-12. Source work only: no training, no panel run, no model compute.

> **One line.** The census said the largest D23 gap was the objective: nothing in Line A trained without
> the implicit softmax inside cross-entropy. This is the first replacement: `Objective::MarginHinge`, a
> rank hinge `mean_k relu(1 − z_target + z_k)` over each row's strongest rival logits — **no exponential
> of any kind in the forward or the gradient** (the owner's no-softplus/no-softmax-shaped rule is honored
> by construction) — implemented for the CPU with its CUDA f32 forward and backward kernels, and wired into
> `train` and `dialogue-train` behind `objective=hinge|hinge8`.

## What landed

- **`Objective` enum + `StackConfig::objective`** (`Ce` default): a `Ce` configuration writes no field, so
  every existing config stays byte-identical (the frozen-config test passes unchanged); a margin objective
  is recorded in `config.json` and validated (rivals 1–64, a pointer head is refused for now — its own
  objective is the next rung).
- **`MarginRank` op** with the exact weighted-mean semantics of `CrossEntropy` (zero-weight rows cost
  nothing, weights normalize by their sum). CPU forward and backward in f64; CUDA f32 kernels
  (`margin_rank_rows`, `margin_rank_grad`): one warp per row, per-lane top-k selection merged on lane 0,
  ties to the lower index — the CPU stable-sort's order, so CPU and CUDA agree except on exact float ties
  of equal-valued rivals (loss-identical, gradient-slot-different; declared). Metal is refused for now.
- **Trainer flag** `objective=ce|hinge|hinge8` in `train` and `dialogue-train`; the run's config records it.
- **Tests (9 new):** hand computation of the row loss at three rival counts and the win case; a
  finite-difference gradient check off every hinge kink (the first fixture sat exactly on one, a real test
  defect caught and fixed); `Ce` bit-exactness through the new dispatch (the pre-change values, captured
  first); hinge-vs-hinge8 ordering; objective serde round-trip and every refusal; a serving-compliance
  witness that a non-pointer model's `next_scores` returns the raw last logits bit-for-bit (no softmax
  anywhere in the margin rung's scoring path).

## Verification

- `cargo test -p uor-r4-training --lib` (with `UOR_BUILD_SOURCE_COMMIT` set): **938 passed, 0 failed**.
- `cargo check -p uor-r4-training --examples --bins`: clean; `cargo fmt --check`: clean;
  `check_claim_wording.py`: PASS.
- **CUDA feature compile + parity on a pod** (the #2212 lesson — a CUDA path is not verified until a
  pod built from the branch runs it): **PASS** on an RTX 4090 (pod `zkjg238gzdhyv3`, cap 89, nvcc 12.8,
  rustc 1.97.1) at `f1e9acb7a`: the full CUDA parity suite **38 passed / 0 failed**, including
  `test_margin_objective_parity ... ok` (BUILD.json on the pod volume; bootstrap log in
  `~/.local/share/uor-r4/compute/logs/`). Two defects were caught by this gate before merge: an
  out-of-bounds read in the selection merge's tie-break on small vocabularies, and the CUDA-only test
  build's missing `objective` fields — both fixed and re-verified.

## Design and basis

The revised [learning-law design](../d23-learning-law-design-2026-10-12/README.md) carries the full
replacement program (objective, read, murmuration addresses, served output, evaluation) and the
owner-supplied geometric basis — the SpiralCore v69 site and the elliptic-curve murmurations paper
(arXiv:2204.10140v2), both deep-reviewed this session: the 600-cell/H4 addressing substrate the stack
already uses, the Cl(0,6) E8 operator orbits, the H2 decagons, the FBS binder, Kuramoto phase coupling as
the named readout alternative (with a shuffled-geometry null), and the paper's real lesson — role is
recoverable from trace-oscillation patterns — with the standing distinction from the starling-flock family.

## Cost

| | |
|---|---|
| compute | laptop CPU (builds + focused tests); one 1×4090 pod for the CUDA compile receipt, ≤ $0.50 |
| external | none |

## Next

1. TEST FITNESS on rung 0 (#2029 comment 6124773996 as amended).
2. Rung 0 on one pod: M1/M8 arms × 2 seeds on `mw-balp`, recurrence-only `rrrrrr`, margin curves and
   unviolated-target rate against the pre-registered bars.
3. Rung 0b's kernels: the rank read's CUDA kernel, the score-gradient stop (the rank read's backward is a
   softmax STE today), and the murmuration address credit.
