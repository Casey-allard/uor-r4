# #395 E8/icosian spike — Measurement 1: neighborhood preservation at matched bits

- **Date:** 2026-08-04
- **Issue:** #395 (program: #393)
- **Tooling:** `crates/uor-r4-graph-certify/tests/e8_dump_bundles.rs` (dump) +
  `research/395-e8/e8_spike.py` (measurement)
- **Data:** 50,000 centered context-bundle vectors (dim 288), sampled every
  10th record from the checked-in fixture corpus — the exact f32 objects the
  shipped path sign-codes into 36 bytes. Disjoint train (25k) / pool (20k) /
  query (500) split, seed 395.
- **Claim language:** Empirical Criterion, status Empirical (pinned-corpus
  measurement with declared protocol; not a proof).

## Question

At an exactly matched budget of 288 bits/vector, does E8 lattice quantization
of real store content preserve neighborhood structure better than the shipped
orthant (sign-bit) code?

## Codes

| code | construction | bits |
|---|---|---|
| SIGN | sign bit per dim; Hamming ranking | 288 |
| E8-256 | 36 blocks × 8 dims; Conway–Sloane nearest E8 point (best of D8, D8+½) at a per-block scale grid-picked on train MSE; codebook = 256 most frequent train lattice points per block; ranking by L2 between dequantized vectors | 36 × 8 = 288 |

## Results

| metric | SIGN | E8-256 |
|---|---|---|
| recall@10 vs cosine ground truth | 0.553 | **0.606** |
| recall@10 vs L2 ground truth | 0.541 | **0.634** |
| relative reconstruction MSE | 0.446 | **0.213** |

Train data occupies ~22k distinct lattice points per block; the 256-point cap
discards most of them and E8 still wins on every metric. Reconstruction error
halves; L2 neighborhood recall improves +9.3pp absolute (+17% relative).

## Icosian reading

Each 8-dim block codeword is an E8 lattice point; via the icosian ring
(quaternions over Z[φ], Turyn norm) every such point is canonically a
golden-coupled **pair of R⁴ points**. A 288-dim context vector under this code
is 36 icosian pairs — 72 quaternionic orientations — rather than 288
uncoupled sign bits. This is the construction-level R⁴ substrate #393 calls
for; nothing here touches scoring channels.

## Caveats

- recall@10 against a 20k pool is a retrieval proxy; the decision-grade test
  is downstream (region assignment / store keys built from E8 codes, graded
  by certify metrics and the #394 infill criterion).
- Fixture distribution only; natural-corpus rerun pending #381 obs corpus.
- Heavy-tailed blocks (per-block scale grid was necessary); no entropy coding
  assumed — hard 8 bits/block cap.
- Decoder cost: Conway–Sloane E8 is round/compare/coset-select per block —
  compatible in principle with the no-multiply contract (fixed-point scales),
  but kernel-contract conformance is unverified and is future work.

## Decision gate

Per #395: E8 preserves neighborhood structure better than orthant signs at
equal bits → **promote to a construction experiment**: compile a bundle whose
assignment path consumes E8 block codes (prototype/membership geometry over
icosian pairs), grade with certify + #394 infill rows against the shipped
assignment at equal store budget.
