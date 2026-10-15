# #1368 — No material, tune-robust complementarity: the count prior is at least as good locally (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T21:14:12Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-09-23T21:15:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1368
- **Merged:** 2026-09-23T21:15:19Z
- **Merge commit:** 4b638b2e95d3ccf2c76a9cfd304544122deacd32
- **Base / Head:** main / UOR-Foundation:codex/local-capacity-adjudication-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

Adds a default-off `--count-blend` diagnostic to the `ordinary-lexical` runner and records the named next
milestone after the joint fit: the **count-prior blend**. It measures the log-linear (geometric) pool of the
frozen artifact's token-conditional served readout `A` with the tuned `(prev, cur)` count prior `C` on the served
window conditioning, decides **complementarity vs domination**, and answers whether an exact addressed local read
belongs inside the served readout.

Runner-only: model source is unchanged (`learner/transferable_lexical.rs` sha256 `03f83eb8…b250`); artifact
unchanged (`olx-form-2/artifacts/model.tlx` sha256 `f89d4499…4aaa9`). No training, no artifact promotion, no
serving-path change. Base `34bbe2ae`.

## Result (scoped negative)

Full development split (5,376 targets, 24 documents), tune-frozen `λ*=0.8` (chosen on 2,048 tune windows before
any development number was read):

| predictor | bits/target |
| --- | ---: |
| artifact served readout `A` | 6.684102 |
| tuned `(prev,cur)` count prior `C` | 5.121721 |
| tuned `E1` | 5.922406 |
| tune-frozen pool (λ\*=0.8) | **5.126648** |
| in-sample oracle pool (λ=0.9) | 5.096540 |

- The **exact oracle criterion** is satisfied (`gap = 1.562381`, `mean_kl = 2.115408`, `b'(1) = +0.553026 > 0`),
  so an *in-sample* `λ` can beat `C`.
- But the tune-frozen pool is **+0.004928 [−0.026228, +0.040527]** against `C` (interval includes zero), the
  in-sample oracle gain is only **−0.025181**, and on the **422 rare targets outside the top-1024 the pool is
  ≈+0.552 bits/target worse**. On the K=1024 subpopulation the tune-frozen pool is −0.041691
  [−0.068550, −0.010625] — below the standing 0.10-bit threshold.
- **`blend(E1,C)` control** fails (`mean_kl 0.554076 < gap 0.800685`); oracle `λ=1.0` is exactly `C`, so the
  count prior already dominates the one-token information the state carries.
- The duplicate-count null gives a larger oracle gain (−0.219) than the primary (−0.025), but its arms are
  half-data; the matched secondary reading is only −0.0076. The pooling band is wide and the primary gain sits
  inside it, so the null does not by itself prove denoising.

**Decision: no material, tune-robust complementarity; do not build an exact addressed local read inside the
served readout on this evidence.** Scope is the log-linear pool protocol on one artifact/corpus/split; it does
not refute a different local-access mechanism or the recurrence family.

## Reproduction and controls

- Full `C` **5.121721**, restricted `C` **4.784593** and restricted `E1` **5.565123** reproduce the merged
  state-probe values **exactly**. `A` reproduces to **≈2.9e-6 bits** because the receipt uses the
  token-conditional readout (Stop renormalised out, reported separately, mean 2.0212e-6); the residual is
  criterion-invariant.
- Recorded-state bits reproduce `score_example`; the dense `family_p` replica is unit-tested equal over all 4096
  tokens; `λ=0/1` reproduce `A`/`C`; the curve is convex; determinism is fixed and two release runs agreed
  byte-for-byte on the decisive arrays. 13/13 focused tests pass; `cargo fmt --check` clean.
- An independent evidence audit required wording corrections (recorded in the result): "domination" is not
  uniform (the pool marginally beats `C` on the frequent head), the pooling-null inference is not a matched
  bound, and the rare-tail regression must be reported. Direction of the decision is unchanged.

## Limits

In-sample oracle is labelled and never quoted as the achieved result; 24-cluster percentile bootstrap
(BCa/studentized not applied); K=1024 set is fit-derived; the pooling control is not sample-size-matched. No
general-language, reasoning, geometric-advantage or energy claim; greedy generation still collapses; energy
UNAVAILABLE. **No K3 architecture review was obtained** (conservative scoped negative, no promotion).

## Resources

One diagnostic run 189.8 s wall (one process, ≤2 threads, ≪1 GiB) plus two smoke runs; charged **346,000 ms**,
cumulative **436,825,582 / 437,500,000 ms**. New retained storage ≈2.4 MB in the sealed root
`olx-count-blend-1`; 128 MiB stop margin untouched. No paid/external compute.

References #820, #973. The next ordered step is to measure whether the artifact uses **longer** context than the
`(prev, cur)` table at all, against an ordinary information-matched longer-context count control.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

