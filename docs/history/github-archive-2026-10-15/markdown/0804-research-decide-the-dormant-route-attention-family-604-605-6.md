# #804 — research: decide the dormant route-attention family (#604/#605/#606/#643) — fit against a real teacher on the amended corpus, or retire with a record

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T21:21:21Z
- **Updated:** 2026-08-19T05:26:18Z
- **Closed:** 2026-08-19T05:26:18Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/804
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The dormant route-attention family — `r4-route-attention` (#604), `route-fit` (#605), `target-operator-certificate` (#606), `msa-structured-selector` (#643) — is the largest untested mechanism in the tree and the closest transformer-shaped, goal-aligned one (the ROADMAP itself named it a #745-era candidate lever). It is built, packed, P-4-scanned, and differentially tested bit-for-bit with witness replay and zero-alloc asserted — but it has **never been fitted against a real teacher**. Its fate currently lives scattered across a closed issue's comments (#605), the closed reassessment (#786 → `docs/dormant_reassessment_786.md` S1/S3), and a governance gap (#531 closed COMPLETED with no closure record; its saturation corpus is unlocated). This issue exists so the question gets **answered** — fitted and measured, or retired with a record — instead of staying parked.

Why now: the #783/#785 arc fixed the serving-side decode path (the CLI now serves the scored graph on real evidence), and the #784 instrument showed the remaining quality collapse is **argmax over converging marginals** (distinct contexts sharing the corpus-dominant continuation), not code collision. A fitted attention-support operator re-ranking candidates with long-range context is precisely the mechanism class that could address that — which makes this the right time to find out whether it actually does.

## Evidence on record

- #605: synthetic 5-stage ladder PASS with a valid instrument; real-teacher stage was UNAVAILABLE (snapshot absent — **now present** on the dev host at `.uor-models/sources/smollm2-135m-instruct`, 257 MB); real-corpus stage UNAVAILABLE (#531 corpus never located).
- #604: packed lowering uses only P-4 §2 operation classes; differential + witness replay + zero-alloc asserted (synthetic).
- #643: msa structured selector built + differentially tested; its pre-registered A/B compares against a **fitted** route-attention baseline, so it is strictly downstream of S1.
- #606: certificate composes #605's outputs; NOT_PASSING by construction until a real ladder exists.
- #786 corpus-era check: no dormant negative was measured on an ordering-corrupted corpus; the #509 `smollm2-360m-broad` corpus is 0% non-contiguous.

## D1 — corpus prerequisite: DECIDED (maintainer, 2026-08-18)

**Amend the #605/#643 pre-registered contracts to use the existing #509 `smollm2-360m-broad` corpus** (cheap-check-first): if the fit fails its gates on a real broad corpus, the saturation-corpus question is moot; if it passes, the #531-method saturation corpus can still be produced afterward for the stronger claim. This amendment is hereby declared on-thread per the pre-registration discipline — the run below uses the 360m-broad corpus, not a silently substituted one.

## Scope

- **D2 — run S1, the real-teacher route-fit arm.** Requires the #603 `full/1` trace-profile observation pass over the amended corpus (the fit consumes attention-support traces; this is the actual compute cost — hours-class, so it launches only after the run contract below is confirmed on-thread with a duration estimate). Then the #605 ladder under its own pre-registered gates: support overlap vs N1/N2 nulls; teacher-forced top-1 ≥ 0.90; bits ≤ 1.10× teacher; anti-vacuity N2 < 0.5× fitted. **Exit rule: stop at the first failing stage.** Identical next-actions on both branches = drop the run (none here: the branches diverge).
- **D3 — positive branch:** fitted instances land; #606 certificate rows go real; run S3 = the #643 msa-vs-route-attention A/B per its pre-registered contract. Then the **activation question returns to the maintainer** with the ledger gate in hand ("a measured lift clearing the pre-declared bar") — activation stays gated on explicit sign-off, never automatic, and no serving default changes ship from this issue.
- **D4 — negative branch:** the negative report closes the question with the operator retained dormant, plus a named maintainer decision: retain (dormant, negative on record) vs remove (reduce surface). Either answer closes this issue with a record.

## Run contract (pre-declared; finalize duration estimate before launch)

- Metric: the #605 ladder's own pre-registered gates (above) on the amended corpus.
- Null/falsifier: the ladder's N1/N2 nulls are the falsifiers; the instrument already failed honestly on synthetic-vs-real distinctions once (#605's UNAVAILABLE records), so it can fail.
- Compute: dominated by the #603 `full/1` trace-profile observation pass (hours-class; estimate to be posted on-thread before launch per AGENTS long-run discipline, with checkpoint/harvest plan).
- Outcome handling: positive → D3; negative → D4. Both end in records here.

