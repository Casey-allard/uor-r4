# #1638 — Lower geometric capture and temporal residual sources to q4

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T00:31:26Z
- **Updated:** 2026-10-09T13:17:24Z
- **Closed:** 2026-10-03T01:16:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1638
- **Merged:** 2026-10-03T01:16:17Z
- **Merge commit:** 39c4813e5add67ec20c7f4cc2ca24935ee2030c1
- **Base / Head:** main / UOR-Foundation:codex/geometric-event-age-q4-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Geometric capture and age still used wide learned coefficient sources, and F32 reconstruction of integer controller scores could change a hard capture action at a rounded tie. This adds explicit four-bit source formats and keeps native integer actions authoritative through the connected offline span/answer path.

Both retained parents refuse unchanged quarter-grid conversion without clipping. One declared offline positive event rescaling, and the exact initialized temporal prior plus a signed-q4 eighth-nat age residual, admit both. Legacy schemas retain their meanings. Reload regenerates source/table bindings; context, value, NoRead and composition numerical payloads remain unchanged after dependency metadata rebinding. The geometric training dispatch bypasses q/k and retains the existing biased softmax/tangent/Hamilton estimator.

The two retained conversions complete512 authored development rows:446→444 correct,1 gain/3 losses; seed1 CE worsens and seed2 CE improves. All actual-position persistent/whole-prefix output bits match, and independent reload, integer prefix causality, same-shape/distinct-sequence B2 and mismatched dependency rejection pass. Every refusal and changed row is retained. Zero optimizer updates.

