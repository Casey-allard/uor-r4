# D6: what the 2I read representation discards — result

September 28, 2026. References #973 and #820.
- **Lab:** Lab 2 (OpenCode), code commit `67942bfb` ([#1464](https://github.com/UOR-Foundation/uor-r4/pull/1464)).
- **Record and evidence:** added by Lab 1, which is not the author, from the sealed roots ([evidence](../evidence/d6-information-audit-2026-09-28.json)).

**Status.** The run is complete and evaluation only: no training, fitting, generation hook or codebook sweep. Two full runs, `full-1` and `full-2`, give identical deltas. The compiled sources of `full-2` equal commit `67942bfb`, file for file.

## Setup

**Parent:** #1438's kernel-off checkpoint (optimizer step 16,696), the native model with a dot read (width 256, read width 64). **Its anchor is reproduced:** comparison read NLL 1.9847528 against a declared 1.984752805 ± 1e-3.

**Population:** 48,412 selected read positions. These are comparison-tail positions with at least one earlier event and NoRead mass below 0.5, out of 232,560 positions; 184,148 were excluded by NoRead.

**Metric:** Δ is the fraction of selected positions whose top-1 read event differs from the parent's own dot score (O). The age bias, NoRead, candidates, values and decoding are held fixed.

**Instrument check:** the computed dot-score argmax equals the model's own read-mass argmax at all 48,412 positions.

## Result (sealed root `full-2`)

| Arm | Representation | Δ | Top-1 differs |
|---|---|---:|---:|
| O | dot / √read_width, the anchor | 0.000000 | 0 |
| U | per-lane unit dot, unquantized: 16 lanes of 4-D, L2-normalized | 0.413431 | 20,015 |
| D | 2I direction only: each unit lane snapped to the nearest of 120 roots | 0.464368 | 22,481 |
| G | 2I direction plus a 3-bit dyadic gain: 10 bits per lane | 0.414938 | 20,088 |
| K | per-lane raw k-means with 1,024 centroids: 10 bits per lane | 0.347331 | 16,815 |

**Outcome: ORDINARY-BETTER,** applied mechanically by the pre-declared logic.
- **Magnitude is load-bearing:** Δ(D) = 0.4644 ≥ 0.05, and Δ(U) = 0.4134 ≥ Δ(D)/2.
- **The gain does not restore it:**
  - Δ(G) = 0.4149 > Δ(D)/2 = 0.2322;
  - Δ(G) > Δ(K) + 0.01 = 0.3573.

**Reported, not gated:**
- top-2 code collisions: D and G 8.3e-5, K 5.8e-4;
- lanes clipped by the gain scheme: 396 of 4,510,144 low, none high.

**Complete answers: NOT_RUN.** Per-arm generation needs a hook that substitutes the scorer, which D6 intentionally did not build.

## Scope

- One parent, one population, one declared gain scheme and one k-means seed, evaluation only.
- It audits **the native model's dot read**, not the main-line stack's Lorentz read.
- Δ measures ranking changes, the top-1 event. It says nothing about value error or answer quality.

## Disposition (Lab 1, under the three-lab charter)

**Adopt as representation evidence for G:**
- A post-hoc 2I direction code, with or without a 3-bit gain, is **rejected as an address code at this scope**. It changes the dense read's decision at 41–46% of selected positions.
- G's proposal (Lab 2) must either:
  - carry magnitude, or be learned end to end with the read; **or**
  - show an advantage other than ranking fidelity, for example decode cost or exact-store addressing.
- It must compare against an ordinary learned code at equal bits (k-means-like), which preserved ranking best here.
- D6 is evidence, not a veto (owner, 15:27 UTC): it does not decide geometric readers as a family.

**Carrying it to the stack** needs a stack-side check: the same substitution on the stack's Lorentz read. That check becomes worth running when G's design depends on it.

**D3's gain-controlled follow-up** is pre-registered by Lab 2 on #973. Its disposition stays with Lab 2's pre-registration.
