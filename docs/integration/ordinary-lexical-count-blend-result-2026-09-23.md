# No material, tune-robust complementarity: the count prior is at least as good locally — September 23, 2026

This record executes the named next milestone after the joint fit: measure, on the served window conditioning,
whether a **log-linear (geometric) pool** of the frozen artifact's served readout `A` with the retained tuned
`(prev, cur)` count prior `C` is **complementary** (beats both) or **dominated** (collapses to `C`). The
milestone decides whether an exact addressed local read belongs inside the served readout **before** one is
built.

Base `34bbe2ae`; new work on branch `codex/local-capacity-adjudication-20260923`. Model source unchanged
(`learner/transferable_lexical.rs` sha256 `03f83eb8…b250`); artifact unchanged
(`olx-form-2/artifacts/model.tlx` sha256 `f89d4499…4aaa9`). Runner-only addition: a default-off
`--count-blend` mode. Sealed root `olx-count-blend-1`; evidence
[`docs/evidence/ordinary-lexical-count-blend-2026-09-23.json`](../evidence/ordinary-lexical-count-blend-2026-09-23.json).

## Design (established before the run)

For context `c = (prev, cur)`, the artifact's **token-conditional** readout `p_A(x|c)` (rows `0..4096` of
`TlModel::readout`; the `Stop` row is renormalised out and reported separately, mean `2.0212e-6`) and the count
prior `p_C(x|c) = family_p(c1,c2,uni,prev,cur,x,lambdas)` are both distributions over the **same** 4096 tokens.
The pool is `p_λ ∝ p_A^{1-λ} p_C^λ`. Two exact facts frame the test (verified symbolically and, here,
numerically):

- **Convexity.** `b(λ) = −E[log2 p_λ(target)] = (1−λ)b_A + λ b_C + E_c[log2 Z_c(λ)]` is convex in `λ`, because
  `Z_c` is a sum of exponentials of affine functions in `λ`.
- **Exact oracle criterion.** `b'(1) = (b_C − b_A) + E_c[KL_bits(p_C‖p_A)]`; with `b(1)=b_C` and convexity,
  an in-sample `λ` beats `C` **iff** `E_c[KL_bits(p_C‖p_A)] > b_A − b_C`. The `KL` side is label-free.

Pre-declared decision rule: **complementarity** requires the tune-frozen pool to beat `C` by **≥ the standing
0.10 bits/target programme threshold** with the paired 95% document-cluster interval entirely below zero;
**domination / no material complementarity** otherwise, with the dev oracle reported as an explicitly
in-sample bound. Attribution controls were fixed in advance: `blend(E1,C)` (bounds the one-token channel the
state already carries at 96.7%), a duplicate-count null, and the K=1024 subpopulation alongside the full split.

## Result

Full development split (5,376 targets; 24 documents), tune-frozen `λ* = 0.8` (chosen on 2,048 tune windows /
114,364 positions before any development number was read):

| predictor | bits/target |
| --- | ---: |
| artifact served readout `A` | 6.684102 |
| tuned `(prev,cur)` count prior `C` | 5.121721 |
| tuned `E1` (unigram + bigram(`cur`)) | 5.922406 |
| **tune-frozen pool (λ\*=0.8)** | **5.126648** |
| in-sample oracle pool (λ=0.9) | 5.096540 |

- **The exact criterion is satisfied, so an in-sample oracle can beat `C`.** `gap = b_A − b_C = 1.562381`,
  `mean_kl = 2.115408`, `b'(1) = +0.553026 > 0`.
- **But the effect is not material and not tune-robust.** The tune-frozen pool is **+0.004928
  [−0.026228, +0.040527]** against `C` on the full split — the interval includes zero and the pool is
  *slightly worse* than `C`. Only the in-sample oracle (`λ=0.9`, chosen with knowledge of the development
  data) beats `C`, by **−0.025181 [−0.040195, −0.007841]** — **a quarter of the 0.10-bit threshold**.
- **The effect is population-dependent.** On the K=1024 subpopulation (4,954 targets) the tune-frozen pool is
  **−0.041691 [−0.068550, −0.010625]** against `C` (interval excludes zero, below the threshold), while on the
  **422 rare targets outside the top-1024 it is ≈ +0.552 bits/target worse** than `C`. So the pool marginally
  helps the frequent head and materially harms the rare tail.