## DoD

D1 decided (done, above); the S1 contract confirmed with a compute estimate and executed (or explicitly declined on-thread); positive branch → S3 + certificate + the activation decision recorded; negative branch → the retire-or-retain decision recorded. Either path ends with the dormant-attention question **answered**.

**Next action:** post the #603 trace-profile compute estimate for the 360m-broad corpus, confirm the run window, then launch S1.


## Comments (4)

### comment 5334408881 — 2026-08-18T21:41:29Z — Casey-allard

## S1 compute estimate + run plan (executing D1: amend to `smollm2-360m-broad`)

**Corpus (fixed by D1).** The #516-pinned broad corpus: **360,924 records over 2,994 contiguous Simple-Wiki stories** (seq-len 128, D3 partition rule, story-contiguous — the #755 ordering defect class measured at 0% here). The amendment naming this corpus in place of the unlocated #531 saturation corpus will be declared on #605 and #643 **before** any run, per the pre-registration discipline; the #531-method saturation corpus remains a possible follow-up after a positive, not a prerequisite.

**What S1 actually runs.** Re-observe the same 2,994 articles under a #603 trace profile carrying the attention-support lane, then run the #605 ladder's real-teacher/real-corpus stages against the produced bundle. Verified against the fit code: `route_fit_report.rs` consumes **supports + logits only** (`step.supports[lane][head]` for the Jaccard/N1/N2 overlap gates, logits for top-1/bits parity) — no q/k/v, no residual lanes.

**Profile decision (recommendation embedded).** Teacher config (pinned source): 32 layers × 15 heads, GQA 5 KV heads, hidden 960.

| profile | lanes beyond minimal | sidecar disk at full depth (32L, S=8) | verdict |
|---|---|---|---|
| `attention-support/1` | per-head top-S supports | 360,924 × 32 × 15 × 8 × 8 B ≈ **11 GB** | **recommended** — carries every lane the fit reads |
| `full/1` | + q/k/v rows + per-layer residuals + final hidden | ≈ **130+ GB** (all layers) / ≈ 33 GB (8 declared layers) | does not fit on the box today (~69 GB free) at full depth; extra lanes are unread by the fit |

So the amendment will name **`attention-support/1`, all 32 layers declared, support cap S=8** (`PRIMARY_TOP_K`), unless you prefer `full/1` at reduced layer depth.

**Wall-clock anchors (measured, this box, native CPU/Accelerate).**

- 360M teacher forward: **74.5 tok/s** (`docs/smollm2_teacher_baseline_320.md`).
- The identical corpus at `minimal` profile: the #516 pass ran **overnight** through the resume daemon — the honest precedent.
- Support capture taps the #602 exact-executor path, whose throughput under capture is unmeasured → estimate **overnight-class (~8–14 h)** assuming ≤1.3× capture overhead. This is the one soft number; the smoke step below replaces it with a measured one before you're asked to commit the window.

**Sequence (cheap-check-first).**

