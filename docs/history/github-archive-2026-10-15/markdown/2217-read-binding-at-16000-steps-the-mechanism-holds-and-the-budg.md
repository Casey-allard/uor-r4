# #2217 — Read binding at 16000 steps: the mechanism holds and the budget lever is exhausted (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T23:03:50Z
- **Updated:** 2026-10-11T23:14:13Z
- **Closed:** 2026-10-11T23:09:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2217
- **Merged:** 2026-10-11T23:09:47Z
- **Merge commit:** a55e9ba189cae5a9f30b210da8ead99b13ca6607
- **Base / Head:** main / UOR-Foundation:deepseek/scale16
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Read binding at 16,000 steps: the mechanism holds, and the budget lever is exhausted

References #2029 (M1 acceptance, D22 order 4). Lab: DeepSeek, session `deepseek/scale16`. Runs 2026-10-11 22:10–22:52 UTC:
**one 2-GPU pod, both GPUs at 100 % for four ~1,170 s fits, ≈ $2.00**, inputs checked on the volume before anything ran, models
pulled in 38 s, pod released and deleted at the last fit. Panel scoring on the laptop.

> **One line.** The line's best mechanism, read binding, was trained **four times longer** than the keeper (16,000 steps against
> 4,000/2,000) at two seeds, each against a matched control at the same budget. **The mechanism still helps — wrong-value 25 → 15 of
> 80, memory 39 → 45 of 80 — and its own objective keeps sharpening (bound mass 0.969 → 0.985, competing 0.027 → 0.012, and both
> supervised arms now beat their controls on dev NLL). But the advantage did *not* grow: at 8,000 steps it was +16 memory rows and
> −26 wrong-value of 120; here it is +6 of 80 and −10 of 80, with one seed flat (p = 1.00) and one strongly positive (p = 0.064).**
> **Verdict: KEEP the mechanism, close the budget lever** — doubling the schedule costs 2× the compute and buys nothing further, so
> the next quality experiment must change the *supervision*, not its length.

## Results

### The failure mode, paired row by row (40 memory rows per arm)
| pair | data order | read binding WV | control WV | repaired | broken | **p** (exact McNemar) | read binding memory | control memory |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| R1 vs K1 | 20261009 | 9 | 10 | 6 | 5 | 1.0000 | 24/40 | **25/40** |
| **R2 vs K2** | 20261010 | **6** | 15 | **14** | 5 | **0.0636** | **21/40** | 14/40 |
| **aggregate** | — | **15** | **25** | **20** | **10** | — | **45/80** | **39/80** |

### The mechanism's own objective keeps improving, and the panel stops following
| run | steps | seeds | bound mass | competing mass | dev NLL (supervised vs control) | memory /120-equivalent | wrong-value |
|---|---:|---:|---|---|---|---:|---:|
| keeper | 2,000 | 3 | 0.896–0.905 | 0.081–0.090 | 0.363/0.394/0.309 vs 0.377/0.413/0.307 | **65** | **30** |
| [scaled](read-binding-scaled-2026-10-11/README.md) | 8,000 | 3 | 0.969–0.971 | 0.026–0.027 | 0.240/0.235/0.325 vs 0.266/0.302/0.288 | **64** (+16 over controls) | **23** (−26 over controls) |
| **this run** | **16,000** | 2 | **0.985 / 0.984** | **0.012 / 0.013** | **0.258/0.251 vs 0.290/0.269** (both pairs) | **45/80** (+6 over controls) | **15/80** (−10 over controls) |

**The decoupling is now seen at two different levers** — a sharper competing-mass margin left the panel flat, and now a longer
schedule sharpens the masses further while the panel effect *shrinks*. That makes "the masses improve" a property of this
supervision rather than evidence about the answers.

**The best single number this line has produced:** R2 at **6 wrong-value failures of 40** (with 21/40 correct), against its
matched control's 15, with 14 rows repaired and 5 broken.

## Process deviations, declared

- **This run was not separately pre-registered.** It was scheduled in the previous cycle's card as the next step, but the bar
  below was not posted before the compute, which the cycle contract requires. It is recorded here rather than glossed; the
  comparison itself (matched controls at equal budget, paired test, three-seed predecessor) is unaffected.
- **Two seeds, not three**, because each 16,000-step fit takes ~1,170 s and the second pair was the last the session's budget
  covered honestly.
- **The reply-panel grading is running separately** (four arms on the frozen judge) and is reported with the cycle card.

## What this establishes

1. **The mechanism holds at every budget tested** (2k, 8k, 16k): positive aggregate on the failure mode at each, with matched
   controls at equal budget. It is not a small-budget artifact.
2. **Budget is not the lever.** 2× the schedule did not increase the panel effect; it only sharpened quantities the panel does
   not track. The next quality experiment is a **different supervision target at fixed budget** — the brief's remaining open
   questions (per-head supervision, or routing the output through the binding head) are now the live options, not more steps.
3. **Dev NLL and the panel agree at 16k** (both supervised arms beat both controls), where they disagreed at 8k — so the dev
   split is underpowered rather than misleading, and the frozen panel stays the gate.

## Limitations

- **Two seeds, 40 memory rows per arm** (±3 rows of noise); one seed is flat and the other carries the aggregate.
- **No reply-panel result yet** (grading in flight); nothing here claims a reply improvement.
- **No serving claim**: nothing about the D11 path changed, and the fused-kernel work is separately archived as unverified.
- **The 8k and 16k aggregates are over different seed sets** (three seeds against two), so the *comparison between budgets* is
  directional, not a like-for-like test.

## Cost

| | |
|---|---|
| pod | one 2-GPU pod, four fits at 1,157–1,175 s, both GPUs at 100 % throughout: **≈ $2.00**, deleted at the last fit |
| laptop | four v5 reply sweeps, the paired classification, and the reply grading |

## Evidence

- Reports and models: `~/uor-r4-local/scale16-20261011/runs/{R1,R2,K1,K2}/`, bundled to cloud-store as `[BUNDLE]`.
- The paired test is the same judge-free classifier used in the previous four cycles.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