- **No complementarity against the one-token channel.** `blend(E1,C)`: `mean_kl(p_C‖p_E1) = 0.554076 < gap
  0.800685`, so the criterion fails; the oracle `λ=1.0` is exactly `C`. `C` already dominates the one-token
  information the state carries.
- **The pooling control bounds the mechanism but is not a matched null.** The duplicate-count null (two
  disjoint half-fit `(prev,cur)` estimators of the *same* conditional) gives an oracle gain of
  `−0.219296` — larger than the primary `−0.025181`, but its arms each use half the data and are individually
  much worse (`b_half1 = 5.442558`, `b_half2 = 5.539463` vs `b_C = 5.121721`). The **matched** secondary
  reading (full `C` pooled with half-1 `C`) gives only `−0.007590 < 0.025181`. The pooling band is therefore
  wide (`≈0.008–0.219`) and the primary gain sits inside it; the null does **not** by itself prove the gain is
  denoising.

## Verdict and exact scope

**No material, tune-robust complementarity was demonstrated.** The retained tuned `(prev,cur)` count comparator
is at least as good as the log-linear pool on the full development split and materially better on the rare-token
complement; the only interval-excluding-zero tune-frozen effect (`−0.042` on the fit-derived K=1024
subpopulation) is below the standing 0.10-bit threshold and is accompanied by a `+0.55`-bit regression on the
complement. **Decision: do not build an exact addressed local read inside the served readout on this evidence.**

Scope: one frozen artifact, one pinned corpus, one fit/tune/development split, seed 13, the served window
conditioning, prose-only supervision, and **the log-linear pool protocol**. The negative does **not** refute a
different local-access mechanism (e.g. an input-side second-order channel, a learned local read interacting with
the recurrent map, or a different pool), and it does not rule out the recurrence family. The in-sample oracle is
labelled as such and is never quoted as the achieved result.

## Controls, reproduction and instrument notes

- **Independent reproduction of the merged anchors.** Full `C` **5.121721** and restricted `C` **4.784593**
  and restricted `E1` **5.565123** reproduce the merged state-probe values **exactly**. `A` reproduces to
  `≈2.9e-6` bits: the receipt uses the **token-conditional** `A` (Stop renormalised out), worth exactly the
  reported `p_stop` mass, whereas the merged record's `6.684105` includes Stop in the denominator. The residual
  is criterion-invariant (four orders below the threshold). The `p_A` normalisation guard is self-referential
  (`Σ p_A = 1` by construction); the `p_C`/`p_E1`/half guards are non-trivial and passed at `≤6.6e-14`.
- Instrument controls: recorded-state bits reproduce `score_example`; the `family_p` dense replica is unit-tested
  equal over all 4096 tokens for seeded contexts; `λ=0/1` reproduce `A`/`C`; the curve is convex; determinism is
  fixed (seeds, grid, tie rule) and two independent release runs agreed byte-for-byte on the decisive arrays.

## Limits

In-sample oracle labelled; 24-cluster percentile bootstrap (BCa/studentized not applied); K=1024 set is
fit-derived; the pooling control is not sample-size-matched (see above). No general-language, reasoning,
geometric-advantage or energy claim; greedy generation still collapses; energy UNAVAILABLE. **No K3 architecture
review was obtained for this claim** because it is a conservative scoped negative with no promotion; the
independent evidence audit was used instead and required the wording corrections recorded above.

## Resources

One diagnostic run, 189.8 s wall (one process, ≤2 threads, ≪1 GiB). Two release smoke runs (~156 s) plus this
run are charged **346,000 ms**; cumulative **436,825,582 / 437,500,000 ms**. New retained storage ≈2.4 MB in the
sealed root; the 128 MiB stop margin is untouched. No training, no artifact, no paid/external compute.

## One evidence-supported next milestone

**Re-ask the local question, because the pool is not the missing mechanism.** The two-token count prior is a
stronger local predictor than the served readout and no material complementary gain survives a frozen pool, so
local prediction is bounded by an ordinary table at this state. The decisive next question is therefore **what
the learned recurrence supplies that the `(prev,cur)` table cannot**: measure the artifact's loss conditioned on
*longer* context (beyond the two tokens `C` sees) against an ordinary information-matched longer-context count
control, and test whether a bounded historical/interference signal exists at all. That is the honest fork the
domination result leaves open, and it comes before any local table, bypass or geometric transport is built.
