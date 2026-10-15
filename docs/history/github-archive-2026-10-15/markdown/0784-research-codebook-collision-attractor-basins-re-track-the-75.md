# #784 — research: codebook-collision attractor basins — re-track the #759→#460 root cause (distinct prompts quantize to identical context codes)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:28Z
- **Updated:** 2026-08-18T18:45:16Z
- **Closed:** 2026-08-18T18:45:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/784
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

#759 closed (2026-08-17) with a corrected diagnosis: the measured prompt-collision collapse is **greedy argmax over quantization-collided context codes at real resolution depth** — an attractor-basin effect — not the fallback tier originally scoped. Its closure pointed follow-up root-cause work at #460, but **#460 is closed**, so the root cause currently has no open tracking issue. This issue re-tracks it.

## Evidence on record

- #758: 15-prompt exact-match protocol — `smollm2-135m-instruct` (post-#755 retest bundle) 9/15 distinct; `smollm2-360m-broad` 11/15 distinct; largest identical group 5 prompts.
- #762: opt-in `--sample <SEED>` weighted sampling takes 13/15 → 15/15 distinct — a mitigation, not a fix; it explicitly deferred the context-window/collision-resolution lever here (then-#460). Note the sampler is currently unreachable on the R4G1-preferred ask path (see the decode-path issue).
- 2026-08-18 baseline audit (live, at `aea30bae`): two different prompts → **byte-identical** output on both `smollm2-135m-instruct` and `smollm2-360m-instruct` through the pure R4G1 path (pre-#755 bundles; the refresh issue re-baselines this).
- Gate C live run same day: EXCT resolves 85.1% of held-out; the **EXCT-miss generalization slice is 1.81% top-1 / 16.89 bits (n=14,943)** — off exact context, the graph path is near the historical unigram-floor family (#456/#457).
- #460's own records: resolution levers never paid; fit levers paid small; scaled cover capacity (48→110 regions) lifted region-path top-1 +5.0pp but serving impact capped ~0.15pp at that coverage.

## Scope (measurement-first; no lever ships without a pre-registered contract)

1. **Collision-rate instrument (cheap, minutes):** on refreshed bundles (dependency: #783), measure how often distinct prompts quantize to identical compiled context codes — the 15-prompt set plus a held-out prompt sample; report collision rate by trailing-window length and code configuration.
2. **Pre-registered lever experiments** (each with its own contract posted here before launching, per AGENTS long-run discipline): scaled cover capacity (#460 lever 1), codebook-fit lever, context-window/collision resolution (#762's deferred lever), and signature-lane differentiation at the terminal fallback (#759's historical scope — superseded as the diagnosis, still bounds worst-case behavior).
3. Each experiment names its null, falsifier, and what each outcome causes; identical next-actions on both branches = drop the run.

## DoD

Collision-rate numbers on record; a ranked lever list with pre-registered contracts; at most one lever adopted or all recorded negative — either outcome closes this with a record.

**Next action:** blocked on #783 (increment A1); then run the collision-rate instrument on the refreshed 135m bundle.


## Comments (2)

### comment 5332557314 — 2026-08-18T18:45:13Z — Casey-allard

## Collision-rate instrument: run 2026-08-18 on the refreshed canonical bundle — the collision hypothesis is REFUTED at full depth

**Instrument** (scratch measurement tool, no repo changes; mirrors the plain ask path's exact quantization: `encode → trailing window → bundle_window_plain → assign_code_for_bundle`): 15-prompt protocol set + 17 held-out natural sentences (wiki_corpus.txt lines ≥ 40 chars, deterministic prefix; the ≥40-char filter exhausted at 17 — n=32 total, recorded honestly). Positive control: identical window ⇒ identical code (asserted). Negative control: disjoint synthetic windows separate at full depth (passed — the instrument can fail and didn't).

**Collision rates by trailing-window length k and code depth** (share of prompts whose code prefix is shared with ≥1 other prompt):

| depth | k=2 | k=4 | k=6 | k=8 |
|---|---|---|---|---|
| 1 | 38.7% | 41.9% | 33.3% | 23.8% |
| 2 | 6.5% | 6.5% | 8.3% | 0.0% |
| 3 | 6.5% | 6.5% | 8.3% | 0.0% |
| 4 (full) | **0.0%** | **0.0%** | **0.0%** | **0.0%** |

**Store-resolution depths** (full window): histogram [d0:0, d1:1, d2:8, d3:7, d4:16] — half the prompts resolve at full depth, and **not one full-depth code is shared between distinct prompts**.

**The actual mechanism, measured:** despite fully distinct codes, **11/15 protocol prompts argmax to the same token 198 ("\n")** from their *different* store rows (counts 68–2,914); the outliers are exactly the distinct-output groups #758 observed ("What is April?" → " are", "hello" → "ired", "1 2 3 4 5" → "0" — the digit attractor's seed, "Once upon a time…" → " more"). So the prompt collapse is **argmax-of-converging-marginals** — distinct contexts whose continuation distributions share the corpus-dominant mode — not quantization collision. This sharpens #759's corrected diagnosis one step further: the basins are in the distribution content, not the code space. It is consistent with the #785-C3 witness on the scored-graph path (the "the 19…" digit attractor is row-tier, i.e. real evidence, not collided codes).

## Ranked lever list (per DoD)

1. **Decode policy on real evidence** — windowed/anti-cycle repetition handling + sampled-vs-greedy on the R4G1 path. Directly attacks the confirmed mechanism. Contract already pre-registered as **#785-C4** (PASS iff sampled ≥ 14/15 distinct AND no regression on `repetition_rate_rule12` 0.2219); the C3 witness observation (digit/comma cycles evade the consecutive-only beam penalty) feeds it. **Runs under #785.**
2. **Signature-lane differentiation at the terminal fallback** (#759's historical scope): the graph engine's `signature_affinity_bonus` already exists but is bounded at ≤ 2,880 raw — two orders below typical row-score deltas, and the serving path now prefers the scored graph whose row tier carries the signal. Recorded as **superseded by the scored-graph path**; no run.
3. **Cover capacity / codebook fit (#460 levers) and context-window collision resolution (#762's deferred lever): DE-PRIORITIZED by this measurement** — they remedy collisions, and full-depth collisions measure 0.0%. The depth-1..3 prefix sharing is the hierarchical backoff working as designed, not a defect. No contract; reopen only if a future corpus/codebook change shows nonzero full-depth collision.

## Closing per DoD

Collision-rate numbers on record; ranked lever list with the one live contract pre-registered (#785-C4); no lever adopted here (the top lever's adoption decision is C4's maintainer call). The root-cause question this issue re-tracked from #759→#460 is answered by measurement: **not collision — marginal convergence under argmax**. Instrument source retained at `/tmp/collide784` on the dev machine and reproducible from the description above.


### comment 5332557466 — 2026-08-18T18:45:14Z — Casey-allard

Closed with the measurement record above: full-depth collision rate 0.0% (hypothesis refuted), mechanism = argmax over converging marginals (11/15 prompts share token 198 = "\n" from distinct codes), ranked lever list posted with the one live contract pre-registered as #785-C4. Unassigning.


---
_Comment/review ids in this file: 5332557314, 5332557466_