Validation: source8713997d passes6 integer/10 distinct training-artifact cases; source093dc5e7 adds4 integer/8 training cases and builds the actual loaded driver. Final headb2631039 passes the same12 focused cases and release driver build in hosted M1 [run37084505954](https://github.com/UOR-Foundation/uor-r4/actions/runs/37084505954); all147 source hashes match and the rebuilt binary is byte-identical to the actual loaded executable. Root import failure21d347aa was repaired and retained. Actual answer-loss fixtures reach event transitions/age/head; independent review recomputes all saved predictions, CE, source transforms and frozen payload identities. Review and actual source/run identities are recorded in PR comments.

Scope: these are learned attention source coefficients, not four-bit output/table storage or a complete native language model. Expanded integer tables/fixed prior, unchanged float Stack tail, full-prefix access, natural grounded language and geometry advantage remain unfinished. The connected fixture still uses the older training reader; end-to-end learning through the retained strict reader is the next explicit integration task. No GroundedSession/peer-job edits, local Cargo, private model upload, stochastic serving or scale sweep.

References #1512. Parent PR #1635 is merged.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5963933191 — 2026-10-03T01:06:25Z — Casey-allard

Exact-head review: b26310392fafba2aa9d28f6af054701d0cc6290a

Authorship: root Codex self-review permitted by D19, with independent source/mathematical and saved-output reviews by specialist agents. No non-author approval is fabricated.

All147 numerical Rust/example source hashes equal the hosted-tested093dc5e7 source and actual local binary binding. Only two capability documents and loaded evidence were added after that run. New source bypasses q/k via geometric-address dispatch; hard native capture actions anchor forward; existing biased softmax/tangent/Hamilton credit remains declared. Both artifacts /1 and /2 preserve semantics; new age /3 uses architectural fixed prior plus learned signed-q4 residual. Reload regenerates coefficient and expanded bindings; no clipping/fallback, no serving inverse gauge. Numerical dependency payload equality is checked after metadata rebinding.

Loaded conversion ends446→444 correct across512 authored rows (1gain/3losses), zero updates. No model-quality promotion or perfect-imitation gate. Expanded tables/fixed prior remain wider integers; full-prefix access and float host tail remain outside final serving objective. Scope neither changes GroundedSession nor touches peer jobs. Saved-output independent review independently reconstructs answers/CE/transforms/payloads, while all-position/prefix/B2 assertions remain driver-executed. Manifest inventory/sizes are independently checked; BLAKE3 rehash by reviewer unavailable, driver seal/verify executed.

No source blocker found. Final exact-head actual hosted compile/tests must pass in run37084505954 before protected merge. Queue compatibility statuses are not executable checks. Documentation claim-wording and diff checks passed. #1512 stays open.


### comment 5963938256 — 2026-10-03T01:07:04Z — Casey-allard

# Independent loaded-result review — event/age Q4 conversion

Reviewer: Codex adoption_review_evidence. Date: 2026-10-02. Verdict: **PASS saved-result consistency, with a mixed conversion outcome; no delivery blocker in this evidence scope.** No inference, model fit, Cargo build/test, source edit, Git action, or modification beneath a sealed attempt was performed by this reviewer.

Execution-bound source is `093dc5e7b508af8066c65a02abd1b2161a42433a`; actual retained executable SHA256 is `6fdefffe7829c4fa826148502640dbaba8c27b3a60fca1316af6916438b7772d` (6,475,696 bytes). Both fixed conversions and both original quarter-grid refusals have exit-zero execution receipts, completed reports and sealed manifests. All 147 scoped source-file hashes match each execution receipt and the current source. This is loaded-data review, not an independent claim that the reviewer executed those workers or the hosted tests.

## Recomputed outcomes

All predictions were recomputed from the 40 saved answer logits using earliest maximum; answer CE was independently recomputed by stable log-sum-exp. The largest row CE discrepancy from the stored value was 8.89e-16. All 512 parent answer vectors match the original context-parent vectors in F32 bits. Original episode identities, parent predictions, full parent value traces, parent NoRead values and the two parent query-head reduction records match the saved context-parent rows.

| Seed / panel | Correct parent → converted | Gains / losses | Mean CE parent → converted | Changed event positions / affected rows |
|---|---:|---:|---:|---:|
| 1 original | 93 → 91 / 128 | 0 / 2 | 1.121969286391 → 1.122901631302 | 7 / 7 |
| 1 stress | 110 → 109 / 128 | 0 / 1 | 1.112053125768 → 1.137371101716 | 13 / 8 |
| 2 original | 124 → 124 / 128 | 0 / 0 | 0.448721902656 → 0.448523593174 | 4 / 4 |
| 2 stress | 119 → 120 / 128 | 1 / 0 | 0.573584675421 → 0.571944813328 | 0 / 0 |

Total: **446 → 444 correct of 512, one gain and three losses**, mean CE **0.814082247559 → 0.820185284880**. Event-action differences total **24 token positions across 19 rows**, not 24 rows. These are exposed, related construction/control rows; 512 is a descriptive row denominator, not a claim of 512 independent samples or new held-out generalization.

The two seed-1 original losses (rows 65 and 71) have unchanged event action sequences. The seed-1 stress loss (row 26, swapped_values) has an action difference at position 11/write_key; its head-0 correct-source mass changes from 0.969765 to 0.201563. The seed-2 stress gain (row 104) also has unchanged event actions. These are saved-data associations, not a counterfactual separation of event conversion and age conversion. All event transition/state trajectories change, so equal final action sequences do not imply unchanged controller states.

All 512 old and 512 converted action sequences contain a valid nonempty commit and a valid OLD held span at the answer query; neither set contains an all-HOLD episode. This validity was derived from saved OPEN/APPEND/COMMIT order, not from labels or a new model run. First action divergences are noise_key in 16 affected rows, write_key in two, query_key in one.

## Independent bindings and reconstructable checks

- Exact manifest inventory, regular relative paths and declared lengths pass for all **110 manifest-listed files / 417,442,148 payload bytes**. Current SHA256 values for every payload and all four manifests are in the companion JSON. No available BLAKE3 verifier was present, so the manifest BLAKE3 digests were **not independently rehashed**; completed `seal`/`verify` is execution evidence from the driver and exit-zero receipts.
- Actual executable bytes match the retained build verification receipt. Parent report/argument digests, original model/config/event parameter/event metadata digests and selected episode/parent-row hashes match actual files.
- Context source/native, value, NoRead and composition numerical payload files are byte-identical to their retained context parents. The exponential table is unchanged. Rebinding changes their dependency metadata only.
- Every saved converted event shadow is exactly the original F32 shadow times the fixed dyadic factor: TT/ST/NT × 1/2, TE/SE × 1/4. This check compared parameter bytes after the declared exact dyadic transformation. No scale selection or clipping was inferred.
- Both admitted attention metadata objects have an empty `wider_source_components` list and strict entries for context, potential, values, NoRead, events and age. Age is explicit reducer schema /3 with fixed initialization prior plus an eighth-nat residual. This classifies learned attention sources, not the entire floating Stack tail.
- Stored old and converted event hard actions are independently consistent with earliest argmax of their respective stored integer event scores. Action-difference counts and affected rows agree with the reports. For every saved reduction row, causal occurrence count and occurrence-plus-NoRead total weight arithmetic agree.
- Original unchanged-quarter attempts preserve both explicit range refusals, no event panels and no integrated result; their completed admission decisions are not model-quality failures.

## Executed assertions kept separate

The driver asserts source/native event trace equality but persists only one converted trace plus `source_native_exact`, so equality of two separately persisted traces cannot be independently reconstructed. Likewise, all-actual-position whole-prefix/session logit-bit equality, integer prefix causality, same-shape short/B2 comparisons, and rejection of mismatched valid value/context dependencies are fail-closed assertions in the execution-bound source with exit-zero receipts. Their booleans are not substituted here for independently stored paired outputs. Independent output checks above concern the persisted answer vectors, old/new traces and parent joins.

The earlier source-review condition is resolved: F32 prefix/B2 comparisons now use same-shaped native references, cross-length causality compares integer rows, B2 takes distinct retained rows, and parent answer joins compare F32 bits. No further source-review blocker remains in the root-owned driver/adapter scope.

## Recommendation

Preserve and deliver this fixed-conversion construction with its original refusals and all changed rows. It admits strict learned event/age sources and records a small mixed retention loss; it does not show a quality gain, a geometry advantage, natural-language ability or complete native model serving. The result can inform the next explicit learning bridge without repeating scale sweeps or requiring perfect parent recovery as a new gate. Any same-model training claim must use the actual strict context/value/NoRead/composition consumers; the existing connected floating answer-credit fixture alone does not establish that full route.

Companion machine-readable evidence: `independent-loaded-result-review.json`.


### comment 5964002131 — 2026-10-03T01:15:24Z — Casey-allard

Final exact-head validation PASS at b26310392fafba2aa9d28f6af054701d0cc6290a. Hosted M1 run37084505954 executes4 integer and8 training/artifact tests (all pass), then successfully builds the release integrated driver. All147 numerical source hashes match both retained artifact inventories; binary SHA6fdefffe7829c4fa826148502640dbaba8c27b3a60fca1316af6916438b7772d is byte-identical to the executable used for the actual512-row comparisons. Documentation claim-wording/diff checks pass; exact-head self/source and independent saved-result reviews are posted above. No outstanding material blocker in this bounded deliverable. Delivery-evidence status is advisory; compatibility statuses are not tests. Proceeding through protected merge under D19. #1512 stays open.



---
_Comment/review ids in this file: 5963933191, 5963938256, 5964002131_

