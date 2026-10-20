# Card P7 — Per-mechanism falsification sweep — RESULT

Card: [`P7-falsification-sweep.md`](P7-falsification-sweep.md) · Date: 2026-09-19
Executed by: Zed (agent) · **Status: executed, unsealed diagnostic run (not a fresh draw)**

Raw receipt: [`docs/evidence/native_geometric_p7_ablation_2026-09-19.txt`](../../evidence/native_geometric_p7_ablation_2026-09-19.txt)

## Method

Each mechanism is ablated **in isolation** in an in-memory copy of the shipped artifact
and the held-out BPB is re-evaluated. No training, no fitting, no new draw. Numbers are
the **discrete scorer** (`ExportedGeometricModel::score_context_candidate_with_vsa`)
normalised over the full 4,096-token vocabulary with the serving sampler form
`exp((s − max) / (temperature · 8192))` at `temperature = 1.0`. Deltas are **paired** on
identical positions, with a 1,000-resample bootstrap 95% CI.

## Result

> **Superseded labels.** The `Verdict` column below is the first-pass reading and is
> **corrected** by [D2](../../DECISIONS.md): an ablation is a measurement, not a verdict.
> Mechanism class, an equivalence margin and decision-flip rate are now reported, and
> `vsa` in particular is a **wiring defect, not a retirement**. See
> “Corrected reading” below.

Artifact `native_geometric_prose_model.rgm` (2,691,950 B, vocab 4096, 4 lanes,
`flags = 0x000f`); 2,112-token held-out slice, 2,048 teacher-forced positions.
Baseline BPB **1.8055**.

| Ablation | ΔBPB | Paired 95% CI | Verdict | Artifact bytes |
|---|---:|---|---|---:|
| `jepa` | **+0.3743** | [+0.3519, +0.3973] | CONTRIBUTES | 64 |
| `s2_readout` | **+0.3377** | [+0.3204, +0.3565] | CONTRIBUTES | 40,960 |
| `engram` | **+0.2228** | [+0.1919, +0.2544] | CONTRIBUTES | 495,632 |
| `bias` | **+0.1482** | [+0.1366, +0.1604] | CONTRIBUTES | 16,384 |
| `lattice_fine` | **+0.0955** | [+0.0894, +0.1017] | CONTRIBUTES | 57,600 |
| `lattice` (both tiers) | **+0.0927** | [+0.0853, +0.1000] | CONTRIBUTES | 1,851,400 |
| `vsa` | −0.0007 | [−0.0010, −0.0005] | **INERT/HARMFUL** | 168,022 |
| `lattice_coarse` | −0.0066 | [−0.0101, −0.0033] | **HARMFUL** | 1,728,000 |
| `lanes` | −0.0071 | [−0.0082, −0.0059] | **HARMFUL** | 115,232 |

## Corrected reading (per D2)

| Mechanism | ΔBPB | Band | Class | Wiring | Action |
|---|---:|---|---|---|---|
| `vsa` | −0.0007 | NEGLIGIBLE | `enabler` | **MIS-WIRED**: heads bind a fixed random codebook (`vsa/codebook.rs`) disconnected from the learned 120-root assignment (`jepa_trainer.rs:1119`) | **Repair the wiring and re-measure.** Not a retirement verdict — the delta measures the wiring, not the mechanism |
| `lanes` | −0.0071 | NEGLIGIBLE | `primary-carrier` | presumed wired | Diagnose. Not a retirement verdict on this evidence alone |
| `lattice_coarse` | −0.0066 | NEGLIGIBLE | `count-table` | wired | Removal candidate **on resource cost** (64.2 % of artifact bytes), not a “no effect” finding — it flips **9.5 %** of decisions |
| `engram` | +0.2228 | MAJOR+ | `count-table` | wired | Retain; 37.3 % flip rate |
| `jepa`, `s2_readout` | +0.3743, +0.3377 | MAJOR+ | `primary-carrier` | wired; **the two interact** | Retain; treat as a pair, deltas overlap |
| `bias` | +0.1482 | MAJOR+ | `modulator` | wired | Retain |
| `lattice_fine` | +0.0955 | MINOR+ | `count-table` | wired | Retain |

Two facts the first-pass labels hid, and which the corrected instrument surfaces:

- **A BPB-neutral mechanism can still change many decisions.** `lattice_coarse` is
  BPB-negligible yet flips 9.5 % of decisions; `vsa` flips 0.7 %. Average loss is not the
  only quantity that matters.