1. **S1-0 — CLI wiring (day-class, standing cadence).** `r4 transformerless observe-text` does not expose `--trace-profile` today: #603's registry/bounds/sidecar machinery is fully built at library level (enforced in `observation.rs::validate_registered_trace_profile`) but `src/main.rs` has no flag. Small slice + tests.
2. **S1-smoke (minutes).** One-story pass under `attention-support/1`: measures true tok/s under capture, validates sidecar + manifest identity binding. Measured rate posted here.
3. **Contract amendments on #605/#643** — corpus, profile, declared bounds; every pre-registered gate (support overlap vs N1/N2, teacher-forced top-1 ≥ 0.90, bits ≤ 1.10× teacher, anti-vacuity N2 < 0.5× fitted; stop at first failing stage) unchanged. Posted before the run.
4. **S1 observation (overnight-class — needs your explicit go + a window).** Resume-daemon pattern exactly as #516.
5. **Ladder + #606 certificate rows (hour-class, same day after).** Positive → fitted instances + certificate go real, S3 becomes runnable; negative → the negative report closes the question with the operator retained.
6. **S3 (#643 A/B) consumes the same bundle** — one observation pass funds both probes; no second overnight.

**Ask:** confirm (a) profile = `attention-support/1` at full depth, (b) an overnight window for step 4. Steps 1–3 proceed under the standing cadence; step 4 waits for the explicit go per the long-run rule.


### comment 5334754568 — 2026-08-18T22:18:07Z — Casey-allard

**Correction to the estimate above, before any run** (caught at the loader itself while wiring): the profile recommendation was wrong. `route_fit::load_route_trace_corpus` — the function that feeds the #605 real arm — **refuses `attention-support/1`** with a typed error: the *fit* consumes the teacher's **q/k rows and supports** (`qkv_lane` + `attention_support_lane`, aligned layer lists); only the evaluation *gates* are supports+logits-only, which is what the earlier comment measured. The amendment therefore names **`full/1`**.

Full depth does not fit (32 layers ≈ 362,240 B/record ≈ 131 GB vs ~69 GB free — exact row math from `TraceRowLayout` for this teacher: per declared layer 11,200 B = residual 960×4 + q/k/v (960+2×320)×4 + supports 15×8×(4+4); plus final-hidden 3,840 B once), so per maintainer decision (2026-08-18, **superseding the earlier attention-support/1 answer**):

> **`full/1`, declared layers `[0, 4, 9, 13, 18, 22, 27, 31]`, support cap S=8 (`PRIMARY_TOP_K`), full 2,994-story `smollm2-360m-broad` corpus** → 93,440 B/record ≈ **33.7 GB** sidecar, ~35 GB headroom.

The whole-model ladder stage runs over the declared 8-layer set; widening the layer list is a follow-up pass gated on a positive — the same cheap-check-first spirit as D1 itself.

Also verified while reading: the capture stack is complete at library level — `observe_sharded_traced`, `HuggingFaceLlamaOracle::step_with_trace_capture` (through `Llama::forward_capturing_trace`, the production executor path), the sidecar writer with resume reconciliation, and `load_route_trace_corpus`. What's missing is exactly S1-0: the **text-observation pipeline** (`observe-text`, which the broad corpus uses for teacher-forcing) and the CLI have no trace plumbing. Wiring now; smoke numbers next.


### comment 5337082099 — 2026-08-19T03:19:00Z — Casey-allard

**Assigned + actively in progress** — the S1 evidence run is executing right now: the traced full/1 observation of the amended corpus is at ~69% (ETA ~05:05Z), and on completion the landed `route-fit-real` runner (#812) executes the pre-registered S1-1 fit + overlap gates on the declared 1000×64 subset, with the verdict posted here and on #605. Please don't pick this up in parallel — the corpus run is hours of single-machine state that would be redone from zero.

### comment 5337906136 — 2026-08-19T05:26:17Z — Casey-allard

**Closing with the decision this issue chartered: the dormant route-attention family (#604/#605/#606/#643) stays DORMANT, now with real-teacher evidence on the record.**

The S1 evidence run completed overnight exactly as amended and pre-registered on #605:

- **S1-0 corpus**: 3000 articles / 361,693 records traced under `full/1` (row bytes 93,440 manifest-verified), ~5h48m under the approved measurement-only Accelerate exception — full record: #605 issuecomment-5337863280.
- **S1-1 fit + overlap gates**: **FAIL — instrument VACUOUS.** Fitted support overlap 0.396 vs N1 (permuted codes) 0.192 and **N2 (next-position supports) 0.292** — the pre-registered anti-vacuity gate (N2 < 0.5 × fitted) failed because teacher attention supports are temporally smooth on this corpus, so the instrument cannot attribute fitted structure to genuine per-position routing. 119/120 heads individually vacuous; every deployed-kernel runtime check passed across 7.5M packed steps. Full numbers + κs + honest reading: #605 issuecomment-5337903765.

## The decision, per the pre-declared tree

- **Operator retained dormant** — `r4-route-attention-dormant` stays behind its unchanged ledger activation gate. Nothing activated, nothing deleted; the kernel remains the differentially-tested, P-4-legal artifact it was, now with a recorded real-teacher measurement attempt.
- **S1-2 and the #643 A/B do not run** (both were gated on an S1-1 PASS).
- **Any future attempt needs a new pre-registered contract** with a vacuity-robust instrument (excess-over-N2 as the primary metric, or restricted-forward parity directly). Not scheduled; maintainer approval required.
- **Findings worth keeping**: (1) teacher attention supports are temporally smooth at this scale — successor instruments must beat temporal persistence, not chance; (2) layer 9 head 0 was the sole head clearing individual vacuity (fitted 0.432 vs n2 0.202) — a pointer, not a claim.

## What this unblocks

Per the standing maintainer decision ("wait for the S1 verdict, then decide"), the **#784-family** question — output distinctness / context-code convergence, plus the D4 semantic-OOD finding from #811 — now returns to the maintainer for its own contract, with route-attention no longer a pending alternative lever. The roadmap's standing next action (recompile the canonical bundles with the #755 fix + fresh quality read, toward a server-admissible release bundle) is unchanged and is the recommended first step.

*Run discipline note: this issue was assigned and marked in-progress while the multi-hour single-machine run executed, per the repo convention; with the verdict recorded, the work is complete and the issue closes. Zero ledger rows changed.*


---
_Comment/review ids in this file: 5334408881, 5334754568, 5337082099, 5337906136_

