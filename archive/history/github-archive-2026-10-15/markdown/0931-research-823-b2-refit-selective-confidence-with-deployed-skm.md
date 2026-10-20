# #931 — research/#823-B2: refit selective confidence with deployed SKMX/PSIB content evidence under the frozen gates

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-23T02:47:40Z
- **Updated:** 2026-08-25T18:29:26Z
- **Closed:** 2026-08-25T18:28:57Z
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/931
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #823

## Context

#837 closed with `NO CALIBRATOR ESTABLISHED`: its suffix support×margin table ranked confidence but produced only a too-thin high-precision slice, raw margin was singleton-polluted, and every suffix-feature arm discarded all 2,454 content-answerable novel-suffix positions. The S1 redesign subsequently delivered deployed RF-31 SKMX/PSIB content-conditioned evidence through #897, #904, #906, #908, and #910. The maintainer verdict on #823 sanctions exactly one content-side calibrator re-entry under the unchanged #838 constitution.

## Problem

It is unknown whether the deployed SKMX/PSIB content evidence separates reliable next-token decisions from false answers at useful coverage, or merely improves aggregate prediction. A threshold chosen after reading the untouched split, a proxy result described as semantic certification, or a content feature computed differently from RF-31 would make the result uninterpretable.

## Goal

Run one bounded, artifact-only calibrator bake-off using production-equivalent RF-31 content evidence. Select at most one deterministic integer/table candidate under the frozen #838 risk/coverage rule. A positive report activates #839 phase 2 without retuning; a negative, research-only, or unavailable report is a legitimate completion and leaves #839 in legacy-coverage mode.

## Scope

