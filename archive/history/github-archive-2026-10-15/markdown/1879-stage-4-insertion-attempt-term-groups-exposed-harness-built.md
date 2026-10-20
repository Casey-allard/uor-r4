# #1879 — Stage 4 insertion attempt: term groups exposed, harness built, gate NOT passed (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T06:22:48Z
- **Updated:** 2026-10-09T13:20:59Z
- **Closed:** 2026-10-08T06:24:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1879
- **Merged:** 2026-10-08T06:24:30Z
- **Merge commit:** 41234d24b06dbb16dcb2ea9644c50ffcc97b1416
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-insertion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

Stage 4's gate is improvement on **real** held-out text with the discretisation gap and utilisation reported. This lands the harness and its first measurement.

1. **`ExportedGeometricModel::score_context_candidate_terms`** — the scorer's six additive term groups (bias, lattice lags, S2 readout, VSA, engram, hierarchical lattice) exposed as `CandidateTermScores`, with `score_context_candidate_with_vsa` now defined as its `total()`. Pinned by `term_groups_sum_to_the_scorer_exactly`, which asserts equality for every candidate of three contexts — so a learned term reads exactly what the scorer sums and cannot drift from a parallel reimplementation.
2. **`crates/uor-r4-core/src/bin/lut4-prose.rs`** — 12-bit features per candidate (sign and magnitude bit per term group), LUT-4 training at the training corpus head, correction scale chosen on a disjoint selection tail, held-out bits per byte on a **separate real corpus file**, plus an exhaustive search over all `C(12,2) × 16 = 1056` two-input terms and a feature-informativeness diagnostic.

## Two real defects found and fixed

- The backward pass routed the connectivity **utility** upstream instead of the chain-rule gradient. `Lut4Layer::backward` now returns the exact input gradient; the utility only keeps a live alternative wired.
- A uniform gate mixture maps **every** input pair to exactly 0.5 — the mixture's fixed point. The next layer then receives a constant, no gate can alter its output, and training stalls at the best constant: measured as a loss pinned at 0.25 with balanced labels and an output gate collapsed to a constant. Initialisation now breaks the symmetry near the identity gates.

## Gate: NOT PASSED

Artifact `native_geometric_prose_model.rgm` sha256 `7023c450…`; training on `chat-v0-p2/train/tokens.u16`, held out on `chat-v0-p2/heldout/tokens.u16` (real conversational text, a separate file).

| run | baseline BPB | with term | delta | top-1 change |
|---|---:|---:|---:|---:|
| 512 train / 256 eval | 4.211070 | 4.211070 | 0.000000 | 0.0 |
| 64 / 64, exhaustive best term (bits 4,7 gate 14) | 4.669634 | 4.669640 | **+0.0000061** | 0.0 |

The exhaustive best term reaches a training loss of 0.2371 against a constant-0.25 baseline — it classifies target-versus-decoy at about **76 %** on balanced classes — and the best single feature bit separates the classes by 0.371 mean. **The features carry signal; the term does not transfer it to held-out probability**, at any correction scale in the grid, and the argmax never changes.

## Scope, so it is not over-read

One artifact (a TinyStories-trained prose scorer), one real corpus, 64–256 scored positions per slice, debug build, no serving default changed. This establishes that at *this* feature encoding, arity and correction form, a learned nonlinear term does not improve held-out BPB. It is **not** evidence against nonlinearity: a term that classifies at 76 % carries information this form does not convert.

## Next actions this points at

1. Richer features: quantise each term group to several thresholds instead of two bits, and keep per-lag lane terms separate rather than summed.
2. Insert the term as a calibrated per-position re-ranking correction rather than an additive logit shift, so a 76 %-accurate classifier changes the ordering it can change.
3. Re-run this harness — the scale grid, the informativeness diagnostic and the 1056-term search are in place and cheap.

Evidence: `docs/evidence/native_geometric_lut4_stage4_insertion_2026-10-08.txt`. Verification: `cargo check -p uor-r4-core --all-targets` clean, `cargo test -p uor-r4-core lut4` 6 passed, the new equivalence test passes, `rustfmt` clean.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

