# #460 — Cover split criterion and codebook fit: the two capacity levers that remain after STAGES/K measured negative

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-06T13:22:39Z
- **Updated:** 2026-08-07T13:42:05Z
- **Closed:** 2026-08-07T13:33:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/460
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Measured 2026-08-05/06 (falsifier-2 scale campaign).** Running the score pipeline on a 4.6x larger corpus (500k -> 2.11M records, the 10k-article wiki observation set) produced a DEGENERATE configuration, not a better model:

| metric | 500k | 2.11M |
|---|---|---|
| induced regions | 40 (8/14/18 by depth) | **8, depth 1 only** |
| split candidates accepted | 16 of 22 | **0 of 8** |
| mean records / region | 3.1M | 17.9M |
| emission contrast | 0.5012 | 0.0915 |
| probability mass kept | 0.0157 | 0.0003 |
| Rule 1+2 top-1 | 36.5% | 26.1% |
| Gate C validity | measures generalization | **CANNOT measure generalization** (strict miss 2.2%) |

**PRIMARY CAUSE (traced, arithmetic in docs/capacity_scaling_432.md):** the cover's split test is an ABSOLUTE entropy-gain floor — `gain_bits > 0.25` (induction.rs:606, DEFAULT_SPLIT_ENTROPY_GAIN_BITS induction.rs:184) — evaluated on the plug-in MLE entropy with no bias correction (induction.rs:1108-1119). The plug-in estimator inflates a binary split's measured gain by ~0.721(T-1)/S bits: with T~8,000 next-token types that is ~0.231 bits at S~25,000 (essentially the whole 0.25 budget) but ~0.055 bits at S~105,000. **The bar is constant in N; the quantity measured against it falls as 1/S. At 2.11M not one candidate clears it (max gain 0.176 vs floor 0.25); the entropy floor is the only rule that ever rejects.**

**SECOND FINDING:** `RVQ_SAMPLE_CAP = 10_000` (compiler.rs:201, applied :1515) is a hard sampling cap on codebook training, and the caller discards full-set codes (:1782) — **#407's CTX_SAMPLE 6k->50k raise did not change the codebook's training set at all.** 10,000/256 = 39 vectors per centroid; 2.50% of the 500k split, 0.59% of 2.11M. Realized full-code keys grow only 2.26x for 4.27x records (~N^0.55). The 500k baseline everything is calibrated against was already on an under-fit codebook.

Also saturating with N fixed: emission E=64 (mass kept 0.03%), root_top_b/exct_top_x=64, transition_out_degree=8, FWDA_ENTRY_CAP=64 + FWDA_MIN_TOTAL=2 (a noise floor that weakens as data grows), context_entries=64/order=2.

**Consequence for #399:** the at-scale forward-anchor negative (true-anchor 16.2% vs Rule 1+2 26.1%) was measured in this degenerate regime and CANNOT be read as a scale law for the channel. Falsifier 2 is INCONCLUSIVE pending a capacity-scaled re-run.

Shipping: audit doc + a `capacity_scaling` instrumentation test that prints per-structure SATURATION verdicts for any corpus, so no measurement is trusted on a saturated configuration again. Scaling formulas proposed in doc section 5 (bias-corrected or relative split criterion; RVQ/CTX sample = min(N/10, cap); Heaps-exponent caps) — proposals only, each paired with its confirming measurement. No production default changed in this PR.

## Comments (10)

### comment 5207129272 — 2026-08-06T15:47:21Z — Casey-allard

Reopened: PR #461's body said "Closes #460 measurement scope" and GitHub auto-closed the whole issue on merge. Only the AUDIT half shipped (the trace, the instrumentation test, the proposed scaling laws). The substantive half — the capacity-scaled re-run that decides whether the at-scale forward-anchor negative was a capacity artifact — is running now and has nowhere else to land.

Run in flight (all pre-existing env overrides; no code change needed): `R4_RVQ_SAMPLE_CAP=211000` and `R4_CTX_SAMPLE=211000` (was 10k/50k), `R4_COVER_SPLIT_CRITERION=relative`, `R4_COVER_K0=18`, `R4_COVER_REGIONS_BUDGET=512` on the 2.11M corpus. **The codebook is the primary lever, not the cover** — per the #435 record, scaled k0 alone took the cover 14 to 25 regions and left Rule 1+2 at 26.1% identical, because 97.9% of held-out positions resolve as ExactContext and only 2.1% ever touch the graph path. Fixing the cover cannot move a headline the cover barely participates in; the codebook is what makes those ExactContext keys discriminative, and at a 0.59% sample it was making 'exact context' a coarse bucket.

Gate before any number is trusted: the `capacity_scaling` instrumentation must report PASS, not SATURATED, on cover and codebook. Comparison targets: degenerate 2.11M row (Rule 1+2 26.1% / 22.13 bits, true-anchor 16.2%, 8 regions, ExactContext 97.8%) versus the 500k row (36.5%, true-anchor 39.6% / 40.9% live).

