# D5 addressing contest — predeclared plan (T2, Lab 2 OpenCode)

September 27, 2026. References #973 under #820. **Frozen before any run.** Director
spec: ROADMAP.md §4.2 plus the Step 2 work unit; this plan fixes the implementation
decisions and cost only. No training or fitting of the model; no serving default changes.

## Question

At equal bits and bytes touched per query, does a fixed geometric codebook admit the
events the dense read uses as well as the best ordinary index?

## Model and populations

- Parent: `fit-quaternion-6/checkpoint-final` (step 15,672), continuous, Full admission.
  The paired ordinary-recurrence parent only if cheap.
- No Lorentz arm (these keys were not trained with a distance score).
- Populations: (a) the evaluator comparison tail (912 blocks / 233,472 targets); (b) its
  long-range slice — positions whose dense top-1 event is ≥32 tokens back; (c) the 5
  condition-A distractor rows — is the correct entity admitted?
- Codebooks are fit on the **tune blocks only** (64 blocks), f64, offline.

## Arms (s ∈ {4, 8, 16, 32} admitted events of up to 255; model read ranks within the set)

1. `recent-s`;
2. `oracle top-s by q·k` — the ceiling for any content index;
3. 600-cell / 120 H4-root codes on 16 × 4-D blocks (6.9 bits/block) vs k-means-120 per
   block (3 seeds) vs random-projection sign codes at equal bits;
4. E8-root codes on 8 × 8-D blocks (7.9 bits/block) vs k-means-240 per block vs sign
   codes at equal bits;
5. each fixed codebook with and without a seeded randomized Hadamard pre-rotation
   (sign flips + Walsh–Hadamard; adds only; the 1/8 scale is a shift);
6. hybrids: the best arm per family with a recent-8 share inside the same s.

Every arm scores keys **asymmetrically** against the exact query, PQ-ADC style: quantized
keys plus a per-query table of block inner products. Boundaries are reported as exact
decisions, not estimated retractions.

## Metrics (per arm and budget)

- recall@s of the dense top-1 and top-3 events, NoRead excluded;
- captured dense attention mass (sum of dense masses over the admitted set);
- decode work per query (table reads, adds, multiplies) and index bytes per event;
- greedy decision flips and NLL change against Full — restricted-read runs only, best arm
  per family at s=16 (cost control).

## Cost control

- One forward pass over the Full-run states computes the one-step metrics for all arms.
- Full restricted-read evaluations only for the best arm of each family at s=16.
- Implementation: `addressing_arms.rs` (new Lab 2 module: codebooks, fit, encode, PQ-ADC,
  accounting, serde) + `examples/joint-addressing-contest.rs` (one-pass harness) + a thin
  in-crate CLI that reuses the existing fixed-weight policy evaluation
  (`joint-evaluate-admission` / `evaluate_loaded`); **no new evaluator**.
- Additive accessors may expose the per-step read query and key history (`JointStep`),
  default-inert, since §4.2 specifies frozen read-head queries and keys dumped from the
  retained model.

## Decision rule (fixed, director)

A geometric arm qualifies as T3's admission index if it is within **1 point of recall@s**
and **0.005 nats** of the best ordinary arm at the same s, **with a cheaper
multiplier-free decode**. Otherwise geometric addressing is not adopted for serving,
scoped to this model, T=256 and these bit rates. (Rerun on T1(a)'s stack reads later if
that becomes the main line and its checkpoint is local.)

## Resources

- Light job at ≤2 threads / ≤1.5 GB unless the pass is heavier; claim
  `/Volumes/UOR-Workspace/locks/model-slot.json` before any run over 2 GB RSS or over
  10 minutes at more than 2 threads; never overlap the owner's energy measurement; never
  kill or pause another lab's job.
- Build cache `/Volumes/UOR-Workspace/BuildCaches/opencode-addressing-contest`; outputs
  `/Volumes/UOR-Workspace/uor-r4-lab/opencode-addressing-contest`.
- Report model compute separately from orchestration; reconcile the ledger in one line.

## Scope

Offline, fixed weights, no training. This decides one index question for one model at
T=256 and these bit rates; it does not qualify a geometric advantage, a serving path or
general language.
