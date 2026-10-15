# #446 — Two-sided context: distribution classes over ⟨left,right⟩ (Kracht p.24/p.438) — measured +14.9pp over left-only

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T01:26:27Z
- **Updated:** 2026-08-06T17:04:37Z
- **Closed:** 2026-08-06T17:04:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/446
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First measurement in PR #444: two-sided keying reaches 38.77% held-out top-1 vs 23.86% left-only and 31.72% for the shipped forward-anchor fusion; the foreign-right-key falsifier collapses to 6.70%, so the gain is right-context information, not key-space inflation.

**Theory (Kracht, *The Mathematics of Language*):** a context is a pair ⟨left,right⟩ and the closed sets under the Galois connection between strings and contexts are the distribution classes (p.24, Sestier operator). Categories may not distinguish objects finer than context sets (p.438). Our forward-anchor channel was an accidental one-token instance of this.

**Program:**
- M1: two-sided keying inside the score pipeline (not just the certifier harness), graded by the pipeline's own eval.
- M2: serving disposition. Right context is NOT causally available in left-to-right generation. Viable regimes: infill/A-mode (validated), two-pass, or using two-sided classes at CONSTRUCTION time only — i.e. induce the cover/store partition from two-sided classes, then serve left-only. That third option is the interesting one: it imports the structure without needing the future at inference.
- M3: interaction with #435 capacity scaling — two-sided classes grow ~n^0.80 vs ~n^0.44 for left-only, so the two-sided key space needs a data-proportional budget.

**Guard rails from the same source:** distributional data is licensed as a ceiling and falsification test on categories, not a generator (Def. 5.67); the definition is circular without an existing grammar (p.446). And Theorem 3.11 (weak compositionality ⟺ recursive enumerability) means 'structure will emerge from a good enough next-token model' is formally vacuous — do not reason that way.

## Comments (3)

### comment 5201046702 — 2026-08-06T06:12:28Z — Casey-allard

**M1/M2/M3 complete. Direction resolved.**

M1 (PR #447, not causal): two-sided keying in-pipeline 31.0% / 6.9411 bits vs Rule 1+2 26.4% / 8.2674; falsifier collapses to 8.0%. Confirmed the dilution mechanism — 98.9% of held-out resolves at full graded code, and the right key subdivides 11,221 left cells into 192,381 (17.1×), gaining +4.6pp / −1.32 bits on exactly that population.

M2 (PR #454, causal): marginalizing the right class gives 25.91% / 7.3707 — top-1 NEGATIVE (−0.51pp) but bits −0.897 vs baseline and better than the TLA3 store baseline.

M3 (PR #455, causal): hard-select 22.81%, top-k=3 24.54% — monotone the wrong way. Class predictability 19.55% vs 1.95% chance, but 4.25 bits of posterior entropy over ~51 classes. **The latent right-class family is measured dead for top-1**; the oracle's +5.4pp is unreachable from the left key.

**What survives:** the calibration gain (−0.897 bits, causally legitimate, top-1 at parity) and the diagnosis that fixed code-space dilution — not cover capacity — is the binding constraint at scale. The live direction from here is code-space capacity, not latent classes.

### comment 5207605964 — 2026-08-06T16:54:16Z — Casey-allard

Capacity cross-check for this issue's closing direction ('the live direction from here is code-space capacity, not latent classes') — measured on #460's capacity-scaled 2.11M run:

Raising the codebook sample 21x (0.59% -> 12.4% of the split, gate PASS) moved Rule 1+2 from 26.1% to **26.5%** and made the ExactContext share go UP (97.8% -> 98.8%), with records-per-full-key still 36 against a limit of 32. **Code-space capacity is confirmed as the constraint, but it is not reachable by fitting the codebook better — the 4-stage x 256 code itself cannot subdivide 2.11M records.**

That is the same wall M1 hit from the other side: two-sided keying subdivided 11,221 left cells into 192,381 (17.1x) and gained +4.6pp on exactly that population. M1's gain was subdivision, and subdivision is precisely what this run could not buy legitimately.

Also measured on the same run, for the record — the causal arms reproduce their negatives at scale (two-sided live 1.0% vs Rule 1+2 26.8%; latent-mix 1.0% but bits 15.4778 vs 22.2078, i.e. the calibration gain survives and is now very large). Consistent with M2/M3: the family is dead for top-1, alive for bits.

Next step for both issues is the same and it is a code change gated on measurement: **raise STAGES or K and re-measure records_per_full_key and the headline.**

### comment 5207672689 — 2026-08-06T17:04:36Z — Casey-allard

**Closing with the full record; the live thread moves to #460 (one stream, no split tracker).**

M1 (PR #447, not causal): two-sided keying 31.0% / 6.9411 bits vs Rule 1+2 26.4% / 8.2674; foreign-key falsifier collapses to 8.0%. Mechanism: 98.9% of held-out resolves at full graded code, and the right key subdivides 11,221 left cells into 192,381 (**17.1x**), gaining +4.6pp on exactly that population.
M2 (PR #454, causal): marginalizing the right class gives 25.91% / 7.3707 — top-1 NEGATIVE (−0.51pp), bits −0.897 vs baseline, better than the TLA3 store baseline.
M3 (PR #455, causal): hard-select 22.81%, top-k=3 24.54% — monotone the wrong way; class predictability 19.55% vs 1.95% chance but 4.25 bits of posterior entropy over ~51 classes. **The latent right-class family is measured dead for top-1**; the oracle's +5.4pp is unreachable from the left key.
At 2.11M scale (#460 capacity-scaled run) the causal arms reproduce their negatives (two-sided live 1.0% vs Rule 1+2 26.8%) while the **latent-mix calibration gain grows large and survives: bits 15.4778 vs 22.2078.**

**What survives and where it goes:** (a) the calibration gain — causally legitimate, top-1 at parity, now large at scale; (b) the diagnosis that fixed code-space dilution, not cover capacity, is the binding constraint. Both are carried by **#460**, whose capacity-scaled run confirmed the same wall from the other side: a 21x better-fitted codebook moved Rule 1+2 only 26.1% -> 26.5% with records-per-full-key still 36 (limit 32). M1's +4.6pp WAS subdivision, and subdivision is exactly what capacity config cannot buy — the next lever is STAGES/K.


---
_Comment/review ids in this file: 5201046702, 5207605964, 5207672689_