### comment 5207604588 — 2026-08-06T16:54:03Z — Casey-allard

**Capacity-scaled re-run complete. The codebook lever worked and the headline did not move — which relocates the constraint.**

Config (env overrides only, no code change): `R4_RVQ_SAMPLE_CAP=211000`, `R4_CTX_SAMPLE=211000`, `R4_COVER_SPLIT_CRITERION=relative`, `R4_COVER_K0=18`, `R4_COVER_REGIONS_BUDGET=512` on the 2.11M corpus.

**Instrumentation gate (the point of shipping it):**

| verdict | degenerate run | capacity-scaled run |
|---|---|---|
| codebook_sample_fraction | 0.00586 SATURATED | **0.12359 PASS** (21x) |
| cover.observations_per_region | SATURATED | **94,850 PASS** |
| regions | 8 (depth-1 only) | **22** |
| code.records_per_full_key | — | **36.02 SATURATED** (limit 32) |
| exct.supported_record_fraction | — | **0.9882 SATURATED** (limit 0.90) |
| cover.entropy_floor_rejects | 1.0000 SATURATED | **1.0000 SATURATED** |

**Gate C, 402,802 held-out:**

| scorer | degenerate | capacity-scaled | 500k reference |
|---|---|---|---|
| Rule 1 (chain) | — | 27.1% / 22.4726 | — |
| Rule 1+2 | 26.1% / 22.1317 | **26.5% / 22.2078** | 36.5% / 8.3534 |
| TLA3 baseline | 26.0% | 26.4% / 33.8872 | 39.1% |
| true-anchor fused | 16.2% | 16.4% | 39.6% |
| ExactContext share | 97.8% | **98.8%** | 85.4% |

**Reading:** raising the codebook sample 21x moved Rule 1+2 by +0.4pp and made the EXCT bucket MORE dominant, not less (97.8% -> 98.8%). Records per full code key is still 36 against a limit of 32. So the binding constraint is not the sample used to fit the codebook — it is the code space itself: **4 stages x 256 cannot partition 2.11M records finely enough**, and no env flip reaches that. Gate C still declares it CANNOT measure generalization here.

Two independent measurements now point at the same wall: this one, and #446 M1, where two-sided keying bought a 17.1x subdivision of the left cells and +4.6pp on exactly that population. Subdivision is what pays; we cannot currently buy it.

**Next hypothesis (a code change, gated on measurement, not an env flip): raise STAGES or K in the graded code and re-measure records_per_full_key + the headline.** Secondary: the relative split criterion still rejects 100% of candidates — either `R4_COVER_RELATIVE_THETA` needs setting or, per the #435 record (H(parent) 6.52 -> 9.65 bits at this scale), fixed-theta relative is itself the wrong criterion and MDL should be tried.

