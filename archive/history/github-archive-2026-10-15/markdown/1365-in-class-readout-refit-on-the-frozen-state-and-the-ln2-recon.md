# #1365 — In-class readout refit on the frozen state, and the ln2 reconciliation (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T16:07:30Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-23T17:19:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1365
- **Merged:** 2026-09-23T17:19:59Z
- **Merge commit:** 05243622c6b93601d573e5e551a235bc3ca7d5fd
- **Base / Head:** main / UOR-Foundation:codex/readout-refit-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the milestone the previous PR named: reconcile the state probe's artifact-initialisation discrepancy, then test whether the served output layer can be refit **in its own alphabet** on the frozen state to recover the output-side headroom.

`References #820` and `References #973`. Nothing is closed.

**Stacked PR — third in a chain.** Base is the head of open PR #1364 (`1f77a985`), which is itself stacked on open PR #1363. **Merge in the order #1363 → #1364 → this one.** New work here is one commit, `0da2d61e`.

## 1. The discrepancy was a softmax **base** convention, worth exactly `ln 2`

The probe normalised with the natural exponential; the artifact's served scoring (`log2_softmax_row`) normalises with **base 2** over logits scaled by `2^-score_shift`. The artifact initialisation handed the probe the served logits unchanged, so the probe's distribution was the artifact's at a temperature `1/ln 2` too small — **sharper**. Because the artifact's dyadic scale sits at its own NLL optimum, sharpening could only *raise* the loss, which is exactly why the epoch-0 loss came out **above** `A_restricted` — the inequality that looked impossible.

Four steps located it, all in the receipt: the pre-existing logit control showed the rearrangement was right (deviation `0.0` over 131,072 values); the new `A_kclass` reference reproduces the **recorded** `A_restricted` exactly (dev 6.055956, fit 6.018168) and satisfies `kclass = full + mean(log2 mass)` exactly; a new **per-row** check showed **identical logits (1.1e-14) but different losses** (worst row gap 5.65 bits at K=64); and multiplying the initialisation by `ln 2` collapsed the gap to **7.1e-15**.

The invariant is now **asserted** for both the probe and the refit — they abort unless their initialisation matches the artifact exactly, measured at **deviation 0 over 262,272 logits** in the served scope.

**Corrected matched reference:** `A_kclass` = **5.977039 dev / 6.018168 fit** at K=1024; the class restriction is worth **0.078917** bits of mass; the pre-fix diagnostic was inflated by **0.6183**. **The previous PR's published output-side headroom is confirmed at `A_kclass − P = 0.4634` bits** (it was published as "≥0.40") — that claim needed no correction, only the tighter number and an instrument that makes it checkable.

## 2. In-class served-alphabet refit on the frozen state: positive but bounded

`--readout-refit` trains the readout in the served alphabet (ternary rows, one power-of-two scale per row, integer bias) on the frozen state, with the artifact's own straight-through convention, and scores through the artifact's own integer kernel; `Copy`/`Stop` rows are frozen at the artifact's values so the loss is comparable with the served one.