- [ ] Reproduce the canonical #833 corpus/artifact identity, #837 story-disjoint FIT/CAL/untouched-TEST partition sizes, 246.6‰ locked base predictor, and #837 suffix-only reference curve before interpreting a new result.
- [ ] Reproduce the #908 deployed RF-31 replay identity and headline counts (base 19,372; skip-mix 21,424 of 72,130) before accepting content features.
- [ ] Fit SKMX/PSIB only from the canonical TRAIN split and derive bounded integer features from the same joint-row, fallback-row, candidate-injection, and winner-selection semantics used by the deployed lane.
- [ ] Record joint/fallback row availability, winner support, conflicting-row count, winner/runner content contributions and saturated margin, base agreement, injected-candidate status, base margin, candidate count, and the locked #837 suffix support/margin features.
- [ ] Compare, in fixed budget order: `content-margin`; `content-support-margin` (bounded lookup table); and `hybrid-content-suffix` (new table plus the locked #837 table). Prefer the simpler arm on a true tie.
- [ ] Preserve non-selectable SKMX-only, PSIB-only, no-injection, and candidate-injection-without-confidence ablations.
- [ ] Fit tables on FIT only, choose the operating point on CAL only, and consume TEST exactly once only if an arm release-qualifies on CAL.
- [ ] Emit a deterministic CID-bound result containing identities, feature exposure, curves, budgets, controls, ablations, selected threshold/model if any, and exact claim status.
- [ ] Append the historical #837 record; reconcile the current S2 summaries without rewriting prior evidence.

## Non-goals

- Change the #838 thresholds, powered semantic benchmark, status vocabulary, or claim gate.
- Implement #839 phase-2 artifact format, runtime decision, CLI/server/WASM lowering, or cross-surface certification.
- Add a teacher, network, clock, RNG, or remote fallback to inference.
- Treat improved SKMX accuracy, corpus-proxy risk, coverage, or D4 novelty as semantic answerability.
- Tune on TEST, retrofit a threshold after the result, or mint a new conformance capability for this research leaf.

## Native dependencies and entry gate

This leaf is a native child of #823, ordered immediately before #839. It is blocked by closed #838 (frozen semantics and gates), #837 (locked negative baseline, partitions, and falsifiers), and #910 (deployed RF-31 content evidence). It natively blocks #839 phase 2. All required code/artifact predecessors are closed; the canonical local #833 serving bundle is required for the empirical run.

## Candidate contract

All selectable candidates must lower to deterministic, bounded integer/table behavior:

1. `content-margin`: winner-versus-runner SKMX/PSIB residual contribution threshold.
2. `content-support-margin`: a bounded support × integer-log-margin precision table.
3. `hybrid-content-suffix`: a bounded deterministic combination of item 2 with the reproduced, locked #837 suffix support×margin precision.

Quantization, saturation, missing-feature behavior, row precedence, ties, table dimensions, bytes, reads, and projected operations are frozen in the harness before CAL is read. Compiler/certifier reference work may allocate; the candidate form must remain compatible with the later P-4, fixed-capacity, allocation-free runtime lowering owned by #839.

## Controls and falsifiers

- [ ] Always-serve, always-decline, current D4, and the exact locked #837 suffix-only table.
- [ ] Margin-only, support/count-only, constant-score, and inverted-score controls.
- [ ] Deterministic label shuffle, feature shuffle, and content-key/last-token pairing shuffle; neither shuffled null may qualify.
- [ ] Candidate injection without content-confidence scoring.
- [ ] Absent SKMX/PSIB identity and explicit missing-feature behavior.
- [ ] Planted leakage, label-leak, inverted-threshold, integer/reference-drift, corrupt/truncated row, and saturation cases.

## Binding cheap instrument

Run before the full sweep and publish its verdict:

- Verify the canonical bundle, corpus/artifact/report identities, and non-vacuous recorded teacher labels.
- Assert FIT/CAL/TEST story identities are pairwise disjoint and keep CAL/TEST labels out of table fitting.
- Reproduce #837 and #908 baseline readings exactly.
- On FIT only, require non-degenerate content feature availability and variance, a bounded packed/reference feature differential of zero, and a label-aware feature-cell oracle whose reachable coverage clears at least one frozen operating-point floor.
- Require content-key shuffle to change the content features and the leakage detector to fail planted contamination.
- Record the full #838 8-category gold-suite identity as `UNAVAILABLE` if its real annotation fixture is absent; a planted reference fixture or corpus next-token proxy is never semantic PASS.

If the instrument fails, do not consume CAL or TEST; close with `NO CALIBRATOR ESTABLISHED` or `UNAVAILABLE` and the exact failed prerequisite.

## Frozen selection and exit rules

- **Release candidate:** on CAL, maximize coverage subject to the unchanged UCB95 false-answer proxy bound ≤ 10‰ and coverage ≥ 20‰; ties use the frozen deterministic rule. Open TEST once and confirm the same threshold without retuning.
- **Research-only:** record UCB95 ≤ 50‰ at coverage ≥ 50‰, but do not activate production.
- **Semantic claim boundary:** the token-level teacher-grounded corpus proxy can select a candidate for #839; it cannot establish the #838 semantic claim. Full 8-category, four-axis-disjoint false-answer/false-abstain certification (release false-abstain ≤ 200‰; research ≤ 300‰), production-path identity, and cross-surface equivalence remain #839 phase-2 gates. Missing category evidence is `UNAVAILABLE`, never PASS.
- **If positive:** publish `SELECT-CANDIDATE`, activate #839 phase 2, and pass the frozen model/threshold/identity report forward without retuning.
- **If research-only:** publish `RESEARCH-ONLY / NO PRODUCTION CALIBRATOR`; #839 remains legacy-only.
- **If negative:** publish `NO CALIBRATOR ESTABLISHED`; preserve all curves/falsifiers and give #823 an evidence-backed LIMIT/REVISE input.
- **If required evidence is absent:** publish `UNAVAILABLE`; do not substitute a planted or different-CID fixture.

## Acceptance criteria

- [ ] Candidate extraction and lowering are bounded, deterministic, integer/table based, and tied to exact RF-31 row semantics.
- [ ] The locked #837 baseline and deployed #908 reading reproduce non-vacuously on the pinned bundle.
- [ ] FIT/CAL/TEST isolation, threshold provenance, and single-use TEST behavior are machine-checked.
- [ ] Every selectable arm and required control has a reproducible curve, byte/read/operation budget, and negative-control verdict.
- [ ] The selected arm, if any, beats the locked #837 model at its selected point, retains content-answerable novelty, and confirms on TEST without retuning.
- [ ] Double-run and reordered-input result bytes/CID agree.
- [ ] The result and documentation state exactly `SELECT-CANDIDATE`, `RESEARCH-ONLY`, `NO CALIBRATOR ESTABLISHED`, or `UNAVAILABLE`, with scope and limitations.

## Verification

- [ ] Fast fixture teeth for feature extraction, primary/fallback precedence, quantization, saturation, tie behavior, and missing/corrupt rows.
- [ ] Planted leakage, constant, inverted, shuffled-label, shuffled-feature, key-shuffle, and integer/reference-drift negatives.
- [ ] Canonical ignored instrument and full study, with a non-vacuous fixture-presence check and deterministic result-CID recomputation.
- [ ] Issue-specific tests first, then workspace no-fail-fast tests, clippy all targets/features, fmt, graph-format no_std ladder, claim-wording, conformance/generated-file determinism, doc tests, and other applicable repository gates.

## Run contract

- **Metric/current value:** false-answer UCB95 among served at useful coverage on the teacher-grounded corpus proxy. No production calibrator exists. The locked #837 suffix table reached 11‰ observed error at 10‰ coverage and UCB 41‰ at the 20‰ release floor, missing the 10‰ release bound.
- **Reachability ceiling:** compute on FIT before fitting from content-feature exposure and a label-aware feature-cell oracle. Existing RF-31 evidence gives 65,049/72,130 positions with content support (901.82‰), 39,360/72,130 lane changes (545.68‰), and 50,706/72,130 deployed misses (702.99‰ absolute improvable-error ceiling); the binding instrument must recompute the partition-specific ceiling.
- **Binding instrument verdict:** proceed only if identities/baselines reproduce, features are non-degenerate, leakage/null teeth pass, packed/reference extraction agrees, and the FIT oracle clears a frozen floor.
- **Exit rule:** the frozen rules above; no post-result threshold changes.
- **If positive:** activate #839 phase 2 with the exact report.
- **If negative/unavailable:** leave #839 legacy-only and record the stage decision input.
- **Cost estimate:** teacher-free single-process offline work; expected seconds for the instrument and roughly 1–4 minutes for fit/replay, approximately 2 GiB peak RSS; no hours-long run and no shared serving outage.

## Compatibility, migration, and conformance

All outputs are experimental offline records and rejected by production loaders until #839 defines the validated optional section and admission rules. This leaf changes no serving behavior, artifact compatibility, protocol, or migration policy. Execution scope is offline compiler/certifier analysis of production-equivalent RF-31 tables; it is not a deployed-serving result. It maps to RF-01, RF-22, RF-23, RF-29, RF-30, and RF-31 as evidence only, adds no `model/ids.toml` row, and never edits generated `CONFORMANCE.md` directly.

## Required deliverables

- An issue-numbered integration harness under `crates/uor-r4-api/tests/`.
- A CID-bound JSON result under `docs/`.
- An append-only issue record under `docs/`.
- An append-only follow-up in `docs/selective_calibration_837.md`.
- Current-summary reconciliation in `docs/RESEARCH.md` and `docs/r4_intelligence_completion_plan.md` as the verdict requires.

## Comments (9)

### comment 5383873408 — 2026-08-23T02:49:52Z — Casey-allard

### Pre-registered run contract — #931 content-evidence calibrator re-entry

This contract is posted before the binding instrument and before any CAL/TEST consumption. The run is teacher-free and minutes-scale; recorded `t_argmax` labels are the only target labels.

- **Metric/current value:** UCB95 of wrong deployed skip-mix next-token predictions among served positions on the canonical corpus proxy, at useful coverage. No production calibrator exists. The locked #837 suffix table reached 11‰ observed error at 10‰ coverage and UCB 41‰ at the 20‰ release floor, so it missed the release bound.
- **Frozen folds:** FIT 24,064 positions / 199 stories; CAL 24,232 / 200; untouched TEST 23,834 / 200. At the per-fold floors: FIT release ≥482, research ≥1,204; CAL release ≥485, research ≥1,212; TEST release ≥477, research ≥1,192. Table fitting reads FIT labels only; threshold selection reads CAL only; TEST is opened once only after CAL release qualification.
- **Reachability ceiling:** the instrument recomputes this on FIT. Existing RF-31 evidence gives content support on 65,049/72,130 = 901.83‰ positions, lane changes on 39,360/72,130 = 545.68‰, and 50,706/72,130 = 702.98‰ deployed misses as the absolute improvable-error ceiling. These are prior whole-heldout values, not a substitute for the partition-specific instrument.
- **Candidates, fixed budget order:** `content-margin` → bounded `content-support-margin` lookup → bounded `hybrid-content-suffix` lookup. A simpler arm wins a true tie. All eligible scores are deterministic integers/tables with frozen saturation, missing, and tie rules.
- **Controls/falsifiers:** locked #837 table; always serve/decline; current D4; margin/count-only; constant/inverted; deterministic shuffled labels/features; content-key/last-token pairing shuffle; candidate-injection-without-confidence; absent sections; planted leakage and integer/reference drift. Shuffled nulls are never selectable and must fail qualification.
- **Binding cheap instrument:** PASS only if the #833 identities and #837/#908 headline readings reproduce non-vacuously; story folds are disjoint; FIT content features have availability and variance; packed/reference extraction agrees; the content-key shuffle changes features; leakage teeth fire; and a FIT-only label-aware feature-cell oracle clears at least one frozen floor. If not, CAL and TEST remain unread and the verdict is `NO CALIBRATOR ESTABLISHED` or `UNAVAILABLE`.
- **Exit rule:** on CAL, RELEASE means maximize coverage with UCB95 ≤10‰ at coverage ≥20‰; RESEARCH means UCB95 ≤50‰ at coverage ≥50‰. A release-qualified threshold is evaluated once on TEST without retuning. `RESEARCH-ONLY` never activates production.
- **If positive:** publish one `SELECT-CANDIDATE` report and activate #839 phase 2 with the exact frozen model/threshold/identity.
- **If negative/unavailable:** publish the full falsifier record, leave #839 in legacy-coverage mode, and return an evidence-backed S2 LIMIT/REVISE input.
- **Claim boundary:** this corpus-position proxy may select a candidate for production lowering; it cannot certify semantic answerability. The real #838 8-category/four-axis suite is currently absent and therefore `UNAVAILABLE`, not PASS; its false-answer/false-abstain and cross-surface certification remains #839 phase-2 work.
- **Cost estimate:** instrument seconds; full fit/replay approximately 1–4 minutes, about 2 GiB peak RSS, zero live-teacher/network work, and no shared-serving outage.


### comment 5383884552 — 2026-08-23T02:53:21Z — Casey-allard

### Pre-run contract clarification — predecessor reproduction must not consume sealed TEST

The implementation review found a conflict before any #931 instrument ran: recomputing #908's 19,372/21,424 all-heldout accuracy counts, or #837's all-heldout 246.6‰ accuracy, necessarily reads labels from the #931 TEST fold. That would violate the stronger pre-registered rule that TEST opens only after a CAL release qualification.

The binding interpretation is therefore:

- The committed, CID-bound #837 and #908 records and their merged harness pins are verified as closed predecessor evidence before the run.
- The #931 instrument recomputes the canonical identities, table/graph lowering equivalence, partition sizes, and FIT-only baseline/content readings. It does not score CAL or TEST labels.
- CAL is scored only after the FIT-only instrument passes.
- The exact all-heldout #837/#908 headline counts are recomputed by #931 only if CAL release-qualifies and TEST legitimately opens; otherwise their committed CIDs/counts remain referenced predecessor evidence, not a newly claimed reproduction.
- A negative instrument or CAL verdict therefore leaves TEST unconsumed. No gate, threshold, or positive branch moves.

This resolves partition provenance without relaxing the reproduction requirement: current-run facts are recomputed where legal, predecessor all-heldout facts stay content-bound to their already merged records, and TEST is not inspected merely to prove that it was once inspected by a predecessor.

### comment 5383942371 — 2026-08-23T03:10:50Z — Casey-allard

### Binding instrument outcome — **PROCEED** (FIT only; CAL/TEST still sealed)

The final pre-run instrument completed in 19.8 s, teacher-free, against the canonical #833 bundle. No CAL or TEST label was scored.

- **Identity reproduction:** corpus `blake3:aa9d1767…`; regenerated base graph `blake3:aaf98b68…`; regenerated skip graph `blake3:19eb04d7…`. These exactly match the CID-bound #837/#908 predecessor records.
- **Frozen folds:** 24,064 / 24,232 / 23,834 positions and 199 / 200 / 200 stories, pairwise story-disjoint.
- **FIT top-1 diagnostics:** deployed base 6,390/24,064; deployed skip-mix 6,958/24,064; locked suffix predictor 5,859/24,064.
- **Feature exposure:** winner support 999‰; nonzero content margin 985‰; RF-31 lane changed 547‰; selected token injected beyond base top-8 331‰. Content-key/last-token shuffle changed 22,090/24,064 feature rows (917‰). Packed/reference selected tokens and witness contributions agreed exactly throughout replay.
- **Full declared-family oracle ceiling (FIT only):** release-reachable at 3,408 served / 29 wrong, coverage 141‰, UCB95 10‰; research-reachable at 3,932 / 148, coverage 163‰, UCB95 39‰. Both clear their frozen floors/bounds, so the candidate sweep can distinguish the decision branches.
- **Semantic evidence status:** the real powered eight-category annotation fixture remains `UNAVAILABLE`; the planted #838 constitution fixture is not empirical PASS.
- **Verdict:** `PROCEED` — open CAL for the predeclared three-arm sweep. TEST remains sealed unless a selectable arm release-qualifies on CAL.

Preflight note: an initial instrument implementation used only the two-dimensional content-table cell as its oracle and printed STOP. Review caught that it did not upper-bound the declared hybrid feature family. Before any downstream label access, the oracle was corrected to the full declared content-plus-suffix cell. No threshold, floor, feature, or outcome branch changed; this comment records the binding full-family result.

### comment 5383974650 — 2026-08-23T03:20:28Z — Casey-allard

Binding outcome (2026-08-22)

- Instrument: PROCEED on FIT only. Corpus and regenerated base/SKMX+PSIB graph CIDs matched the pinned predecessors; feature exposure was non-degenerate (winner support 999 permille, margin 985 permille, lane changed 547 permille, injected 331 permille), content-key shuffle changed 22,090/24,064 positions, and the FIT oracle cleared a frozen gate.
- CAL: no selectable arm met the frozen release gate (UCB95 <= 10 permille at coverage >= 20 permille) or research gate (UCB95 <= 50 permille at coverage >= 50 permille). Best raw content-margin slices were 8/243 wrong with UCB95 46 permille at 10 permille coverage, 35/485 with UCB95 79 permille at the 20 permille release floor, and 217/1,226 with UCB95 180 permille at the 50 permille research floor.
- Falsifiers: label, feature, and content-key shuffles all failed both gates. Always-serve, always-decline, current D4, locked suffix, support-only, base-margin, constant, inverted, SKMX-only, PSIB-only, no-injection, and injection-without-confidence readings are recorded.
- TEST: remained sealed and was not consumed. No threshold was retuned.
- Powered semantic suite: UNAVAILABLE; constitution and planted fixtures are not substituted for the absent real 4,800-item annotation fixture.
- Determinism: two complete bundle runs reproduced result CID blake3:2de371465c31cbbe1dfd168bcc8ce6bd8ba2cebe43bce2fa5fbfae6fc09efd0e and file SHA-256 686e26ce55314fbf47455e40f711fe8f35af9bd7ecfe7f47fb64ccaf7898c178.

Verdict: NO CALIBRATOR ESTABLISHED. Semantic abstention remains NOT ESTABLISHED; #839 remains legacy-only. The single S2 re-entry is exhausted, so the next S2 stage action is a maintainer verdict on parent #823, not another automatic fit.

### comment 5384100783 — 2026-08-23T03:57:44Z — Casey-allard

### Superseding binding instrument outcome — **PROCEED** (FIT only; CAL/TEST still sealed)

This append-only correction supersedes the numeric instrument readings in the earlier comments. Post-review hardening made deployed eligibility explicit, checked the complete RF-31 feature vector, verified every compiler row against its packed SKMX/PSIB representation, and corrected the effective candidate union. The instrument was then rerun before the audited full study or any new CAL access.

- **Identity reproduction:** corpus `blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce`; regenerated base graph `blake3:aaf98b68a78dd615f06dbb727a22dc4e170a152f055313fcc4fa574309f42d1e`; regenerated skip graph `blake3:19eb04d7dbf3fccd126069982ad8cbc1de31d536fff7e77ef2dacb26e64106cc`.
- **Frozen folds:** FIT/CAL/TEST positions 24,064 / 24,232 / 23,834 and stories 199 / 200 / 200, pairwise story-disjoint.
- **FIT top-1 diagnostics:** base 6,390; deployed skip-mix 6,958; locked suffix predictor 5,859.
- **Deployed D4 eligibility:** 24,041/24,064 eligible and 23 abstained. Candidates cannot resurrect an abstention.
- **Feature exposure:** winner support 999‰; nonzero content margin 985‰; lane changed 546‰; injected candidate 331‰. The content-key/last-token shuffle changed 22,097/24,064 feature vectors.
- **Differentials:** full packed/reference feature vectors agreed at all 24,041 eligible positions; all 811,421 compiler SKMX rows and 19,710 compiler PSIB rows matched their packed rows.
- **FIT-only declared-family oracle:** release-reachable at 6,962 served / 43 wrong, theta 333; research-reachable at 7,161 / 203, theta 195. Both clear the frozen instrument gate. This oracle is a reachability ceiling, not a selectable model.
- **Semantic suite:** the real powered eight-category fixture remains `UNAVAILABLE`; no planted fixture or corpus proxy is called semantic PASS.
- **Verdict:** `PROCEED`. CAL may now be opened for the frozen three-arm sweep. TEST remains sealed unless an arm also clears the locked-#837 improvement, content-caused novelty, and all null-control admission checks.

Elapsed time was 22.3 s, teacher-free. The earlier 22,090 shuffle count and 3,408/29 oracle reading must not be used as final evidence.

### comment 5384193613 — 2026-08-23T04:24:04Z — Casey-allard

### Superseding audited final outcome — `NO CALIBRATOR ESTABLISHED`

This append-only outcome supersedes the result CID, SHA-256, and pre-audit counts in the earlier “Binding outcome” comment; that comment remains as historical evidence.

#### Binding FIT instrument

- Corpus: `blake3:71b34a8fca54df5bcb716046ad2f6f07b52c857131ca461d4c9ec6b0df49f5ac`
- Base artifact: `blake3:24b48b67484cbbe4cf63dbf79560f08704a7bb0d23d59d186e6c78fde77cccf9`
- Skipmix artifact: `blake3:1b86659edb993f8ae38db877a7c942d6cb42e9fea9c38a91f3479be41bf97241`
- Split sizes: FIT 24,064 / CAL 24,232 / TEST 23,834 positions; 199 / 200 / 200 stories.
- FIT content hits: base 6,390; skipmix 6,958; suffix 5,859.
- D4 eligibility: 24,041 eligible; 23 current abstentions.
- Feature range: support 999‰, margin 985‰, content lane changed 546‰, injected prediction 331‰.
- Feature differential: all 24,041 eligible FIT positions changed under the planted feature negative.
- Key shuffle: 22,097 FIT positions changed.
- Packed/source audit: all 811,421 compiler SKMX rows and 19,710 PSIB rows matched the packed artifact.
- Frozen oracle: release 6,962/43 at θ=333; research 7,161/203 at θ=195.
- Semantic suite: `UNAVAILABLE` because the real annotations, generator, rubric, and split identities do not exist. Instrument verdict: **PROCEED** for the proxy experiment only.

#### CAL result

No selectable arm beat the matched predecessor UCB floor at the frozen service levels:

| Arm | 10‰ served/wrong, error/UCB ‰ | 20‰ | 50‰ |
|---|---:|---:|---:|
| content-margin | 243/8, 32/46 | 485/35, 72/79 | 1,226/217, 176/180 |
| content-support-margin | 3,020/1,752, 580/582 | 3,020/1,752, 580/582 | 3,020/1,752, 580/582 |
| hybrid-content-suffix | 299/34, 113/124 | 485/57, 117/124 | 1,269/188, 148/151 |

The historical #837 suffix-predictor reference was independently reconstructed as 261/3 (11/23‰), 549/19 (34/41‰), and 1,350/170 (125/129‰). Applying that locked suffix score to the skipmix artifact produced 261/19 (72/85‰), 549/43 (78/84‰), and 1,350/214 (158/161‰); the two references are intentionally distinct.

CAL D4 had 24,222 eligible positions and 10 current abstentions. The current policy was wrong on 16,980 eligible positions; always-serve was wrong on 16,990 of all 24,232 positions. CAL feature hits were base 6,518; skipmix 7,242; suffix 5,995; SKMX 7,248; PSIB 2,361; no-injection 7,261. The current-run content-caused novelty proxy counted 517 CAL positions, but no selected served set exists because no arm qualified.

All planted nulls passed non-vacuously:

- label shuffle changed FIT 9,974 / CAL 10,190 positions;
- key shuffle changed FIT 22,097 / CAL 22,335 positions;
- feature negative changed FIT 24,061 / CAL 24,228 positions;
- saturation, quantization, integer/reference drift, corrupt/truncated input, missing-section, and reorder-determinism teeth passed.

#### Sealed holdout and evidence identity

TEST remained untouched: `test_opened=false`. `cal_candidate`, `selected`, `test_evaluation`, and `all_heldout_reproduction` are all null.

The missing semantic-suite identity is explicit:

- manifest CID: `blake3:19e6c0f9051567f3ecb421aa92b78515598ecfe49168e5b9d67ff9cff4c8b31e`
- constitution CID: `blake3:37f770b346ad03471a4757cd21adbcac7c5ac39bb1b6aa51e7c8d8b378ed2e9b`
- annotations / generator / rubric / split CIDs: null (`UNAVAILABLE`)

Two complete audited runs produced identical evidence:

- Result CID: `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`
- Result file SHA-256: `838b4591f126f6ab0d2f767a976f24c07782b9932910831e6c9dff3a019d7ade`
- Wall time: 41.4s and 41.2s.

**Verdict:** `NO CALIBRATOR ESTABLISHED: no selectable arm met the frozen release or research gate on CAL; TEST stayed untouched and #839 remains legacy-only`.

Accordingly, #839 must retain the legacy selective confidence path. The next action is the maintainer promotion/limitation verdict on parent #823; this negative result does not authorize a production-policy change.

### comment 5384217051 — 2026-08-23T04:30:50Z — Casey-allard

### Identity correction to the superseding audited outcome

The three input-identity lines in the immediately preceding comment were transcribed from the wrong bundle. They are corrected here, append-only; all counts, curves, null verdicts, result identity, and the `NO CALIBRATOR ESTABLISHED` verdict in that comment remain unchanged.

- Corpus metadata CID: `blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce`
- Regenerated base graph CID: `blake3:aaf98b68a78dd615f06dbb727a22dc4e170a152f055313fcc4fa574309f42d1e`
- Regenerated skip-mix graph CID: `blake3:19eb04d7dbf3fccd126069982ad8cbc1de31d536fff7e77ef2dacb26e64106cc`

These values match the audited harness, `docs/selective_calibrator_reentry_931_result.json`, and `docs/content_evidence_calibrator_931.md`.

### comment 5396388102 — 2026-08-24T14:08:35Z — Casey-allard

### Verification blocker and parking record (2026-08-24)

The fixture-present workspace gate is **ABORTED, not PASS**. `cargo test --workspace --no-fail-fast --offline` was stopped cleanly after more than 34 hours; cargo exited 130. The BDD suite therefore has no final PASS verdict.

Evidence recovered before stopping:

- S2 completed 36 exact live-teacher forwards in about 4h29m, roughly 448 s/forward on this machine.
- A one-second read-only stack sample showed the process healthy and still inside S4 `timed_teacher_generate`, executing the exact `uor-matmul` kernel rather than hung.
- The old S4 shape performs seed/warm-up work plus three sequential 128-token generation runs, hundreds of additional exact forwards, with no counter, worker telemetry, durable progress, or defensible ETA.
- Missing conditional evidence remains UNAVAILABLE; an interrupted suite is not converted into a pass.

The issue implementation and evidence files remain preserved on `issue-931-content-evidence-calibrator`; nothing has been staged, committed, pushed, or discarded. Native child/blocker #932 now owns the exact-parallel, fully instrumented parity-harness repair. This issue is parked behind #932 and is being unassigned so active ownership remains truthful. After #932 merges and its short scaling gate qualifies, #931 can resume its required non-vacuous verification without restarting the former black-box design.

### comment 5414914047 — 2026-08-25T18:29:24Z — Casey-allard

### Definition of Done — completed via PR #938

Merged through the protected queue as `9fe85b6e59d117f6319f841591a867cc2606eebd`.

- Final verdict: **`NO CALIBRATOR ESTABLISHED`**.
- Binding FIT instrument: `PROCEED`, 20.5 s, all pinned corpus/base/skip identities reproduced.
- Teacher-free CAL sweep: 38.4 s; no selectable arm met the frozen release or research gate.
- Nulls: label, feature, and content-key shuffles were non-vacuous and none qualified.
- TEST remained sealed: `test_opened=false`; no threshold was retuned.
- Semantic fixture: `UNAVAILABLE`; semantic abstention remains `NOT ESTABLISHED`; #839 remains legacy-only.
- Result CID: `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`.
- Result SHA-256: `838b4591f126f6ab0d2f767a976f24c07782b9932910831e6c9dff3a019d7ade`.

The protected merge-group run passed the full binding ladder: workspace nextest, BDD, doc tests, fmt/clippy, no-std, deterministic rebuild, κ reproduction, all-features tests, wasm, fuzz, audit, conformance, and Gate C trend.

Queue evidence: https://github.com/UOR-Foundation/uor-r4/actions/runs/32881645725
PR: https://github.com/UOR-Foundation/uor-r4/pull/938



---
_Comment/review ids in this file: 5383873408, 5383884552, 5383942371, 5383974650, 5384100783, 5384193613, 5384217051, 5396388102, 5414914047_