**On falsifier 2 (#399, closed — cited only):** the at-scale forward-anchor inversion REPRODUCES under a non-degenerate cover and a properly fitted codebook (true-anchor 16.4% vs Rule 1+2 26.5%; live 11.5% vs 26.9%; win/loss +15,094/-55,535; predicted-anchor accuracy 0.0%). It was not a cover artifact. It remains conditional on the code-space regime, so the honest status is: the channel is negative in every configuration reachable without a code-space change.

### comment 5210240288 — 2026-08-07T00:15:30Z — Casey-allard

## Run contract — STAGES/K code-space experiment

Posted before launching, per the AGENTS.md long-run discipline (PR #466).

**Metric to move:** Rule 1+2 top-1 on the 2.11M corpus. Current 26.5% (was 26.1% degenerate; 36.5% on the 500k home corpus).

**Reachability arithmetic.** Realized key occupancy from the latest instrument run (`/tmp/capfix_instr.log`, capacity-scaled artifact):

| level | occupied keys | nominal | occupancy |
|---|---|---|---|
| 1 | 225 | 2.56e2 | 0.879 |
| 2 | 3,689 | 6.55e4 | 5.63e-2 |
| 3 | 19,939 | 1.68e7 | 1.19e-3 |
| 4 (full) | 47,403 | 4.29e9 | 1.10e-5 |

Mean construction records per occupied full key: **36.02** (limit 32). Singletons already 41.7% of keys.

The nominal space is 4.29e9 against 1.71M construction records — nominal capacity is **2,500x** the data and has never been the constraint. What binds is *realized* occupancy: each added stage multiplies occupied keys by only 2.38x (19,939 to 47,403), far below the nominal 256x, because the RVQ residual carries less separable structure at each stage. Extrapolating that same realized factor:

- **STAGES=5** (keeps u8 codes): occupied keys ~47,403 x 2.38 = ~112,800, so records/key ~36.02 / 2.38 = **~15.1** — clears the limit of 32.
- **K=512** (needs a wider code type): at most doubles per-stage resolution, so optimistically records/key ~18, similar ballpark at much higher cost.

**Ceiling on headline movement.** This is the number that decides whether to run at all. Rule 1+2 resolves 98.8% ExactContext, so the headline is essentially the exact-context recall. Finer keys help only where a key currently mixes distinct continuations — bounded above by the fraction of records in non-singleton keys (58.3%) and realistically far below it, since much of that mass is genuinely repeated context. **Estimated ceiling: single-digit pp, plausibly +2 to +6pp; the 500k corpus reached 36.5% at records/key ~4.15, which is the regime STAGES=5 approaches but does not reach (~15).** If the observed effect is under +2pp the hypothesis is refuted as the primary lever, not merely unproven.

**Instrument gate (hard).** `capacity_scaling` must report `code.records_per_full_key: PASS` under the new setting. That is the precondition the entire hypothesis rests on: if the code space still saturates, no headline movement is possible and the Gate C run does not launch. Cost ~39s warm (sidecar, PR #470) — but note the sidecar is keyed by artifact κ, so a code-space change invalidates it and the first run pays the full code pass again.

**Exit rule (pre-declared).** Sampled Gate C at `R4_GATE_C_SAMPLE=10000` (SE 0.44pp): STAGES=5 counts as positive if Rule 1+2 top-1 exceeds the 26.5% baseline by **at least 2.0pp** (i.e. >= 28.5%), measured on the same corpus and split, with records/key PASS.

**If positive:** promote STAGES=5 to a proposed default, which requires the full κ re-pin ceremony (below), a re-run of the pinned measurement rows on the new era, and a decision on whether the 500k-era comparisons get re-based.
**If negative:** record code-space subdivision as measured-dead as the primary lever and stop buying subdivision; the remaining hypotheses are corpus composition (#435's stratification, dormant pending #320's broad-corpus teacher) and the two-sided calibration gain that survived #446 on bits rather than top-1. These are genuinely different next actions, so the run has decision value either way.

**Cost estimate.** Compile on 2.11M with 5 stages (one extra RVQ stage over the whole corpus), plus a cold code pass (sidecar invalidated by the κ change), plus ~60s per sampled Gate C arm. Estimated a few hours wall-clock, dominated by compile.

**κ consequence, stated up front.** `STAGES` and `K` are compile-time constants (`compiler.rs:190-191`) and the graded code is κ-pinned. Any change here **moves κ** and requires the #407 ceremony: `TLESS_CANONICAL_DETERMINISTIC=1`, `dump_baseline_kappa`, fixture adoption, `kappa_reproduction` bit-identity, and a re-pin note. This is a cost to budget, not a blocker — but it means the experiment must run on a branch with its own fixtures and must not disturb the pinned era until a decision is made.

**Design note on K.** `K=256` is baked into `[u8; STAGES]` codes in ~100 places across the workspace, so K=512 is a type-width change, not a constant bump. **STAGES=5 keeps u8 and is therefore the cheaper first experiment** — and by the arithmetic above it also buys more realized subdivision per unit of work. K stays unexplored unless STAGES=5 lands positive and more subdivision is still wanted.


### comment 5211519367 — 2026-08-07T02:45:25Z — Casey-allard

## Outcome — STAGES=5, reported line by line against the run contract

**Metric to move.** Rule 1+2 top-1, baseline 26.5% at 2.11M.
Result: **25.6% ± 0.44pp** (n=10,000 sampled decision run, 0.0248 of the split). Rule 1 26.7% ± 0.44; TLA3 baseline 25.4%; forward-anchor arm 16.1% ± 0.37.

**Reachability prediction vs realized.** Predicted ~15.1 records per occupied full-code key, extrapolating the observed 2.38x per-stage growth in occupied keys. Realized growth was **1.68x** (53,901 at level 4 to 90,824 at level 5), giving **18.80** records/key. The extrapolation held its shape and was conservative in the right direction — the prediction method worked even though the hypothesis did not.

**Instrument gate.** `code.records_per_full_key: PASS (18.80 vs limit 32)`. The gate passed, so Gate C was allowed to run, exactly as the contract specified. Subdivision was really purchased: occupied full keys 47,403 to **90,824**, records/key 36.02 to **18.80**, singleton share 41.7% to 53.2%.

**Exit rule.** Pre-declared: positive iff Rule 1+2 top-1 >= 28.5% (26.5% + 2.0pp). Measured **25.6%**, with a 95% interval of roughly 24.7 to 26.5 — the result is **below the baseline**, not merely short of the threshold, and the threshold sits about 6.6 standard errors above the point estimate. **NEGATIVE, unambiguously.**

**Which pre-declared next action applies: the negative branch.** Record code-space subdivision as **measured-dead as the primary lever** and stop buying subdivision.

This is the strongest form that record can take. We did not fail to test the hypothesis — we bought exactly the subdivision the hypothesis asked for, cleared the gate that was supposed to be the precondition, and the headline did not move up. It moved *down* by 0.9pp, and the store baseline fell with it (26.4 to 25.4), which is the signature of thinner per-key evidence rather than sharper context: at 18.80 records/key with 53.2% singletons, keys are finer but each carries less support.

Two structural notes that make the negative more informative:

Exact-context dominance barely responded. `exct.supported_record_fraction` fell only 0.9882 to **0.9708**, and the status split is ExactContext 9,704 / Graph 283 / Novel 13. Gate C still declares it **cannot measure generalization on this distribution**. Halving records-per-key did not meaningfully move what fraction of positions resolve as exact context, so the headline remains a restatement of exact-context recall.

The cover did not participate at all. The 5-stage compile induced **8 regions** with mean probability mass kept 0.0006 — the absolute entropy floor still rejects every split at this scale (the #435 finding). So the subdivision was purely in the code dimension, and the graph path saw 283 of 10,000 positions.

**Cost of the run, against the contract's estimate.** Compile plus a cold 5-stage code pass plus Gate C, about 100 minutes wall-clock. One overrun worth recording: the sampled decision mode did **not** deliver its expected ~60s Gate C, because `R4_GATE_C_SAMPLE` bounds the per-position arm loop but not the full-corpus table construction that precedes it (`derive_right_codes`, `TwoSidedTable::build` walk all 2.11M records regardless). Filed as a follow-up on #467.

**What remains live.** Corpus composition (#435 — the oracle-stratum edge stands as recorded unrecovered signal; dormant pending the broad-corpus teacher on #320) and the #446 calibration gain, which lives in bits rather than top-1 and grew large at scale (latent-mix 15.4778 vs 22.2078 bits). No κ re-pin ceremony is needed: the positive branch did not fire, so the 5-stage artifact (κ `blake3:213dc162…`) stays an experiment on its branch and the pinned era is untouched.


### comment 5212670087 — 2026-08-07T05:08:53Z — Casey-allard

**Rescoping this issue to what is actually live, and unassigning so it can be picked up.**

The title lever is answered and negative. STAGES=5 bought real subdivision (occupied full-code keys 47,403 to 90,824; records per key 36.02 to 18.80; the instrument gate PASSED at limit 32) and Rule 1+2 top-1 came in at **25.6% +/- 0.44pp against a 26.5% baseline** — below baseline, with the 28.5% threshold about 6.6 standard errors away. The store baseline fell with it (26.4 to 25.4), which is the signature of thinner per-key evidence rather than sharper context. Full outcome against the run contract is in the comment above. The merged audit (`docs/capacity_scaling_432.md` §3.2) independently says the same thing: nominal code capacity is ~2,035x the record count, so the code space is not capacity-starved in the counting sense — the **fit** is.

**Two concrete levers remain, both already implemented and both dark:**

1. **Cover split criterion.** The absolute entropy-gain floor (`gain_bits > 0.25`, `induction.rs`) is evaluated on plug-in MLE entropy whose bias term falls as 1/S, so the bar is constant in N while the measured quantity shrinks — at 2.11M records zero split candidates clear it, and every run since has induced 8-22 regions with mass-kept ~0.0006. PR #445 already shipped `SplitCriterion::{RelativeGain,Mdl}` and `scale_k0`/`scale_regions_budget` **default-off**, plus an ignored harness `tests/cover_scaling.rs` with a pre-declared 3-condition exit rule (monotone region count; contrast >= 0.2506; held-out top-1 >= absolute baseline at the largest size). Nobody has run it. Folded in from #435, where it was filed under a claim that is now closed.

2. **Codebook fit.** `RVQ_SAMPLE_CAP = 10_000` (`compiler.rs`, applied at the `sampled_kmeans_rvq` call) hard-caps codebook training at 0.59% of the 2.11M construction split, and the caller discards the full-set codes — which is why #407's `CTX_SAMPLE` 6k to 50k raise never reached the codebook at all. The capacity-scaled run raised the effective sample to 12.4% and the gate passed, but that run also changed the cover, so the codebook term was never isolated.

**Concrete next step for whoever takes this:** run `cover_scaling.rs` at its four corpus prefixes and adopt-or-reject the relative/MDL/scaled-k0 criteria against its own exit rule; that single run discharges lever 1 and is cheap under the sampled Gate C and the code sidecar now on main. Lever 2 is a separate A/B holding the cover fixed. Note both are kappa-affecting if adopted, so a positive result needs the #407 re-pin ceremony.

### comment 5214946476 — 2026-08-07T08:51:58Z — Casey-allard

Taking **lever 2 (codebook fit)**, not lever 1. Your close-out note recommends `cover_scaling.rs` first as the cheap single-run discharge; I think lever 2 ranks higher on this board and want the reasoning on the record before I run anything.

**Why lever 2 first.**

1. **Lever 1's ceiling is small by construction.** The graph path saw 283 of 10,000 positions in the STAGES=5 run, and `exct.supported_record_fraction` is 0.97-0.99. Making the cover participate can only move positions that reach the cover, so lever 1 is capped in the low single digits of pp and realistically well under that. The codebook produces the *graded code itself* — the store key every position is looked up under — so it is not capped that way.

2. **There is already unclaimed measured signal pointing at it.** #407 closed with a recorded +2.1pp (34.8% → 36.9%) from retraining, attributed **+1.4pp era self-consistency / +0.5-0.7pp sample size**. But `docs/capacity_scaling_432.md` §3.1 then established that the `CTX_SAMPLE` 6k→50k raise *never reached the codebook*: `sampled_kmeans_rvq` immediately subsamples to `RVQ_SAMPLE_CAP = 10_000` and the caller discards the full-set codes. So the 6k→50k arm changed the draw pool and the extraction cost, and the codebook's training set stayed at 10,000 vectors in every arm. **The sample-starvation share of #407 was never actually tested.** That is a closed issue holding an untested attribution, which is why I am reopening the thread rather than treating #407 as settled.

3. **It is an evidence-quality lever, which is the only family that has paid.** Same `K`, same `STAGES`, same nominal key space, same budget — only the fit changes. The README pattern says every key-*resolution* lever failed and the two that helped improved evidence quality per key.

4. The audit's own words: *"the graded code's realized resolution grows as roughly N^0.55 while its codebook training set is literally constant — that sublinearity is the codebook cap showing through,"* and *"`code.codebook_sample_fraction` was already SATURATED at 500k; the 500k measurements everything else is calibrated against were themselves taken on an under-fit codebook."*

**Run contract**

```
metric to move:      held-out store top-1, pinned 500k fixture corpus
                     (2,507 stories, ~100k held-out), teacher-free
                     `compile_recorded`, cover held fixed
reachability ceiling: #407 recorded +2.1pp total from retraining, of which the
                     starvation share was never actually delivered to the
                     codebook. Current training fraction at 500k is
                     10k/400k = 2.5% (39 vectors per centroid per stage);
                     cap 200k takes it to ~50%. Ceiling is bounded by #407's
                     total +2.1pp and is plausibly a fraction of it.
instrument gate:     occupied full-code keys must rise MONOTONICALLY with the
                     cap. If the codes do not move when the codebook sees 20x
                     the data, the codebook is not under-fit, the audit's §3.1
                     diagnosis is wrong, and the top-1 arm does not run.
exit rule:           positive if held-out store top-1 at cap 200k exceeds
                     cap 10k by >= 1.0pp
if positive:         propose §5.3 scaling (RVQ_SAMPLE_CAP = min(N/10, 500k)),
                     which is kappa-affecting -> #407 re-pin ceremony
if negative:         codebook fit joins subdivision as measured-dead; the
                     audit's §5.3 recommendation gets withdrawn rather than
                     left standing as advice nobody tested, and the
                     evidence-quality family narrows to storage/calibration
cost estimate:       3 recorded compiles + 3 store builds on 500k, no teacher,
                     no Gate C. Sized before launch; cheap arm first.
```

**The pre-declaration that makes this more than a re-run of the STAGES=5 negative.** Both mechanisms are expected to lower records-per-key, for opposite reasons — STAGES=5 lowered it by adding key *resolution* and top-1 fell (thinner evidence per key); a better-fit codebook would lower it by putting evidence on the *right* key. So:

- occupancy up **and** top-1 up ⇒ fit and resolution are separable, and the evidence-quality thesis is confirmed on a second, independent lever;
- occupancy up **and** top-1 down ⇒ records-per-key is the binding quantity regardless of *why* it fell. That is a **stronger and more general negative than #460's**, and it would mean the STAGES=5 result was never about subdivision specifically.

Both branches change what we do next, so the run has decision value either way.

### comment 5216241820 — 2026-08-07T11:06:51Z — Casey-allard

## Outcome — lever 2, reported line by line against the run contract

PR #477. Record: `docs/codebook_fit_460.md`.

**Metric to move.** Held-out store top-1, pinned 500k fixture corpus, teacher-free recorded compile, cover held fixed. SHIPPED configuration measures **35.69% ± 0.15pp** (n=100,306).

| arm | pool | cap | train | occ. keys | rec/key | top-1 | vs shipped |
|---|---:|---:|---:|---:|---:|---:|---:|
| A | 10k | 10k | 10,000 | 77,799 | 5.14 | 35.53% | −0.16pp |
| **A — SHIPPED** | **50k** | **10k** | **10,000** | **79,244** | **5.04** | **35.69%** | — |
| A | 200k | 10k | 10,000 | 74,952 | 5.33 | 35.74% | +0.05pp |
| B | 50k | 50k | 50,000 | 85,496 | 4.68 | **36.14%** | **+0.44pp** |
| B | 200k | 200k | 200,000 | 79,933 | 5.00 | 36.06% | +0.37pp |

All five arms produced distinct codebook digests, so no comparison is vacuous.

**Exit rule.** Pre-declared: positive iff the largest training set beats shipped by >= 1.0pp. Measured **+0.37pp** at 200k, best **+0.44pp** at 50k — **NEGATIVE**. But not null: +0.44pp is 2.1 SE on the difference, using the conservative independent-sample form (the arms are paired on the held-out set, so the true error is smaller).

**Instrument gate.** I pre-declared occupancy must rise monotonically with the cap. It did not (77,799 → 85,496 → 79,933). I read that as the gate being a **bad gate** rather than as evidence of no effect — top-1 moved coherently while occupancy did not, so occupancy is not a reliable signature of relieved under-fit. Recording that so nobody adopts §5.3's proposed instrument reading, which says exactly this.

**Which pre-declared branch applies.** The negative branch, with one amendment. I said a negative means "withdraw the §5.3 recommendation". The honest version is **narrow it, don't withdraw it**: `N/10` = 39,969 at this corpus, essentially the measured saturation point, so the formula's *shape* is confirmed and only its 500,000 ceiling is unsupported. Adoption is κ-affecting for +0.44pp, so it belongs in the next re-pin that happens for another reason rather than triggering one.

---

## The part that changes the trajectory

**Records-per-key is a symptom, not the binding quantity.**

This issue's STAGES=5 outcome reads: records/key fell 36.02 → 18.80, top-1 fell with it, *"the signature of thinner per-key evidence rather than sharper context."* Here records/key falls **5.04 → 4.68** and top-1 **rises +0.44pp**.

Both changes thin evidence per key. They differ in why: added key **resolution** splits evidence that belonged together; better **fit** moves evidence onto the key that represents it. So thinning is not itself a warning sign, and this issue's causal reading should narrow to **resolution specifically**. The subdivision negative stands exactly as measured — only its explanation gets sharper.

That in turn forces a correction to the README pattern paragraph, which listed *"a better-fitted codebook"* among the levers that **failed** by adding key resolution. Wrong on both counts: fit is not resolution (fixed `K`, fixed `STAGES`, fixed nominal key space — it changes *which* key evidence lands on), and measured in isolation it is **positive**. The distinction worth carrying forward is **which key evidence lands on** (fit — helps) versus **how many keys there are** (resolution — has never helped). This is the first confirmation of the evidence-quality pattern on something touching the graded code itself rather than storage or calibration.

## Two corrections to `capacity_scaling_432.md`

**§3.1 is one step too strong.** It states #407's `CTX_SAMPLE` 6,000 → 50,000 raise "did not change the codebook's training-set size at all". The training set is `min(CTX_SAMPLE, RVQ_SAMPLE_CAP)` and at 6,000 the 10,000 cap **does not bind** — that raise moved the training set 6k → 10k, so #407's attributed starvation share was real, just small. What sweep A kills is raising `CTX_SAMPLE` further while the cap stands, which is every configuration since. The three points form one diminishing-returns curve: **6k→10k ≈ +0.5pp (#407), 10k→50k +0.44pp (here), 50k→200k nothing.** I was working from §3.1's stronger claim when I opened this and it was wrong; flagging it rather than quietly relying on the corrected version.

**§6's calibration worry is now bounded.** "The 500k measurements everything else is calibrated against were themselves taken on an under-fit codebook" is true, and the under-fit is worth **0.44pp**. That closes it rather than leaving open-ended doubt over every pinned row.

## What remains on this issue

**Lever 1 (cover split criterion) is untouched and still dark** — `cover_scaling.rs` unrun, `SplitCriterion::{RelativeGain,Mdl}` and scaled k0 still default-off. Leaving this issue open for it. Worth stating plainly for whoever takes it: with the graph path at 283 of 10,000 positions, lever 1's ceiling is low single digits at best, and this result gives no reason to expect otherwise.

The one follow-up that would change the codebook recommendation is whether the saturation point tracks `N/10` at 2.11M or is absolute near 50k. If it tracks, the lever stays worth ~0.44pp at every corpus size; if it is absolute, larger corpora are already past it and the cap is harmless. Not filing that as an issue — no owner and no next action today.

Unassigning myself; lever 1 is claimable.

### comment 5217247970 — 2026-08-07T12:49:28Z — Casey-allard

## Lever 1: the harness cannot currently answer the question, and it says PASS anyway

Picking up lever 1 (cover split criterion). Before any result, a defect in `cover_scaling.rs` that explains why this lever has stayed dark — running it today is worse than not running it.

**What happens on a default run** (committed fixtures, no `R4_STORIES`):

```
frac 13%: empty partition, skipped
frac 25%: empty partition, skipped
frac 50%: empty partition, skipped
CAP BOUND: frac 100% held-out positions 100306 -> 50000
-- frac 100%: stories <2507, 399694 train obs, 50000 held-out obs
ROW arm=absolute train=399694 regions=44 ... contrast_mean=0.6190 held_top1=0.1270

PASS absolute monotone_regions=true contrast_ok=true predictive_ok=true (top1 0.1270 vs baseline 0.1270)
```

Three of four sizes are skipped, and the harness then reports **PASS on a single data point** — `monotone_regions=true` is trivially true over one size, and `predictive_ok` compares the absolute arm against itself. That is a vacuous verdict of exactly the #354 class, in the harness whose whole job is to decide this lever.

**Mechanism.** The nested-prefix design needs each corpus-size prefix to contain both partitions. The `R4_STORIES` path satisfies that — `stories.jsonl` partitions by `blake3(article id) % 5`, which interleaves held-out stories across the whole id range. The **fallback** path does not: it is a contiguous tail split (`constr[sid] = sid < 0.8 * stories`), so held-out stories are all ≥ 2005 and *every* prefix below 80% has an empty held-out set. The fallback is the only mode available on the committed fixtures, and it is the default.

So the harness is correct for the corpus it was written against and silently degenerate everywhere else.

**Two fixes, both test-only, no κ consequence:**

1. Make the fallback partition interleaved (`sid % 5 != 0` → construction). Same 80/20 ratio, but spread across the id range so every prefix has both partitions.
2. Refuse the vacuous verdict: if fewer than two sizes produced rows, condition 1 is reported NOT EVALUATED and the arm cannot print PASS.

**Run contract**

```
metric to move:      cover capacity + emission contrast + held-out top-1 of a
                     region-path-keyed store, across corpus-size prefixes
reachability ceiling: SERVING impact is capped hard. exct.supported_record_fraction
                     is 0.9882 at the 4-stage baseline and the STAGES=5 run
                     resolved 283 of 10,000 positions on the graph path, so any
                     cover-side change is bounded by roughly 1-3pp of headline
                     and realistically well under. This run decides whether the
                     cover SCALES, not whether it pays.
instrument gate:     >= 2 corpus sizes must produce rows, or condition 1 is
                     unevaluable and no arm may pass
exit rule:           unchanged, as pre-declared in the module docs — monotone
                     regions; contrast mean >= 0.2506 at every size; held-out
                     top-1 at the largest size >= the absolute baseline there
if positive:         a criterion that scales becomes the adoption candidate;
                     kappa-affecting, so it needs the #407 ceremony, and it
                     should be weighed against the 1-3pp serving cap above
if negative:         cover split criterion joins subdivision and codebook fit
                     as measured; #460 closes with all three levers recorded
cost estimate:       ~30s per arm-size on the 500k fixture; 5 arms x 4 sizes,
                     minutes not hours
```

**Stated limit up front.** The defect this lever targets was observed between 500k (38 regions, contrast 0.5012) and 2.11M (14 regions, 0.0915). The committed fixture tops out at 500k, and the absolute arm there induces 44 regions at contrast 0.6190 — healthy. So this run tests whether the criteria scale **within** 65k-500k; it cannot reproduce the 2.11M collapse. If every arm passes within that range, the honest conclusion is that the harness needs the 2.11M corpus to discriminate, not that the criteria are fine. I will report which of those two happened.

### comment 5217418389 — 2026-08-07T13:04:04Z — Casey-allard

## Outcome — lever 1, against the run contract

PR #478. Record: `docs/cover_scaling_460.md`. With this, both levers on this issue are measured.

**Instrument gate.** Pre-declared: >= 2 corpus sizes must produce rows or condition 1 is unevaluable. After the fallback-partition fix, all four sizes produce rows. Gate PASSED — and the gate earned its place, because before the fix only one size did.

**Result — the shipped criterion fails, inside 500k.**

| train obs | 51,696 | 99,455 | 199,562 | 400,006 |
|---|---:|---:|---:|---:|
| `absolute` regions | 50 | 48 | 46 | 48 |

Flat and non-monotone over an 8x data range. The defect was previously inferred from 500k (38 regions) vs 2.11M (14 regions); it is visible inside a single 500k corpus.

**Result — capacity is the binding knob, and it works.**

| arm | @51k | @99k | @199k | @400k | held-out top-1 @400k |
|---|---:|---:|---:|---:|---:|
| `absolute` | 50 | 48 | 46 | 48 | 0.1307 |
| `scaled-k0@ref50k` | 50 | 63 | 71 | **104** | **0.1713** (+4.1pp) |
| `relative+scaled@ref50k` | 46 | 65 | 81 | **110** | **0.1809** (+5.0pp) |

Both PASS all three pre-declared conditions. Contrast 0.6649 / 0.6643 against the 0.2506 floor — above the `absolute` arm's 0.6217, so capacity was not bought by flattening emissions. Mean support per region falls 20,722 -> 8,886 while top-1 rises.

`mdl` scales (10 -> 26) but costs 1.7pp: capacity bought at a loss, which is what condition 3 exists to catch. `relative` alone stays flat — making the **bar** track the data does nothing while `k0` and the region budget stay fixed.

**Reachability, as pre-declared.** Serving impact is capped near **0.15pp**: `exct.supported_record_fraction` 0.9882, graph path 283 of 10,000 positions. I said before running that this decides whether the cover *scales*, not whether it *pays*, and that is exactly where it landed.

---

## Two traps I had to fix before any of the above was measurable

**1. The harness reported PASS on one data point.** A default run skipped three of four sizes (`empty partition, skipped`) and then printed `PASS absolute monotone_regions=true ... (top1 0.1270 vs baseline 0.1270)` — condition 1 evaluated over a single size (`windows(2).all(..)` is vacuously true over one row), condition 3 comparing the baseline with itself. Cause: nested prefixes need every prefix to hold both partitions; the `R4_STORIES` path interleaves via `blake3(article id) % 5`, but the **fallback** was a contiguous tail split, so every prefix below 80% had an empty held-out set. Fixed to an interleaved 80/20, plus an INCONCLUSIVE verdict for any arm with fewer than two sizes. This is the #354 class and I think it is the honest answer to why this lever sat unrun: the harness did not fail, it just said nothing.

**2. `scaled-k0` passes degenerately on this corpus — do not believe the shipped arm's PASS.** `DEFAULT_CAPACITY_REF_N = 400_000`; the fixture yields **400,006** train observations. At the anchor every scaled knob equals its base value, so at the largest size `scaled-k0` reduces *exactly* to `absolute` (identical 48 regions, 0.6217 contrast, 0.1307 top-1) and condition 3 compares it with itself. Its monotonicity comes from being throttled *below* the anchor, not from growth above it. The `@ref50k` arms exist to test the law rather than the coincidence. Anyone re-running this on a ~500k corpus needs to know that.

## Trajectory consequence

#435 and this issue's STAGES=5 outcome both recorded the cover as simply not participating — *"the absolute entropy floor still rejects every split at this scale"*, 8-22 regions at mass-kept 0.0006. That is true of the shipped **configuration**, and it was being carried as a property of the geometry. It is fixable with a knob that already exists and is default-off.

That does not make it worth adopting today — 0.15pp of headline is not worth a κ re-pin. What it changes is the standing of the broad-corpus directions: when #320's teacher upgrade or the #433 frontier drops exact-context coverage, the cover is **ready and measured** rather than a fresh unknown at exactly the moment it starts to matter. The bottleneck is now located: it is not the cover, it is exact-context dominance.

It also puts a boundary on the README pattern. Added resolution helps when a structure is *barely partitioned at all* — 48 regions for 400,006 records leaves each region's emission near the global prior. The resolution levers that failed all subdivided structures already resolved enough to be predictive. Resolution pays up to the point where a structure predicts, and not past it.

## Closing this issue

Both levers are measured and recorded: codebook fit +0.44pp and saturating (#477); cover split criterion scales under scaled capacity, +5.0pp on the region path, capped near 0.15pp at serving. Proposing #460 closes on merge of #478.

The one run that would justify adoption is `cover_scaling.rs` at 2.11M with a production anchor choice — same corpus the original 14-region collapse was seen on. Not filing it: no owner, and it is downstream of the broad-corpus work rather than parallel to it.

### comment 5217810513 — 2026-08-07T13:42:05Z — Casey-allard

Closing record (issue auto-closed on merge of #478, eb3da78; lever 2 landed in #477, 2bccf2f). Both levers measured.

**Lever 2 — codebook fit.** +0.44pp (2.1 SE), saturates near `N/10`, below the +1.0pp exit rule. `docs/codebook_fit_460.md`. Also established that records-per-key is a *symptom*, not the binding quantity — it fell here while top-1 rose — which narrows this issue's STAGES=5 causal reading to resolution specifically.

**Lever 1 — cover split criterion.** The shipped absolute floor does not scale (regions 50/48/46/48 over an 8x data range, visible inside 500k rather than only between 500k and 2.11M). Scaled capacity fixes it: regions 48 -> 110, region-path held-out top-1 **+5.0pp**, contrast maintained above the baseline's. Capacity, not the criterion, is the binding knob — `relative` alone stays flat. Serving impact capped near 0.15pp by exact-context dominance. `docs/cover_scaling_460.md`.

**Two harness defects fixed on the way**, both of which produced green-but-meaningless results: `cover_scaling.rs` skipped every corpus size below 80% under its fallback partition and then printed PASS over a single row; and `scaled-k0` passes degenerately on any ~500k corpus because `DEFAULT_CAPACITY_REF_N = 400_000` coincides with that corpus's full train size, making the arm identical to the baseline exactly where the verdict is read. Anyone re-running this needs the second one.

**What carries forward.** The cover's non-participation was being carried from #435 as a property of the geometry; it is a property of the shipped configuration, fixable with an existing default-off knob. Not worth a κ re-pin for 0.15pp today. The value is that when #320's teacher upgrade or the #433 frontier drops exact-context coverage, the cover is ready and measured instead of a fresh unknown at the moment it starts to matter. The bottleneck is now located, and it is not the cover — it is exact-context dominance at 0.9882.

The run that would justify adoption is `cover_scaling.rs` at 2.11M with a production anchor choice — the same corpus the original 14-region collapse was observed on. Not filing it: no owner, and it is downstream of the broad-corpus work rather than parallel to it.


---
_Comment/review ids in this file: 5207129272, 5207604588, 5210240288, 5211519367, 5212670087, 5214946476, 5216241820, 5217247970, 5217418389, 5217810513_

