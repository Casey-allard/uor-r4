# #2223 — The geometric read against its own ablation: ahead on every seed, aggregate wrong-value 49 -> 32 of 120 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T01:31:44Z
- **Updated:** 2026-10-12T01:42:37Z
- **Closed:** 2026-10-12T01:37:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2223
- **Merged:** 2026-10-12T01:37:41Z
- **Merge commit:** 2d75ca71d9a06d336e93f750f97487503ce8301b
- **Base / Head:** main / UOR-Foundation:deepseek/geometric-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# The geometric read against its own ablation: ahead on every seed, aggregate wrong-value 49 → 32 of 120

References #2029 (M1 acceptance; pre-registered in [#2029 comment 6115054306](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6115054306)).
Lab: DeepSeek, session `deepseek/georead2`. Runs 2026-10-12 00:26–01:10 UTC: **one 2-GPU pod, six 8,000-step fits at 585–640 s, ≈ $2.00**,
released and deleted at the last fit. Panel scoring local.

> **One line.** The geometry audit found geometric operations at **0.05 %** of the trained stack's arithmetic and the deployed model's read at
> **`l2` — the flat Euclidean control of Lorentz**. Six cycles since tuned supervision, placement, kernels and schedules *around* that read without
> ever testing it. This cycle tests it, matched and pre-registered: **`lorentz` (the geometric score) against `l2` (its own ablation), three seeds
> each, the read the only difference**. Result: **wrong-value failures 49 → 32 of 120 (−35 %), memory 48 → 59 of 120 (+11), ahead on all three
> seeds** (per-seed ΔWV −4 / −9 / −4), with seed 2 individually significant (p = 0.049, 13 rows repaired against 4 broken) and **34 rows repaired
> against 17 broken overall**. **The pre-registered per-seed margin (≥5 fewer failures on ≥2 of 3 seeds) is met on 1 of 3** — the direction is
> unanimous, the magnitude clause is not — so this is reported as **positive but below its own bar**, the mechanism stays open, and the geometric
> read is the configuration the line's next run should carry.

## The measurement

| pair | seed | `lorentz` wrong-value | `l2` wrong-value | repaired | broken | **p** (exact McNemar) | `lorentz` memory | `l2` memory |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| L1 vs F1 | 20261009 | **11** | 15 | 12 | 8 | 0.5034 | **18/40** | 16/40 |
| **L2 vs F2** | 20261010 | **11** | 20 | **13** | **4** | **0.0490** | **20/40** | 13/40 |
| L3 vs F3 | 20261011 | **10** | 14 | 9 | 5 | 0.4240 | **21/40** | 19/40 |
| **aggregate** | — | **32** | **49** | **34** | **17** | — | **59/120** | **48/120** |

**Everything but the read's score is identical** — same base (`chat-29m-B-lr5e-4`), store, steps, batch, lr, schedule, policy, seed set and pointer setting;
the arms differ only in `read_override=lorentz` against `l2`, and **each arm's `config.read` was read from its own report before it was counted**
(the discipline the previous cycle's failed run taught). Development NLL, which is not the gate, was **mixed** (lorentz 0.3011/0.2709/0.2922 against
l2's 0.2656/0.3020/0.2878) — the panel, not the dev split, is what separated them, which is why the frozen panel is the pre-registered primary.

## What it establishes

1. **The geometry earns its place on this line — modestly and consistently.** The Lorentz score beats the flat control it replaces on **every seed**
   in both the failure mode and the headline, with an aggregate of −35 % wrong-value and +11 memory rows of 120. That is the first positive geometric
   result on the answer path in this lab's work, and it was obtained by *removing* a mismatch the audit had found rather than by adding a mechanism.
2. **The magnitude is small.** One seed reaches the pre-registered margin (−9); the other two land at −4, inside the ±3-row noise band around it. A
   reader should take "the geometric read is better" from this, not "the geometric read is much better".
3. **It changes the line's default.** Every arm this lab has trained — including the read-binding keeper and the 16k runs — inherited `l2` from the
   base. The geometric read is now the configuration to carry forward, at zero architectural cost (the same weights, another score).

## Limitations

- **Three seeds, 40 memory rows each** (±3 rows) — the standing declared deviation from the brief's ≥ 120 rows; the aggregate is over 120 rows but each
  per-seed test is small, which is why only one seed reaches significance.
- **One pod, one GPU model**, 8,000 steps per fit; the arms are fine-tunes from the flat base, so "the geometric read" here means *the Lorentz score
  fine-tuned in from a checkpoint trained flat*, not a model pretrained geometric end to end.
- **`read_override` does not appear in the run's `settings` record** (the field is written on another branch of the record); the `config.read` of each
  report is the evidence that the intended score was trained, and it is quoted per arm above. That record gap is a small tooling follow-up.
- **The reply panel was not graded** (no local judge runs, per the owner's compute direction); the numbers here are the frozen memory panel's checks.
- **Nothing about serving changed**: no D11 or token-rate claim is made.

## Cost

| | |
|---|---|
| pod | one 2-GPU pod, six fits at 585–640 s, ≈ **$2.00**, released and deleted at the last fit |
| local | six v5 reply sweeps (no judge), the paired classification |

## Evidence

- Reports and models: `~/uor-r4-local/georead2-20261011/runs/{L1,L2,L3,F1,F2,F3}/`, bundled to cloud-store as `[BUNDLE]`.
- The instrument: [#2220](https://github.com/UOR-Foundation/uor-r4/pull/2220) (`set_read_score` + `read_override=`), which made the comparison possible
  after the first attempt produced six identical `l2` arms (reported with its cost).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

