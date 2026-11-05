# REPORT: cpu-training-pilot, deepseek-167

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `127c6e310`.

## Status

Complete and merged. The learned geometric copy read is **REJECT**.

## Question

Can a learned modulo-256 code read choose the correct token among exact-copy
candidates after static writer-row sign distance failed?

## What did not work

Deepseek-165's raw token-row sign-Hamming arbitration was non-vacuous but lost
most of the established exact-copy body gain. This unit therefore learns from
generated train trajectories and restricts the learned output to the existing
candidate set.

The learned read also failed its bar. It tied longest-32 on seed 1 and gained
one body position on seed 2, both below the paired threshold. The shuffled
read did not establish a treatment advantage.

## What changed

- Added a learned generated-state copy readout derived from longest-32 train
  trajectories.
- Added candidate-constrained learned and shuffled-read arbitration.
- Added a deterministic 1,024-row training cap over the existing exact
  nearest-example fitter.
- Added independent tests for code distance, training alignment, thresholding,
  candidate fallback, deepest-occurrence selection, replay and digest binding.
- Registered the fresh seed-2763 screen and the full control set.

## Measurements

- implementation pin: `d32825212b90b8821afa36314d8d0f70a36499df`;
- training fit: 4,661 generated rows reduced deterministically to 1,024 rows;
  leave-one-out top-1 was **116 and 125**, ambiguity **95 and 93**, coverage
  **929/1,024** and **925/1,024**, and fitted distance radius **957/2,048**;
- fresh panel: seed 2763, `dev,dev`, 128 conversations and **453** reachable
  replies per artifact seed;
- learned body positions: **474/4,686** and **475/4,686**;
- longest-32 body positions: **474/4,686** on both seeds;
- learned candidate arbitrations: **210** and **174**;
- learned versus longest changed **52** decisions on two rows for seed 1 and
  **245** decisions on six rows for seed 2;
- shuffled-read body positions: **475** and **474**;
- random-32 changed **2,514** decisions on both seeds;
- other fresh body controls: `k2>k1` **426**, random-32 **340**, static
  sign-distance **258/235**, fixed depth 1 **213**, writer **51/58**;
- every arm had zero exact, BOS-complete and premature-BOS replies;
- fresh-process repeats were byte-identical after timing fields;
- learned throughput was **52.5k-52.6k** generated tokens/s, about 80% of
  longest-32.
- compact evidence:
  [`learned-read-2763/`](../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/evidence/learned-read-2763/),
  raw local manifest SHA-256
  `b53633e201bef6d9b81dec1f51894d1db12297f0330b2de2fd0e6ad6ad141afd`.
- reviewed head: `9909feb356fad2a2157d1a66f327e1cb6d8f842f`;
- protected merge: `f579da5978a80684d47d47498d0817810f90710b`;
- fresh-main CPU library: **302 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**.

The paired comparisons against longest-32, longest-4 and shuffled-learned
failed on both seeds. The sealed outcome is **REJECT**.

## Checks run at the head

- CPU library: **302 passed**, 4 expected ignores.
- All CPU-training binary test targets passed.
- Focused learned-copy tests: **7 passed**.
- Scoped Clippy with `-D warnings`: passed.
- rustfmt and `git diff --check`: passed.
- Fresh output root and seed 2763: absent before compute.
- Local sealed evidence manifest and all copied evidence: verified.
- Published evidence sums and the no-`.u16` corpus boundary: verified.

## What remains

No delivery work remains for this bounded experiment. Learned copy arbitration
remains open, but the next attempt must use a materially different candidate
representation or training target; more rows or a looser threshold on this
exact readout are not indicated.

## Size

Against base `127c6e310`, the merged result changes 35 paths (**+2,185 / -4
textual lines**) plus gzipped evidence blobs. No `.u16` corpus is committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This report makes no complete-reply, language
or serving claim.

## Plain summary

Learning a modulo-code query read did not improve exact-copy arbitration. It
produced a valid, deterministic candidate-only control, but its weak
leave-one-out fit transferred to no meaningful body gain. The single decoder
step remains too underdetermined for this row-level nearest-example approach.
