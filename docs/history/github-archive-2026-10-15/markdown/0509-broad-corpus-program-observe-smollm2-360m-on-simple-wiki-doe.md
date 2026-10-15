# #509 — Broad-corpus program: observe SmolLM2-360M on Simple-Wiki — does a competent-broad teacher lift broad-text substrate accuracy? (the P3 decider)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T21:20:12Z
- **Updated:** 2026-08-08T22:45:36Z
- **Closed:** 2026-08-08T22:45:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/509
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context and direction

The repo's evidence keeps converging on one binding constraint: **the teacher**. Every assignment-side lever is within fractional-pp of its ceiling (#243/#310/#318), the compiler/cover/consistency levers came back negative (#456/#457/#458/#459/#460), and the serving path is healthy (#490/#502). What bounds absolute accuracy on **broad** text is teacher competence: stories15M is near-degenerate off its home distribution (6.4% next==argmax on wiki vs 70% at home) and its teacher-evidence store scores **0.1%** on held-out simple-wiki (below the ~4.5% unigram floor); swapping to *corpus-observed* evidence recovered 4.0% (#394). #320's P2 then measured SmolLM2-360M as a **competent-broad** teacher (floor 3.79 bits).

So this is the decider the project has been building toward: **does observing the 360M teacher on broad (wiki) text produce a substrate that predicts broad text meaningfully better than the stories15M-on-wiki baseline?** If yes, the broad-corpus direction — and the #320-P3 baseline re-pin folded into it — is warranted. If no, the limit is more fundamental than teacher quality.

## Scope (bounded and feasible on the measurement box)

Observe the pinned SmolLM2-360M teacher on `.uor-models/corpora/simple-wiki-20231101` via `r4 observe-text`, at a bounded record count (rehearsal scale, resumable), then `graph-compile`/cover/score and certify. Measure, on held-out **broad** text:

- teacher floor (bits/token) and the artifact's top-1 / bits / teacher-argmax agreement,
- against the stories15M-on-wiki reference (store 0.1%, unigram floor ~4.5%) and the 360M-on-*instruction* P2 rows (#320).

This is **not** a full-scale corpus (teacher generation is ~6.6 tok/s here; millions of positions are out of reach) — it is the bounded rehearsal that decides whether the direction pays before anyone commits hours of compute to scale.

## Pre-registered read

- **POSITIVE** (competent-broad teacher lifts broad-text substrate accuracy clearly above the stories15M-on-wiki 0.1% and the unigram floor) → commit to the broad-corpus program; the #320-P3 re-pin to a SmolLM2 baseline is warranted and folds in here.
- **NEGATIVE / marginal** → the broad-text limit is corpus-scale or mechanism, not teacher quality; record it and keep stories15M pinned.

## DoD

Measured broad-text rows + teacher floor posted here; the POSITIVE/NEGATIVE decision recorded with the P3 implication; `docs/` updated (a broad-corpus section alongside the #320 baseline doc). Provenance κ recorded per the #320 pattern.


## Comments (1)

### comment 5228557593 — 2026-08-08T22:45:35Z — Casey-allard

## Result — POSITIVE. A competent-broad teacher lifts the broad-text substrate off the floor.

One end-to-end broad-text pass, all native (no GPU; AVX2/FMA CPU matmul in the teacher loader): observe **SmolLM2-360M** on **Simple-Wiki** (3,000 articles → 21,235 observation records) → recorded compile (`--vocab-size 49152`) → cover (46 regions, 3 depths) → score `relative_tla` → Gate C on **4,358 held-out D3 positions** (EXCT-miss 62.5%, so Gate C is measuring generalization, not memorization).

### Gate C — held-out top-1 / bits-per-token

| scorer | top-1 | bits/token | note |
|---|---|---|---|
| legacy teacher-hash sum | 0.07% | 72.46 | the off-distribution baseline this program had to beat |
| Rule 1 (graph chain) | 0.32% | 13.51 | exact-context only |
| **Rule 1+2 (D4 precedence)** | **10.19%** ±0.46 | **12.72** | the causal, quotable generation number |
| Rule 1+2, cloud-size-normalized | 13.47% | 12.08 | best plain full-population variant |
| TLA3 store baseline | 25.70% | 14.70 | same-corpus table-native reference |
| **Rule 1+2 + LATENT-MIX (#446 M2)** | **29.0%** | **11.71** | left-key-only at serving → causally legitimate |
| Rule 1+2 + latent ORACLE-RIGHT | 15.9% | 12.39 | upper bound only, **not** causal |
| Rule 1+2 + latent SHUF-CLASS | 21.5% | 12.12 | null control the exit rule must beat |

Cover held-out routing recall: depth-1 reference top-1 **85.5%** / top-M 95.2%. Latent headroom: 10.2% → 29.0% (latent-mix) → 15.9% (oracle-right) = **331.2%** of available top-1 headroom; the #446 exit rule (≥ 2.0pp over baseline AND beats shuffled-class) is **MET (positive)**.

Teacher floor on this Simple-Wiki partition: **3.6015 bits/token** (`evaluate-report`, 21,235 replayed positions, story-contiguous). The evaluate-report table-native artifact itself: 17.3% top-1 / 25.4% agreement at 14.6954 WB bits = **+11.09 over floor** — the same compiler-bound signature P2 (#320) recorded.

### Why this answers the question

The narrow teacher is near-degenerate off its home distribution — the stories15M argmax is **6.4%** next==argmax on wiki10k vs **70.2%** on its home corpus, and the store carrying that evidence served at **~0.1%** off-distribution (below the unigram null; `RESEARCH.md`). Holding the corpus fixed and swapping **only** the teacher moves broad-text held-out top-1 to **10.2% (Rule 1+2) / 29.0% (latent-mix)**, both causal — a **~100–290×** lift that clears the unigram null and the shuffled-class control. Teacher breadth was the binding constraint on absolute accuracy, exactly as #320 predicted.

### Honesty bounds (so the number is not over-read)

- Not a byte-identical A/B against a stories15M-on-Simple-Wiki run; the narrow-teacher figure is the recorded off-distribution serving number. Attribution rests on the teacher being the only changed input **plus** the measured narrow-teacher degeneracy above.
- 10.2% / 29.0% are absolute broad-text accuracies, still ~8–9 bits above the 3.60-bit teacher floor. The substrate is compiler-/data-bound, **not** saturated — this licenses the broad-corpus program, it does not claim substrate↔teacher parity.
- The 29.0% latent-mix row reads the **left key only** at serving (#446 M2) → quotable as a generation number. ORACLE-RIGHT (15.9%) is an upper bound and is not.

### Decision & follow-through

**P3 verdict: POSITIVE** — the broad-corpus direction is warranted. The #320 baseline re-pin remains a maintainer decision per AGENTS.md, now backed by a measured broad-text lift rather than only a ceiling argument.

Writeup (full run, Gate C table, honesty bounds, κ provenance) in **#512** → `docs/smollm2_teacher_baseline_320.md` (new P3 section) and `docs/RESEARCH.md` (#320 / #509 / #273 ledger rows).

Provenance (blake3, 24): teacher `model.safetensors` `eb23c3e8527110b83c091f86` · `tless_artifacts.bin` `9710cd9959c2e9e8d5bd6df8` · `corpus.records` `41fa6286a869d41202197482` · `graph/score.r4g1` `7dab1537ab1e34e76f50505b`. Compiled bundle regenerable from the commands in the doc.

Closing as decided (POSITIVE).



---
_Comment/review ids in this file: 5228557593_