| scope | init dev | final dev | reference | verdict |
| --- | ---: | ---: | ---: | --- |
| served, all 4,098 legal rows over all 5,376 dev targets | **6.684105** (= the artifact's own served loss) | **6.552759** (top-1 0.2161) | artifact served **6.684105** | **−0.131315 bits** |
| matched K-class, 256 rows | **4.779259** (= `A_kclass`) | **4.635738** | `A_kclass` **4.779259** | **−0.143522 bits** |

**In both scopes the integer realisation equals the float surrogate**, so the ternary codes, the per-row power-of-two scales and the integer bias are **not** the constraint: the served alphabet expresses what the fit found. The pre-declared bar of **≥0.30 bits is not met** — a frozen-state output-layer refit buys about a quarter of the 0.4634 bits those states support; the rest needs different states.

**Controls (sealed, delivered source):** kernel cross-check `0.0` on the declared 64-position sample; frozen-bias ablation 4.6232 / 6.5544 (the learned bias contributes ≈0.01 bits, so the gain is in the vocabulary rows); permuted-target null 6.4188 / 8.2744 against floors 4.7793 / 6.6841, `ok=true`, with the floor **corrected** to `min(its own initialisation, the restricted unigram)` because an artifact-initialised refit already beats the unigram; both alphabets' round trips and the straight-through gradient unit-tested; 7 runner tests and 12 `transferable_lexical` tests pass; `cargo fmt --check`, the offline release build and the claim-wording gate pass; all edited-document links resolve.

## 3. Retraction of this milestone's own first draft

An earlier draft of the result document — and the public entrypoints it propagated — concluded that *"the fit is not the limit; the integer realisation is"*, quoting a ≈0.27-bit realisation cost. **That was wrong and is withdrawn.** It was inferred from a single instrument generation whose refit initialisation sat **0.14–0.21 bits above** the artifact's own readout. With an exact initialisation the realisation cost vanishes and the integer result equals the float surrogate. The defect was found by the implementation round auditing its own construction; the delivered source carries the fix and the assertion. The lesson is recorded: **a comparison whose initialisation is not exact cannot scope a negative to a mechanism.**

## 4. Scope retraction of the previous milestone

Its post-hoc re-pricing figures (8.6058 ternary, 7.6181 ternary+offsets, 8.9683 four-bit) came from a **degenerate quantisation** — code density 0.0088, 614 of ~70k codes non-zero, logits in [−18, 5]. At K=256 with a working pre-scale the same machinery gives density 0.30 and a usable readout, so those numbers are **not** evidence about the 4-bit or ternary alphabets. This narrows an instrument claim; the headroom measurement is unaffected.

## Limits

Scoped to `olx-form-2`, the served window conditioning, teacher-forced states, one corpus, one split, one seed, 6 epochs and 75% of the fit population visited; the K-class figures cover 69.4% of dev targets and the served-scope figures all 5,376 over all 4,098 rows. **Grounded-panel retention is unevaluated for a refit readout** — the panels need a readout inside a model, which a runner-side refit is not. No general-language, reasoning or geometric-advantage claim; generation still collapses; sampling NOT_RUN; energy UNAVAILABLE. The refit is an offline diagnostic in the served alphabet and not a serving path or product dependency. The model source `learner/transferable_lexical.rs` is unchanged (`sha256 03f83eb8…b250`).

## Resources

Machine time ≈3,900,000 ms of the **8,200,000 ms** charged; ledger cumulative **422,915,640 / 427,000,000 ms**. Three ledger extensions were recorded prospectively before the work they covered. The overrun cause is recorded twice now and has three named remedies in the ledger: sequence an independent architecture review before implementation; design for the instrument's convergence — and for the artifact-initialised side of every comparison — in the first version; and price the four-minute state-recording phase into the arm count before dispatching. No paid or external compute; no artifact, preserved root or owner checkout modified. One arm (`olx-refit-1`) was stopped deliberately before sealing rather than paying for a redundant third pass, and is labelled unsealed wherever quoted.

## One evidence-supported next milestone

**Take the refit out of the frozen state.** A frozen-state refit of the output layer buys ~0.13 bits while the same states support 0.4634 under a matched float readout, so ~0.33 bits are unreachable without changing the states the readout is fit on. One **joint** fit on the served conditioning with the output layer initialised from the converged refit (optionally an output-focused schedule), keeping grounded Copy/Stop supervision, pre-declaring **served loss below 6.6841 by ≥0.30 bits with preflight A 32/32, held-out 3/3 and class 4/4 retained**, and reporting the matched reference, the float surrogate and the integer realisation so a shortfall is attributable. If that also stalls near 0.13 bits, the next branch is the input side: explicit **second-order local access**, since a training-free decode recovers `cur` at 96.7% and `prev` at 2.1%.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

