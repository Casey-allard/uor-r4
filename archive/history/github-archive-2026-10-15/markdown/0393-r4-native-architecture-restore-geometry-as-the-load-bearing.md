# #393 — R4-native architecture: restore geometry as the load-bearing substrate (framing + program)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T23:31:43Z
- **Updated:** 2026-08-05T16:37:17Z
- **Closed:** 2026-08-05T16:37:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/393
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Diagnosis (file-level receipts)

The repo carries **two stacks that barely touch**:

1. **The R4 stack** (`uor-r4-router`): 512-dim state grounded on the zeta-zero grid (`zeta_zeros.rs`), 4D block-norm projection, Hopf sector transport on S³, VSA/spectral geometry, evolving session state. Last measured: #303 (2026-07-31). **Zero active board tracks touch it.**
2. **The serving/graph stack** (`uor-r4-graph-*`): R4G1 bundles whose semantic space is the sign-bit orthant structure of teacher hidden states (an LSH of the teacher's learned geometry) plus exact-context NGRAM rows (pure memorization).

Every 2026-08 campaign result lands in stack 2, and the campaign has now measured, repeatedly, that geometric *scoring bonuses* bolted onto stack 2 contribute zero or negative: veto-side distinctiveness (#379/#386), flat repetition penalty (#385), phase-clock addressing (#276/#392, recorded negative), and generalization below the unigram null (#391).

Additionally, `syntactic_morphism_score` (engine.rs) computes its Cayley–Dickson centralizer term over `CayleyDicksonVector::from_u32(token)` — a multiplicative hash of the raw token id. It carries no corpus or teacher content and has never been through the measurement discipline. It should be re-grounded or measured-and-removed.

## Claim

The objective is misaligned with the product role. Certify measures open teacher-forced next-token top-1 — transformer mimicry — under which memorized exact-context channels are provably the best strategy and geometry structurally cannot win. The system's actual role in the original hybrid was **syntax completion between routed content anchors** (the router supplied content tokens; a downstream LM arranged syntax). The serving stack exists to take over that completion job, and should be graded on it.

## Program (each piece falsifiable, filed separately)

- **Anchor-infill criterion** — constrained infill between routed anchors as a measured lane, with unigram-null discipline carried over from #391. (follow-up issue)
- **E8/icosian store spike** — construction-level geometry: store vectors as golden-coupled quaternion pairs (icosian ring ≅ E8 lattice, Turyn norm). Explicitly NOT covered by #276's negative, which tested phase as a scoring term, not storage substrate. (follow-up issue)
- **Construction-time structural stratification** (private track, existing).
- **Design doc PR** to follow tonight: two-stacks map, objective spec, what #392's negative does and does not rule out, and where router manifold / Hopf transport / E_f–E_b edges / FMM tables slot in.

Scoring-term bolt-ons on stack 2 are a measured dead end; remaining geometry work must change what is **constructed and stored**, or change what is **measured** — not add channels to the scorer.


## Comments (6)

### comment 5173519191 — 2026-08-04T01:24:46Z — Casey-allard

## Night-1 program summary (2026-08-04, ~01:30 UTC) — for the morning read

Six PRs merged since the program was filed (~3 hours): #396 (criterion harness), #397 (E8 spike M1), #398 (this doc), #401 (fusion probes), #402 (region conditioning + stride sweep), #403 in queue (E8 construction v0). Every result is on its issue; the one-paragraph version:

**The objective-misalignment claim is now measured, not argued.** On the anchor-infill criterion (#394): shipped store 34.7% on free positions, while a zero-past-context table conditioned on the next anchor hits 32.7% at the pre-anchor offset. Product fusion of the two channels reaches **43.4% overall / 50.5% pre-anchor** (#399 gate 1 passed) — beating the best open-row config on record. The forward channel is large, unconsumed, and composes near-independently with everything the causal store knows. Stride sweep: the anchor's backward shadow is ~50% strong at every cadence 3/4/5; cadence is a smooth dial (#394).

**Geometry results, both directions:** the anchor's compiled 3-byte graded code is a sufficient statistic for its syntax influence (region ≡ token table to 0.1pp, #399) — but the fixture is too dense to test generalization; the natural-corpus rerun decides. E8/icosian: representation-fidelity win at matched bits (#395 M1), flat spatial-group store keying negative with OOV flooding diagnosed (#395 v0, PR #403); residual-E8 with a retraining-controlled f32 baseline is running now (#395 v1).

**In flight:** #381 natural-corpus observation loop (~47%), then penalty arms → #381 close → natural-corpus rerun of all the above tables. #400 (CD-term A/B) queued next.

Gate policy remains with Alex & Ari per §9; everything above is measurement.


### comment 5180972196 — 2026-08-04T15:11:10Z — Casey-allard

## Scope qualifier on this week's negatives (Casey's direction note, on the record)

Every negative in the program record — the natural-text floor verdict, the granularity null, the forward-channel absence on wiki, the M0 exit-rule miss — is scoped to **534k open-text training positions with a 15M-parameter story-domain teacher**. At this scale even strong architectures would struggle to show anticipation signal on encyclopedic text. 'Ruled out at this scale' must not be read as 'ruled out': directions closed here may reopen at 10-100x data, and the test suite's specificity (teacher-forced, stride-anchored, token-vocabulary) may itself have filtered out paths that would work under other regimes.

Program response, per Casey: **substrate is the active path** (geometry organizing generalization between observed exact-context anchors — where the measured 36.5% predictive mass lives), and **scale runs in parallel** starting now: a 10k-article corpus (3.33x) is being fetched and observed in the background (~1 day of teacher observation), with the full decider suite to be rerun against it. Negatives that flip under scale get their directions reopened.

### comment 5181451405 — 2026-08-04T15:51:56Z — Casey-allard

## Substrate investigation complete: the 32pp reconciliation resolves to GEOMETRY SOURCE — and the answer is good news

The chase, recorded start to finish: (1) my n-gram suffix proxy could not reproduce the score pipeline's EXCT (8% hit rate / 3% accuracy vs 97.8% status / 36.5%); (2) mechanism read: `context_row` is trigram-with-bigram backoff — but the NGRAM section was EMPTY in the penalty arms (`--context-order 0`), so that isn't the source; (3) partition hypothesis (template-family siblings under the D3 hash split) — FALSIFIED by direct test: rerunning every table under the hash split changes nothing (trigram 4.1%, all arms at the floor); (4) the actual source: the EXCT section (`emit_residual_exct`) serializes a **graded code-keyed store built by the score compiler's own cover induction** with observed-continuation evidence.

**The complete evidence ledger on natural text (667k records, held-out):**

| store geometry | evidence | top1 |
|---|---|---|
| legacy distilled (teacher-representation codes) | teacher top-k | 0.1% |
| legacy distilled | observed continuations | 4.2% (≈ unigram floor) |
| **score-compiler induced cover** | observed continuations | **36.5%** (8× the floor) |

**Conclusion (substrate direction confirmed and sharpened):** geometric context addressing DOES work on natural text — when the cover is induced from the corpus rather than distilled from the teacher. The dead thing is specifically the legacy distillation geometry, which is what every harness arm this week measured. This is the two-stacks diagnosis (§2 of the architecture doc) landing at maximum concreteness: the serving/graph stack's induced cover is the substrate, it is already where E_f/E_b edges live, and the program's mechanism work (E_b walks, icosian storage/distances — all validated contract-native this week) now targets it directly.

**Standing corrections to this week's record:** the natural-text "floor verdict" applies to legacy-geometry channels and raw token tables, NOT to induced-cover addressing; the sequential-vs-hash split question is closed (no difference — both measured, both recorded). Scale track (10k-article observation, running) will retest all of this at 3.33× data.


### comment 5185554723 — 2026-08-04T23:01:44Z — Casey-allard

Program status 2026-08-04 EOD: 21 PRs merged today (#396-#398, #401-#417); closed #374, #394, #395, #400, #407 (+#381). Substrate resolved (induced-cover + observed evidence 36.5%, mechanism = chain-telescoped scoring + residualized EXCT). Forward-anchor channel measured end-to-end on #399: A-mode validated (+4.2pp live), naive two-pass refuted by drift probe — see #399 records. Era hygiene: CTX_SAMPLE 50k + fifth κ re-pin (#407 closed, CI-reproduced). Scale corpus (10k wiki articles) ~33% observed; at-scale reruns of the full arm set queued on completion.

### comment 5193613019 — 2026-08-05T15:14:16Z — Casey-allard

Scale campaign + capacity diagnosis recorded on #399 (falsifier-2). Program-level takeaway: the substrate's fixed-capacity constants (cover entropy floor, RVQ sample caps, entry caps) do not scale with corpus size — the 10k-corpus collapse is measured and mechanistically located; M-C1/M-C2 (dilution control vs capacity-scaled compile) are running and will either establish the scaling law or scope the substrate to its regime. All capacity knobs now env-overridable κ-neutrally (PR #432).

### comment 5194561923 — 2026-08-05T16:37:17Z — Casey-allard

Closing the program umbrella with all rows resolved or tracked (Casey, 2026-08-05): anchor-infill criterion → #394/#399 (closed, full records); E8/icosian spike → #395 (closed, design law recorded); design doc → shipped + addenda; router reconnection → #421 (closed, measured) with ancestry tracked in #434; structural stratification → confirmed unstarted, now tracked in the new issue above with its post-#399 reframing. Program verdict, honestly stated: geometry restored as load-bearing substrate FOR THE REDUNDANCY-RICH REGIME (A-mode shipped and validated there; router anchors carry real signal there); the broad-corpus regime is the open frontier and has its own program spine (#433) with stratification as the leading mechanistic candidate. Successors: #433 (frontier), #434 (geometry ancestry), stratification issue (above), #422 (router quality, active).


---
_Comment/review ids in this file: 5173519191, 5180972196, 5181451405, 5185554723, 5193613019, 5194561923_

