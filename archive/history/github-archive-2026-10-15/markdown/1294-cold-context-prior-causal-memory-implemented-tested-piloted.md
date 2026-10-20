# #1294 — Cold-context prior + causal memory: implemented, tested, piloted — all three gates fail (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T00:44:49Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-09-20T00:45:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1294
- **Merged:** 2026-09-20T00:45:53Z
- **Merge commit:** c0ca7482782a5d5e11d25aa8c6d5b03f0ce4cd07
- **Base / Head:** main / UOR-Foundation:codex/cold-context-prior
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements and pilots the mechanism selected by the [cold-context review](docs/integration/cold-context-review-2026-09-19.md). **All three predeclared gates fail at the specified dose.** The result is a negative/inconclusive pilot, reported as one — no capability is claimed.

## What was built

`learner/cold_prior.rs`: an always-present learned **exact-token local prior** (`E_old` with an explicit absent-prefix marker row at `pad_row(vocab)`, outside the real id range; `E_new`), the **existing modulo-pair causal memory residual**, **one shared low-bit decoder** plus an exported integer bias, per-channel power-of-two normalisation applied *before* the sum (so a large memory read cannot erase a small prior), a bounded ReLU, and declared clamps. The prior is present on cold **and** warm routes; the memory term is exactly zero when unwritten and may be signed when written. `full` / `memory-disabled` / `prior-disabled` / `bias-only` are runtime channel selections, not learned gates. Artifact format `CPR1` binds version, vocab, dv, norm bits, channels, clamps, seed and a tokenizer digest; reload is **byte-exact** with integer-logit parity tested.

13 focused tests pass: absent marker vs real token zero, equal-residue ids separable in the prior, order reversal, no future leakage, reset isolation, same tail with changed earlier writes, signed reads, **cold == memory-disabled exactly**, clamp boundaries, export/reload parity, surrogate gradient reaching both new tables, and a real loss reduction on a repeating context.

## Instrument repairs (review findings, all executed)

- `blended_argmax` seeded its incumbent with the *unmixed* unigram probability and compared it against mixed candidates — fixed; the review's example now selects the right class.
- `reference_order2` re-applied the value row scale. **The runtime was right; the reference was wrong.** Removed, and the fixture now sweeps unit/nonunit row scales × normalization off/active over two lengths — unit-scale fixtures alone could not see it.
- `norm_bits = 0` documented as **disabling** the shift.

## Pilot (V=4096, dv=128, order=2, batch 8, window 64, 256 steps/arm, matched init + optimizer)

One fit-only 4096-vocabulary derivation; one document-separated manifest; corpus `sha256:fe36c2c6…`; open development data.

| | bits/target |
|---|---:|
| matched fit-only unigram | 6.4019 |
| quantized constant control | 6.4522 |
| A prior-only | 6.7011 |
| B prior+memory | 6.8941 |
| full / memory-off / prior-off / bias-only | 6.8941 / 6.7006 / 6.8982 / 6.6709 |

Memory gain on **warm** targets: **−4.4072** bits/target (10.2981 vs 5.8909, n=294).

**Gates:** 1 −0.2993 / −0.2489 FAIL · 2 −4.4072 FAIL · 3 −0.1930 FAIL. Paired document-level difference +0.2489, **90% CI [−0.0340, +0.5056] over 18 documents — sign unresolved**. Warm coverage 4.39% → gate 2 reported **unresolved**, not as a memory verdict.

## Diagnosis

- **Activation scale healthy** (mean |logit| 6.2–6.4; not saturated).
- **Bias not converged** (bias-only 6.6709 vs analytic constant 6.4522).
- **Dose:** 131,072 training tokens is *less than one pass* over 1,460,658 fit targets against 1,048,576 new prior parameters → the prior gate is **undertrained/inconclusive**, not a family-wide negative.
- **Memory harm is separate and measured** (−4.41 bits/target on warm targets): the modulo-aliased fast memory at this dose is confidently wrong. That is a statement about this mechanism as configured, not about exact occurrence/version memory.

## Retained and reproducible

Artifact 614,542 B `sha256:fb780ff6ff19eec58f861135004250ae8874107099e2a071f7bfd1b4318b5a39`; export/reload byte-exact and integer-logit parity PASS. Complete deterministic continuations are printed for **all four interventions without selection** (full `[35,28,198,46,28,28,28,28,39,28,28,39,39,…]`; memory-off collapses to token 28; prior-off/bias-only alternate 28/39). Token 28/39 dominate everywhere: a crude local alternation, nothing resembling prose.

## Validation

`cargo fmt --all --check` clean · `python3 scripts/check_claim_wording.py` passed · `cargo test --release -p uor-r4-core --lib -- cold_prior the_mean_loss_ste the_kernel_gradient` **15 passed, 0 failed**.

## Not run / not claimed

No `crates`-corpus replicate, no second seed, no learning-rate/width sweep, and **no binary/whole-path multiplier audit** of the new integer path (`serving_multiplier_check.py` was not run on it — source-level inspection only). No geometry claim: the order-2 radix pair read exercises no group multiplication, zeta phase or chirality. No chat, coding, reasoning or energy claim. Energy/token UNAVAILABLE.

Ledger: 1,400,000 ms charged (measured) → `149,038,565 / 154,400,000`, ~89.4 min remaining; 128 MiB storage margin untouched; no deletion, no paid compute.

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

