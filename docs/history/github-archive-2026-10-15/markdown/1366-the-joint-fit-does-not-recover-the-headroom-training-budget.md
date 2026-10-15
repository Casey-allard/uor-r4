# #1366 — The joint fit does not recover the headroom: training budget is not the constraint (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T16:51:56Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-23T17:27:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1366
- **Merged:** 2026-09-23T17:27:24Z
- **Merge commit:** ba6645855d7b20eab73f781fc9975b27b482c867
- **Base / Head:** main / UOR-Foundation:codex/joint-fit-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the named next step after the frozen-state refit: test whether the shared grounded learner is **under-trained** by fitting it longer on the served window conditioning with the repaired curriculum.

`References #820` and `References #973`. Nothing is closed.

**Stacked PR — fourth in a chain.** Base is the head of open PR #1365 (`0da2d61e`); merge in the order **#1363 → #1364 → #1365 → this one**. New work here is one commit, `05d35fc8`.

## The arm

`olx-joint-1`: the prose-only window-objective recipe of `olx-form-2` (the best prior prose arm) at **4,000 steps** — double its 2,000 — with everything else identical: same corpus and split, same seed 13, lr 0.02, batch 4, interleaved curriculum, no grounded draws. **No code change**; the existing `run()` path with the existing declared flags. Model source unchanged (`sha256 03f83eb8…b250`).

## Result: a scoped negative — the model has converged operationally

| arm | steps | served bits/target | paired doc-cluster interval vs `olx-form-2` |
| --- | ---: | ---: | --- |
| `olx-form-2` (best prior prose arm) | 2,000 | **6.684105** | — |
| `olx-joint-1` (this run) | 4,000 | **6.708963** | **+0.0249 [−0.0422, +0.0967]** |

- Doubling the steps changed nothing: the paired interval **includes zero**, and the direction is slightly worse if anything.
- The development probe **oscillates** between 6.6318 and 6.9169 (median 6.7843) across steps 2,000–3,900 instead of descending — the 2,000-step endpoint is reached and wandered through.
- Gains against the retained references: tuned `(prev, cur)` count −1.5872, donor E +0.1612, unigram +2.3737.

The pre-declared bar — served loss below `olx-form-2`'s 6.6841 by ≥0.30 bits — is **not met**, so the contingent grounded-supervision arm was not run (a prose-only arm cannot pass panels; `preflight A` correctly reports 0/32).

## What this settles

| output-side lever tested, in order | measured served gain |
| --- | ---: |
| in-class readout refit on the frozen state (exact init) | −0.1313 |
| joint fit at double the training budget | +0.0249 (nothing) |
| matched float-readout headroom on the same states (for scale) | 0.4634 available in principle |

The two cheapest output-side levers are exhausted at about a quarter of the available headroom, and a longer schedule buys nothing. **The binding constraint is not optimisation, training budget or the served alphabet** — it is the *interface*: a rank-≤64 linear readout of a 64-dimensional state. The state's content is already measured: a training-free decode recovers the immediately preceding token at **96.7%** and the one before it at **2.1%**, so first-order local context is present and second-order context is not.

## Limits and claim boundaries

Measured: `olx-joint-1`'s served loss on the complete development split, the paired interval against `olx-form-2`, and the trajectory. **Scoped to** one pinned local corpus, one split, one seed, one schedule family, the served window conditioning, prose-only supervision, and 2,000–4,000 steps of this recipe — it does not rule out a very different recipe. No general-language, reasoning or geometric-advantage claim; greedy generation still collapses; sampling NOT_RUN; energy UNAVAILABLE.

## Resources

One fit, **1,410.6 s** of measured machine time (the runner's own sealed `fit_seconds`), one process, two threads. The milestone came in **under** its projection because it needed only one fit and no code change, and it deliberately avoided the probe's four-minute state-recording phase. Complete milestone charged **2,500,000 ms**; ledger cumulative **425,415,640 / 431,500,000 ms**. Physical free 53,048,876 KiB, above the 24 GiB reserve plus the 128 MiB stop margin. No paid or external compute; no artifact, preserved root or owner checkout modified.

## One evidence-supported next milestone

**The count-prior blend.** On the same 5,376 development targets and the served window conditioning, measure the loss of a **log-linear blend of the artifact's served readout with the retained tuned `(prev, cur)` count prior**, tuned on the tune split, with per-document paired intervals, distinguishing **complementarity** (the blend beats both) from **domination** (it collapses to the count prior). It is a cheap measurement — both quantities are already computed in the runner — and it sets the form of any second-order local-access change before one is built: strong complementarity means the local prior belongs inside the served readout as an exact addressed read; domination means the retained comparator is the right local tool.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

