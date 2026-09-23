# No material, tune-robust complementarity: the count prior is at least as good locally — September 23, 2026

> **K3 architecture review (2026-09-23).** The **decision below is sound**, but the headline is a **full-split
> point estimate**: on the 92.15 % frequent-token head the tune-frozen pool *beats* `C` by **−0.0417
> [−0.0686, −0.0106]** (interval excludes zero) while it loses ≈**+0.552** bits/target on the 422 rare targets.
> The honest statement is "**sub-threshold, sub-cost and tail-harmful**", not "absent". The full correction and
> the cost arithmetic that actually carries the decision are recorded below.

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

## K3 architecture review (2026-09-23) — corrections to the reading

An independent K3 architecture review was obtained after the merge. It **confirmed the decision** but required
four corrections to the reading:

1. **Tune→dev collapse (the actual content of "not tune-robust").** At the chosen `λ*=0.8` the pool beats `C` by
   **0.0736** bits on the 114,364-position tune split (`5.915503 − 5.841928`) and only **+0.0049** on dev — a
   ~**0.078-bit** collapse. The tune argmin is itself population-sensitive: the unmerged investigation's
   8,512-position tune sample selects `β=0.9`, not 0.8. This instability is why neither tune-frozen number
   should be over-read.
2. **Head vs tail.** On the K=1024 head (4,954 targets) the tune-frozen pool beats `C` by **−0.0417
   [−0.0686, −0.0106]**; on the 422 rare targets it loses ≈**+0.552** bits/target. "The count prior is at least
   as good locally" is true of the **full-split point estimate**, not of the frequent-token decision population.
3. **The decision's stated ground is cost, not absence.** An exact addressed local read is a `(prev,cur)` table
   of ≈`4096×4096` plus a `cur` table — ≈**8–16 MiB at 4-bit against the 466,711-byte artifact (~17–35×)** — and
   the measured pool buys only ≈0.04–0.08 bits over serving `C` alone. `C` alone beats `A` by **1.562381** bits
   but remains a *comparator*, not the target, because it has no memory/generation/geometry story — which is the
   programme's actual reason for not serving it. Stated this way "do not build" is clearly correct.
4. **Live contested evidence.** An unmerged post-merge investigation (`codex/observer-transport-20260923`,
   commits `be40da9b`/`0c1b715b`) reports, on the same dev split and states, a two-parameter `C^α A^γ`
   tune-selected combination at **5.041423** vs calibrated counts **5.079035** → **−0.037612
   [−0.057254, −0.016462]** (19/24 documents), and a **transport-aware decode recovering `prev` at 75.90 %**
   versus the merged raw-embedding 2.1 %. This is **unmerged and not yet reproduced on main**; it is registered
   here as contested evidence. Its adjudication is the ordered next milestone (see below), and it is why the
   next step is *not* the longer-context control.

The K3 review also independently verified the exact criterion symbolically (`b'(1) = (b_C − b_A) + KL₂(p_C‖p_A)`
identically; `d²log₂Z/dλ² = ln2·Var_{p_λ}[log₂(p_C/p_A)] ≥ 0`), confirmed the −0.025181 oracle depth is the
curvature-implied value for `b'(1)=0.553`, and matched the observer's state SHA to the sealed probe receipt.

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
geometric-advantage or energy claim; greedy generation still collapses; energy UNAVAILABLE. **A K3 architecture
review was obtained (2026-09-23)**: it confirmed the decision and required the wording/scope corrections recorded
above; the independent evidence audit's corrections are also incorporated.

## Resources

One diagnostic run, 189.8 s wall (one process, ≤2 threads, ≪1 GiB). Two release smoke runs (~156 s) plus this
run are charged **346,000 ms**; cumulative **436,825,582 / 437,500,000 ms**. New retained storage ≈2.4 MB in the
sealed root; the 128 MiB stop margin is untouched. No training, no artifact, no paid/external compute.

## One evidence-supported next milestone (ordered by the K3 review)

**Adjudicate the contested complementarity, and settle the state-content contradiction — before the
longer-context control.** The K3 review ordered one **frozen adjudication + transport-decode depth ladder** run
(no training):

1. **Frozen two-parameter adjudication.** Re-run the one- and two-parameter blends under one pre-declared
   protocol: the full 114,364-position tune split, declared grids (including `λ ∈ {0.925, 0.95, 0.975}`),
   comparisons against raw `C` **and** calibrated `C`, 24-document intervals, mandatory rare-tail accounting,
   and fresh-process byte-verification. This settles whether `codex/observer-transport-20260923`'s −0.0376-bit
   tune-frozen result reproduces.
2. **Context-classed `λ`.** Oracle and tune-frozen `λ` per context-frequency class, classes frozen before dev is
   read; the head/tail split is the signature a fixed `λ` cannot follow.
3. **Transport-decode depth ladder** on the identical recorded states (`c5334071…992f`): `x_(t-2)`, `x_(t-3)`,
   `x_(t-4)` with the existing rotated-label and wrong-frame controls. This settles the state-content
   contradiction and pre-bounds what any longer-context channel could supply.

The longer-context control remains the right question **after** these, because its design depends on whether the
state carries contracted history beyond the two tokens `C` sees.
