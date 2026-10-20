# The joint fit does not recover the headroom: training budget is not the constraint — September 23, 2026

This record executes the named next step after the frozen-state refit: a **joint fit** on the served
window conditioning, to test whether the shared grounded learner is **under-trained**. The question was
deliberately narrow and falsifiable, with the bar declared in the previous record: a longer joint fit must
beat `olx-form-2`'s 6.6841 by **≥0.30 bits**, or training budget is ruled out as the binding constraint and
the deficit belongs to the state/readout interface.

Base `67f2e350` plus open PRs #1363/#1364/#1365; new work on branch `codex/joint-fit-20260923`. The model
source `learner/transferable_lexical.rs` is **unchanged** (`sha256 03f83eb8…b250`). No new code: this used
the existing `run()` machinery with the existing declared flags, so the default path and every prior arm
are unaffected. Evidence:
[`docs/evidence/ordinary-lexical-joint-fit-2026-09-23.json`](../evidence/ordinary-lexical-joint-fit-2026-09-23.json).
New sealed root: `olx-joint-1`.

## The arm

`olx-joint-1`: the prose-only window-objective recipe of the best prior prose arm (`olx-form-2`) at **4,000
steps** — double the 2,000 steps that arm used — with everything else identical: same corpus and split,
same seed 13, lr 0.02, batch 4, interleaved curriculum, no grounded draws. Cost 1,410.6 s of fit.

| arm | steps | grounded draws/step | served bits/target | matched doc-cluster interval vs `olx-form-2` |
| --- | ---: | ---: | ---: | --- |
| `olx-form-2` (best prior prose arm) | 2,000 | 0 | **6.684105** | — |
| `olx-joint-1` (this run) | 4,000 | 0 | **6.708963** | **+0.0249 [−0.0422, +0.0967]** |
| `olx-form-3` (window + 2 grounded, for reference) | 2,000 | 2 | 6.968576 | — |
| delivered artifact | 2,000 | delivered schedule | 7.372198 | — |

References on the same targets: unigram 9.0827, tuned `(prev, cur)` count **5.1217**, donor E 6.8701.

## Result: the pre-declared bar is not met, and the model has plateaued

- **Doubling the steps changed nothing.** `olx-joint-1` − `olx-form-2` = **+0.0249 bits**, with the paired
  document-cluster interval **[−0.0422, +0.0967] including zero**. There is no measurable improvement; the
  direction is, if anything, slightly worse.
- **The trajectory oscillates rather than descends.** Across steps 2,000–3,900 the development probe moved
  between **6.6318 and 6.9169 (median 6.7843)** with no trend; the 2,000-step endpoint of the prior arm is
  reached and then wandered through.
- Both gains against the retained references move accordingly: against the tuned count the gap widened to
  −1.5872; against donor E the small advantage held (+0.1612); against unigram +2.3737.

So on this corpus, split, seed and objective, **the served prose ceiling of the shared learner is
≈6.68–6.71 bits/target regardless of training budget, and the model has converged in the operational
sense.** Combined with the two previous milestones:

| output-side lever tested | measured served gain |
| --- | ---: |
| readout refit **in class** on the frozen state (exact init) | **−0.1313** |
| joint fit at **double** the training budget | **+0.0249 (nothing)** |
| matched float-readout headroom on the same states (for scale) | **0.4634** available in principle |

The two cheapest output-side levers are now exhausted at ≈0.13 bits of the available headroom, and a
longer schedule buys nothing. **The binding constraint is therefore not optimisation, training budget, or
the served alphabet.** What remains is the *interface*: the readout is a rank-≤64 linear map of a 64-dim
state, and the state does not present the local conditional structure to it in a form it can use as
sharply as a table can.

## The decisive next measurement (named, not executed here)

The previous milestones measured the *state's content*: a training-free decode recovers `cur` at 96.7% and
`prev` at 2.1%, so the immediately preceding token is present in the state but the one before it is not.
What has not been measured is whether giving the readout **direct access to the local context the count
model uses** closes the local gap, and whether the recurrent state and the local prior are complementary
or redundant. That is the decisive, cheap check for second-order local access, and it is the named next
milestone below. It is deliberately a **measurement before a build**: it should decide the form of any
local-access change before one is written, because a comparison whose initialisation is not exact cannot
scope a negative to a mechanism.

## Limits and claim boundaries

Measured: `olx-joint-1`'s served loss on the complete development split; the paired interval against
`olx-form-2`; the trajectory. **Scoped to** one pinned local corpus, one fit/tune/development split, one
seed, one schedule family, the served window conditioning, prose-only supervision, and the existing
trainer. The bar `≥0.30 bits` was declared in the previous record and is **not met**; the negative holds
for this objective and budget range (2,000–4,000 steps) and does not rule out a very different recipe. No
general-language, reasoning or geometric-advantage claim; greedy generation still collapses on this arm;
sampling NOT_RUN; energy UNAVAILABLE. The grounded panels were intentionally not run on a prose-only arm;
`preflight A` correctly reports 0/32.

## Resources

One fit, 1,410.6 s of machine time; one process, two threads. The complete projection, the prospectively
recorded extension and the final charge are in the [resource ledger](resource-ledger-2026-09-19.md). No
paid or external compute; no artifact, preserved root or owner checkout modified.

## One evidence-supported next milestone

**Measure the count-prior blend on the served path.** On the same 5,376 development targets and the served
window conditioning, compute the loss of a **log-linear blend of the artifact's served readout with the
retained tuned `(prev, cur)` count prior**, tuned on the tune split, and report it with per-document
paired intervals against the artifact and against the count reference, distinguishing
**complementarity** (the blend beats both) from **domination** (the blend collapses to the count prior).
This is the decisive check for second-order local access and it sets the form of any subsequent change:
if the blend is strongly complementary, the local prior belongs inside the served readout as an exact
addressed read; if the count prior dominates, the honest conclusion is that the retained comparator is the
right local tool and the architecture's local question must be re-asked. It is a cheap measurement — both
quantities are already computed in the runner — and it must be exact before any table or bypass is built.