- **A statistically significant delta can be practically meaningless.** `vsa`’s CI
  excludes zero at a magnitude of 0.0007 BPB. Passing a CI test is not evidence that a
  mechanism matters.

## Original findings (retained, superseded where noted)

1. **The VSA attention layer contributes ≈0 under its current wiring.** The cause is
   identified in source: the four heads bind a **fixed random** token codebook
   (`vsa/codebook.rs`) that is disconnected from the learned 120-root assignment
   (`jepa_trainer.rs:1119`), so the only recoverable signal is token identity. **This is a
   wiring defect with a known repair, not evidence about the mechanism.**

2. **The coarse lattice tier is 64.2 % of the artifact and is marginally net-negative.**
   `coarse_trigram` is 1,728,000 B of 2,691,950 B. Its ablation is BPB-NEGLIGIBLE and
   *improves* BPB by 0.0066 while flipping 9.5 % of decisions. Removing it would cut
   bytes/token by ~64 % for a very small quality change — a direct I2
   (bytes-per-token) win, decided on resource cost rather than on the accuracy delta.
   The fine cluster residual (+0.0955) carries the lattice’s contribution.

3. **The learned lane tables carry no measurable contribution.** Zeroing four
   Adam-trained 120×120 i16 maps moves BPB by 0.0071 in the improving direction. Class
   `primary-carrier`, so this is a defect to diagnose, not a conclusion.

4. **Geometry-carried state prediction is the most valuable mechanism per byte.**
   The JEPA block is **62 bytes** (31 × i16, padded to 64) and its ablation costs more
   BPB than any other single mechanism (+0.3743). The S2 readout is next (+0.3377,
   40,960 B). The count mechanisms (engram, lattice) contribute, but at three to five
   orders of magnitude worse bytes-per-BPB.

5. **The metric/serving divergence is now quantified.** The training-time continuous
   figure is 1.2372 BPB; the discrete artifact that actually serves scores **1.8055 BPB**
   under full-vocabulary normalisation — a 0.57 BPB gap between the number that was
   reported and the model that ships. Any quality claim must state which scorer it used.

## Attention repair applied in the same change

`vsa/attention.rs` bound **independent** random roles into query and key
(`r_query != r_key`), so two occurrences of the same token gave
`d_H = d_H(r_query, r_key) ≈ 2048 = THRESHOLD` — not strictly below the threshold — and
received **zero** rectified weight, while mismatching pairs fluctuating below the
threshold received weight. The head attended to noise and skipped its own matches. The
old `test_head_1_induction_circuit` could not detect this (a single comparison against an
out-of-window candidate, decided by the codebook seed), and
`test_multi_head_roles_orthogonality` *enshrined* the broken configuration.

Fixed by sharing one `r_role` between query and key. Because XOR is self-inverse this
makes the binding algebraically a no-op (`d_H(x⊕R, x_j⊕R) = d_H(x, x_j)`), so identical
tokens now give `d_H = 0`. Replacement tests: a controlled multi-seed induction test with
a predecessor-order control and a no-repeat arm, a value-role orthogonality test, and a
regression guard asserting zero distance for identical tokens.

## Limitations

- 2,048 positions from a single already-open development slice; not a fresh draw.
- **Ablated terms are not orthogonal.** Zeroing the JEPA weights also changes the fiber
  the S2 readout consumes, so `jepa` and `s2_readout` overlap; deltas do not sum.
- Full-vocabulary normalisation, not the 64-candidate serving shortlist: an upper bound
  on served quality, free of shortlist-recall confounds.
- Debug build; timings are not a performance result.
- The run is not sealed into a report root, and the historical 9,984-case regression
  replay was not re-executed in this change.

## Consequences for the plan

- The `vsa` mechanism is **not retired**. The required Stage 3 action is to make the VSA
  codebook consistent with the learned representation, then re-measure: if the delta stays
  NEGLIGIBLE and the flip rate stays low with a correctly wired codebook, that is evidence
  about the mechanism; the current run is not.
- The immediate resource win stands: **remove the coarse lattice tier** (64 % of the bytes
  for a very small quality change), decided on I2 grounds.
- Class assignments are judgement, and they drive the actions, so they should be ratified
  by the owner: `vsa` = enabler, `lanes`/`jepa`/`s2_readout` = primary-carrier, `bias` =
  modulator, `induction` = selector, `engram`/`lattice*` = count-table.
- **Add an interaction-aware instrument before any removal.** Single-mechanism deltas do
  not sum; a pairwise factorial for the known-interacting pair (`jepa` × `s2_readout`) or a
  Shapley attribution over the mechanism set is required.
