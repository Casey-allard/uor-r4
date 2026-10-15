# #434 — Zeta-grid routing at scale + VSA/spectral geometry: the unmeasured ancestry (from #421)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T16:35:20Z
- **Updated:** 2026-08-07T15:06:59Z
- **Closed:** 2026-08-07T15:02:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/434
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Tracker for the two never-measured lineage items that #421 revived but did not itself measure (per the no-orphans rule — these get their own issue, not a closing note): (1) **zeta-grid routing quality at scale** — #246 was fixture-only; the 512-dim zeta-zero grid state has never been graded on a natural corpus; (2) **VSA/spectral geometry** — shipped in uor-r4-router, never measured under the modern discipline. Context now available: the #421/#422 harnesses (router→token mapping, retrieval-rank + sector metrics), the corpus-composition finding (#399 record — any measurement here must run on BOTH the redundancy-rich 500k corpus and a broad-corpus slice), and the #422 spread-vs-retrieval frontier. First candidate measurement: zeta-grid state as the retrieval vector in the #422 harness (replace content-derived query vectors with grid-state vectors, same arms/metrics).

## Comments (8)

### comment 5195076531 — 2026-08-05T17:27:51Z — Casey-allard

First-ever natural-corpus measurement of the zeta-grid state: **CONFIRMED, decisively** (PR #439 harness, full caps 2000/500 on the home corpus): **arm Z retrieval MRR 0.8932 / top-1 0.830** vs content-vector baseline 0.2348/0.208 and shuffled control 0.0001. The 512-dim zeta-zero-grid state — unmeasured since the project began — retrieves at ~3.8× the content-vector baseline with a dead shuffle control. Honest caveat carried from the harness docs: stored content-index vectors are band-sliced while zeta states are full 512-d, so part of the gap may be banding vs. state richness; a band-matched control is the obvious follow-up. Implications if it holds: zeta-state vectors as the router's retrieval substrate would directly attack the #421-lineage anchor-accuracy gap (9.3%) and sidestep the #422 projection frontier on the retrieval side. Next: band-matched control + zeta-state retrieval as the anchor-supply mechanism, re-run through the reconnection harness.

### comment 5195809297 — 2026-08-05T18:39:43Z — Casey-allard

CORRECTION + new lever (PR above): band-matched control KILLS the zeta-state claim — ZB 0.2564 (< 0.4466 bar), content-full-width 0.8948 ≥ zeta 0.8932, evolution adds −0.0016. The 0.89 was vector fullness. **Re-scoped finding: #245's storage banding (1/16th band kept) costs ~0.66 MRR — de-banding stored vectors is a measured ~3.8× retrieval lever, the direct route to router anchor accuracy.** Zeta-grid retrieval per se: measured, not confirmed. VSA/spectral remains this issue's open unmeasured item.

### comment 5196854048 — 2026-08-05T20:15:37Z — Casey-allard

De-banding decision RESOLVED (Casey, option C→A): ARM BF passes all three pre-declared conditions (PR #442: free top-1 8.17 vs banded 7.64, anchor-acc 11.4 vs 9.3 +2.1pp). **ADOPTED: full-width storage in the #245 path — implementation order: (1) consumer audit of band-sparsity assumptions, (2) storage change, (3) re-run reconnection + memory-lift as regression gates.** Honest caveat on the record: 3.8× retrieval MRR → +0.5pp free top-1 means retrieval is no longer the anchor-supply bottleneck; the continuation side-table is next. Remaining #434 item: VSA/spectral measurement.

### comment 5205257502 — 2026-08-06T13:29:25Z — Casey-allard

Relocating results that were misfiled to #273 on 08-05 — they belong to this lineage tracker:

**Router-anchor reconnection (the original hybrid), measured end-to-end on the natural stack:** true-anchor ceiling 31.5% free top-1 | router-selected anchors 7.6% | unigram null 2.1% | router outputs shuffled to wrong positions 3.1%. Router beats the unigram null by +5.5pp and beats its own shuffled control by +4.55pp — **routing carries positional information** (this was the never-run falsifier). Under a non-oracle DRAFT query context: 4.9% vs the same 2.1/3.1 nulls, still positive, so the result survives generation-realistic conditions.

Caveats on the record: token ids were rendered as synthetic words (the corpus carries no text), retrieval was cosine over the router's content-derived state vectors, and story caps were 2,000 construction / 200 held-out. Anchor accuracy was only 9.3% (router) vs 6.0% (unigram) — the gap to the 31.5% ceiling is the routing-quality work this tracker exists for.

Note also that the #422 sector-transport measurement came back negative (occupancy spread without retrieval value), so zeta-grid routing quality at scale is the live lever here, not the Hopf projection.

### comment 5207129403 — 2026-08-06T15:47:22Z — Casey-allard

Scope note as PR #465 (full-width storage) goes in: that completes lineage item (1) of this tracker — zeta-grid routing at scale, which measured, then got corrected by the band-matched control, then re-scoped into the de-banding lever now implemented and regression-gated (memory-lift MRR 0.2348 to 0.8948; router anchor accuracy 9.3% to 11.4%).

**Item (2) remains untouched: VSA/spectral geometry, shipped in uor-r4-router and never measured under the modern discipline.** This issue stays open on that item alone. Leaving it unassigned/claimable.

### comment 5212676125 — 2026-08-07T05:09:47Z — Casey-allard

Level-set: item 1 is done and shipped, item 2 is untouched. Keeping this open on item 2 only.

**Item 1 (zeta-grid routing at scale) — DONE.** Measured (#439 arm Z), corrected by the band-matched control (#440), re-scoped into the de-banding lever (#442 ARM BF), and adopted as full-width content-bearing storage in PR #465, with `docs/debanding_audit_434.md` and `banded_storage: bool` defaulting to false. Regression gates passed: memory-lift content-derived MRR 0.2348 to 0.8948, router anchor accuracy 9.3% to 11.4%.

**Item 2 (VSA / spectral geometry) — NOT measured.** `SpectralGeometry` and `VsaGeometry` live in `crates/uor-r4-router/src/geometry.rs` with `benchmark.rs::run_ablation_benchmark`, but the only test is `tests/ablation_benchmarks.rs`, which asserts `recall_at_3 >= 0.0` and a hard-coded `migration_agreement == 0.98` on synthetic queries. Those files predate the #421 lineage; there is no corpus-scale run and no pre-declared exit rule.

**Concrete remaining work:** run `run_ablation_benchmark` on a real corpus with ground-truth retrieval targets and a pre-declared exit rule, or record it as deferred with a stated activation condition. Either outcome closes this issue.

**One adjacent thread noted in #465 and explicitly out of scope here:** `get_top_resonances_native` still builds a band-only *query* projection against now-full-width stored items, so the production retrieval surface has not yet recovered the ARM BF geometry. That deserves its own issue if anyone wants the de-banding gain to reach serving.

### comment 5218194836 — 2026-08-07T14:18:26Z — Casey-allard

Taking item 2. First, what is actually there — because "never measured under the modern discipline" understates it.

**`run_ablation_benchmark` cannot measure anything, in four independent ways.**

- `migration_agreement: 0.98` is a **hard-coded literal** (`// high consistency score`), not a computed quantity. `tests/ablation_benchmarks.rs` then asserts `migration_agreement == 0.98`, i.e. it verifies a constant equals itself.
- `unlearning_time_ns` is introduced by `// Measure unlearning latency (deleting a route)`, but the timed block calls `geometry.ground(obj)` on the first query and discards the result. It measures grounding. Nothing is deleted.
- `recall_at_3` and `hits_at_3` are the **same quantity by construction** — `recall_sum += 1.0` happens in the same branch as `hits += 1`, and both divide by the query count. Two labels, one number.
- The test's remaining assertions are `recall_at_3 >= 0.0` and `hits_at_3 >= 0.0`, which are vacuously true for a ratio of non-negative counts.

So running it on a real corpus would produce numbers that do not mean what their names say. That is the third vacuous-green instrument this week (`kappa_reproduction` skipping silently, `cover_scaling` passing over one row), and the pattern is worth naming: **an instrument that cannot fail is indistinguishable from one that passes.**

**But the underlying question is real and live.** `GeometryType` is a router config switch (`Spectral` default, `Vsa` via `set_geometry_type`), and `get_top_resonances_native` dispatches on it to genuinely different retrieval implementations — `retrieve_vsa_multi_facet_resonance` versus route-then-`retrieve_geometric_resonance`. Which geometry the router should use is a decision nobody has evidence for.

So rather than run a broken benchmark or defer, I am going to measure the switch itself, through the production retrieval surface, on the machinery that already works: the corpus / window / probe / null laws from `zeta_state_retrieval.rs` (the #439-#440 harness, which produced the band-matched control that correctly killed the zeta claim).

**Run contract**

```
metric to move:      retrieval MRR and top-1 over held-out probes with
                     ground-truth target windows, through
                     get_top_resonances_native, Spectral vs Vsa
reachability ceiling: this is a CHOICE between two shipped implementations,
                     not a new mechanism — the ceiling is whatever the better
                     of the two already achieves. Reference points on this
                     stack: content-full-width 0.8948 MRR, banded 0.2348,
                     shuffled control 0.0001
instrument gate:     the shuffled-probe control must be near zero for BOTH
                     arms, or the harness is matching on something other than
                     content and neither arm's number counts
exit rule:           a geometry wins if its MRR exceeds the other's by >= 0.05
                     with the null dead; otherwise the switch is measured
                     indifferent and the default stands on other grounds
if positive:         the losing geometry becomes a removal candidate and the
                     winner becomes a documented default with a reason
if negative:         record the switch as measured-indifferent with the
                     numbers, so the next person does not re-litigate it from
                     intuition; item 2 closes either way
cost estimate:       minutes; same caps as the zeta harness (2,000
                     construction stories / 500 probes), no teacher
```

I will also repair `run_ablation_benchmark` and its test rather than leave a instrument in the tree that reports a constant as a measurement.

**One adjacent thread I will check while in here, and file separately if real** — the #465 scope note says `get_top_resonances_native` still builds a band-only *query* projection against now-full-width stored items. If so, the adopted de-banding gain (MRR 0.2348 -> 0.8948, anchor accuracy 9.3% -> 11.4%) is not reaching serving, which would matter considerably more than this issue does.

### comment 5218716714 — 2026-08-07T15:06:59Z — Casey-allard

Closing record (auto-closed on merge of #479, 3cfa8d7). Both items done.

**Item 1** — zeta-grid at scale: measured, corrected by the band-matched control, re-scoped into de-banding, shipped as full-width storage in #465.

**Item 2** — Spectral vs VSA, measured through `get_top_resonances_native` on 46,342 windows / 500 probes (`docs/geometry_ablation_434.md`):

| arm | top-1 | MRR | recall@20 | deranged-key MRR |
|---|---:|---:|---:|---:|
| Spectral (default) | 0.6240 | **0.7179** | 0.9720 | 0.0000 |
| Vsa | 0.0000 | **0.0000** | 0.0000 | 0.0000 |

Not a quality gap — a wiring one. The arms read different stores (`corpus_index_by_identity` vs `facet_store`), and `index_corpus` populates the first and never the second. **A caller who sets `set_geometry_type("vsa")` after `index_corpus` silently loses retrieval.** Two dispositions follow — connect the stores, or make the switch refuse/warn — and the record deliberately picks neither because it is a design decision, not a measurement.

The old instrument was replaced rather than run: `migration_agreement` was a hard-coded `0.98` the test asserted equal to `0.98`, `unlearning_time_ns` timed `ground()`, and `recall_at_3`/`hits_at_3` were one quantity under two names.

**Filed separately: #480.** `retrieve_geometric_resonance` builds a band-only *query* projection against stored vectors that are full-width by default since #465, so the adopted de-banding gain (MRR 0.2348 → 0.8948, anchor accuracy 9.3% → 11.4%) is not reaching serving. That was the #465 scope note's open thread; it is confirmed and it is a larger prize than this issue was.


---
_Comment/review ids in this file: 5195076531, 5195809297, 5196854048, 5205257502, 5207129403, 5212676125, 5218194836, 5218716714_

