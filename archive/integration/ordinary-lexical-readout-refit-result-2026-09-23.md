# In-class readout refit on the frozen state: the served alphabet realises the fit, and the whole gain is about 0.13 bits — September 23, 2026

This record executes the milestone the previous one named: *freeze the recurrence and the embedding table
and train the served readout **in class** on the frozen-state data, pre-declaring served loss below
`olx-form-2`'s 6.6841 by ≥0.30 bits with the grounded panels retained, and reconcile the state probe's
artifact-initialisation discrepancy.* It resolves the discrepancy exactly, builds the refit, and reports a
**positive but bounded** result — together with an explicit retraction of this record's own first draft
conclusion.

Base `67f2e350` (head of open PR #1363) plus open PR #1364; new work on branch
`codex/readout-refit-20260923`. The model source `learner/transferable_lexical.rs` is **unchanged**
(`sha256 03f83eb8…b250`). Everything here is a runner-side diagnostic over recorded states; no artifact is
modified or re-sealed. Evidence:
[`docs/evidence/ordinary-lexical-readout-refit-2026-09-23.json`](../evidence/ordinary-lexical-readout-refit-2026-09-23.json).
Sealed roots: `/tmp/olx-sp-e` (the pre-fix generation), `/tmp/olx-sp-parity`,
`/tmp/olx-sp-parity-serv` (the delivered-source generation). One full-coverage run, `olx-refit-1`, was
stopped deliberately before sealing and is labelled unsealed wherever it is quoted.

## 1. The discrepancy was a softmax **base** convention, worth exactly `ln 2`

`FloatProbe::nll_bits` normalises with the **natural** exponential; the artifact's served scoring,
`log2_softmax_row`, normalises with **base 2** over logits already scaled by `2^-score_shift`. The artifact
initialisation handed the probe the served logits unchanged, so the probe's distribution was the artifact's
at a temperature `1/ln 2 ≈ 1.4427` too small — **sharper**. Because the artifact's dyadic scale sits at its
own NLL optimum, sharpening could only *raise* the loss, which is exactly why the epoch-0 loss came out
**above** `A_restricted` (6.5954 vs 6.055956) — the inequality that looked impossible.

Four steps forced the conclusion, all recorded in the receipt: the pre-existing logit control already showed
the rearrangement was right (deviation `0.0` over 131,072 values); the new `A_kclass` reference was
validated three ways — its full-legal side reproduces the **recorded** `A_restricted` exactly (dev
6.055956, fit 6.018168), the identity `kclass = full + mean(log2 mass)` holds exactly, and the class rows
are distinct, inside the legal set, with mass ≤ 1 (worst 0.998); a new **per-row** check showed **identical
logits (1.1e-14) but different losses** (K=64: probe 4.072948 vs artifact 3.762429, worst row gap 5.65
bits), leaving only the exponent base; and multiplying the initialisation's `W` and `b` by `ln 2` collapsed
the gap to **7.1e-15**.

The invariant is now **asserted**, not assumed: `--probe-init artifact` aborts if the initialised probe's
loss differs from `A_kclass` on the identical position sets, and `A_kclass` asserts `A_kclass ≤ A_restricted`.

**Corrected reference values.** `A_kclass` is **5.977039 dev / 6.018168 fit** at K=1024; the class
restriction is worth **0.078917 bits** of mass on the restricted dev targets; the pre-fix epoch-0 diagnostic
was inflated by **+0.6183**.

**The previous record's headroom stands, now measured directly:** `A_kclass − P = 5.977039 − 5.5136` =
**0.4634 bits**. The previous record published "≥0.40" by subtracting a *bounded* convention allowance from
`P − A = 0.5423`; the direct measurement is 0.463, inside that bound. No correction to the previous claim;
only the tighter number and an instrument that makes it checkable.

## 2. The refit: with an **exact** initialisation, the served alphabet realises the fit

`--readout-refit` trains the readout in the served alphabet (ternary rows, one power-of-two scale per row,
integer bias) on the frozen state, with the artifact's own straight-through convention, and scores through
the artifact's own integer kernel (`TlLinear::forward_i32`); the `Copy`/`Stop` rows are frozen at the
artifact's values, so the loss is comparable with the served one.

**Two instrument generations must be kept apart**, and getting this right is the whole content of this
section:

| generation | binary | refit init vs the artifact's own readout | refit dev, K-class scope | refit dev, served scope (all 4,098 rows, all 5,376 dev targets) |
| --- | --- | --- | ---: | ---: |
| `olx-sp-e`, **sealed**, pre-fix | `c87a013b…` | 4.9188 vs 4.7793 / 6.8941 vs 6.6841 — **0.14 / 0.21 bits too high** | 4.7721 vs `A_kclass` 4.7793 (**−0.0071**) | 6.7741 vs 6.6841 (**+0.0900**) |
| `olx-sp-parity` / `-serv`, **sealed**, delivered source | `7cec87f1…` | **exact**: 4.7793 vs 4.7793, and 6.6841 vs 6.6841 | **4.6357 vs 4.7793 (−0.1435)** | **6.5528 vs 6.6841 (−0.1313)** |

In the delivered-source generation the initialisation is **exact** for both scopes — asserted, and measured
as `worst absolute deviation 0` over `64 positions × 4098 rows = 262,272` logits in the served scope. And in
both scopes **the integer realisation equals the float surrogate** (4.6357 = 4.6357; 6.5528 = 6.5528), so
there is **no realisation cost**: the ternary codes, per-row power-of-two scales and integer bias express
what the fit found.

**So the milestone's answer is positive and bounded.** An in-class ternary refit on the frozen state
improves the **served** readout by **0.1313 bits/target** end-to-end through the integer kernel (top-1
0.2161, from a 0.1959 baseline in the pre-fix generation), and beats its matched K-class reference by
0.1435. The pre-declared bar of **≥0.30 bits is not met**: on this state, refitting the readout alone
delivers about a quarter of the 0.4634-bit matched-normalisation figure.

## 3. Retraction of this record's own first draft

An earlier draft of this document (and the public entrypoints it propagated) concluded that *"the fit is not
the limit; the integer realisation is"*, quoting a realisation cost of ≈0.27 bits. **That conclusion was
wrong and is retracted.** It was inferred from a single instrument generation whose refit initialisation sat
0.14–0.21 bits above the artifact's own readout — a defect in the refit's construction, not a property of
the alphabet. Once the initialisation is exact the realisation cost vanishes, and the refit's *integer*
result equals its float surrogate. The lesson is the same one this programme keeps relearning: **a
comparison whose initialisation is not exact cannot scope a negative to a mechanism.** The defect itself was
found by the implementation round auditing its own construction, and the delivered source carries the fix
and the assertion.

## 4. Controls (delivered-source generation, sealed)

Kernel cross-check `0.0` on the declared 64-position sample. Frozen-bias ablation 4.6232 (K-class) and
6.5544 (served) — i.e. the learned bias contributes ~0.01 bits, so the gain is in the vocabulary rows.
Permuted-target null 6.4188 (K-class) and 8.2744 (served) against floors of 4.7793 and 6.6841, `ok=true`,
with the floor corrected to `min(its own initialisation, the restricted unigram)` because an
artifact-initialised refit already beats the unigram. Both alphabets' round trips and the straight-through
gradient are unit-tested; seven runner tests and twelve `transferable_lexical` tests pass.

## 5. Other instrument notes, and a second scope retraction

- **The previous record's post-hoc re-pricing figures were a degenerate quantisation.** At K=1024 the
  quantiser produced an almost empty matrix (code density 0.0088, 614 of ~70k codes non-zero, logits in
  [−18, 5]); at K=256 with a working pre-scale the same machinery gives density 0.30 and a usable readout.
  So 8.6058 / 7.6181 / 8.9683 are **not** evidence about the 4-bit or ternary alphabets. Scoped retraction
  of an instrument claim, not of the headroom measurement.
- **Training adequacy must be read from the receipt.** The vocabulary-wide refit is 4,098×68 and costs
  ≈0.9 ms per training position, so a converged run is expensive; the receipt prints the visited share
  explicitly, because an under-trained refit that fails to beat the artifact is not evidence about the
  alphabet. Both delivered-source arms here trained on 251,098 of 334,796 fit positions (75%).
- The unsealed `olx-refit-1` full-coverage attempt used the **pre-fix** generation; its numbers (init
  6.1932 → 6.0715, float surrogate 5.7984) are the defective-init case and are recorded as unsealed.

## Limits and claim boundaries

Measured, on the delivered source: the discrepancy's cause and exact resolution; the matched reference and
the restriction's mass value; that an exactly-initialised in-class ternary refit on the frozen state
improves the served readout by 0.1313 bits and its matched K-class reference by 0.1435; that its integer
realisation costs nothing relative to its float surrogate; and the controls above. **Scoped to**
`olx-form-2`, the served window conditioning, teacher-forced states, one corpus, one split, one seed, the
top-256 K-class population (69.4% of dev targets for the K-class figures) and all 5,376 development targets
over all 4,098 legal rows for the served-scope figures, at 6 epochs and 75% of the fit population visited.
The pre-declared ≥0.30-bit bar is **not met**. No grounded-panel retention has been evaluated for a refit
readout — the panels need a readout inside a model, which a runner-side refit is not. No general-language,
reasoning or geometric-advantage claim; generation still collapses; sampling NOT_RUN; energy UNAVAILABLE.
The refit is an offline diagnostic in the served alphabet, not a serving path or product dependency.

## Resources

One implementation round that overran past 100 minutes of agent time; one release build; five sealed smoke
runs and three delivered-source parity runs; one full-coverage run stopped unsealed on purpose. Machine time
≈3,900,000 ms of the **8,200,000 ms** charged. The projection, three prospectively recorded ledger
extensions and the final charge are in the [resource ledger](resource-ledger-2026-09-19.md). No paid or
external compute; no artifact, preserved root or owner checkout modified.

## One evidence-supported next milestone

**Take the refit out of the frozen state.** The measured fact is that a refit of the output layer alone, on
states the current model produces, buys about **0.13 bits** while the same states support **0.4634** under a
matched float readout — so ~0.33 bits are unreachable without changing the states the readout is fit on. The
next arm is therefore one **joint** fit on the served conditioning with the output layer initialised from
the converged refit (and, optionally, an output-focused schedule), keeping grounded Copy/Stop supervision,
pre-declaring **served loss below 6.6841 by ≥0.30 bits with preflight A 32/32, held-out 3/3 and class 4/4
retained**, and reporting the same three numbers this record does — the matched reference, the float
surrogate and the integer realisation — so a shortfall is attributable rather than ambiguous. If a joint fit
also stalls near 0.13 bits, the remaining branch is the input side: explicit **second-order local access**,
since a training-free **raw-embedding** decode recovers `cur` at 96.7% and `prev` at 2.1% — **corrected
2026-09-23:** a transport-aware decoder recovers `prev` at **75.90 %** on the same states, so the premise
"the state does not retain `prev`" is withdrawn for the training-free observer class; see the
[state-probe correction](ordinary-lexical-state-probe-result-2026-09-23.md).
