# #933 — architecture/#821-F: restore normative R4G1 serving reachability, bind deployed quality, and re-ratify RF-31

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-24T21:07:55Z
- **Updated:** 2026-08-25T06:12:24Z
- **Closed:** 2026-08-25T06:10:41Z
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/933
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Parent tracker / stage

#821 — S0 truth and inference closure. Milestone: **R4 Intelligence / S0 — Truth and inference closure**.

## Problem and intended outcome

ADR-0001 / #831 designates `uor-r4-graph-runtime::R4G1Runtime` as the sole normative deployed token selector. `R4Engine` / `GraphScorer` is reference/certifier scope plus D4 policy resolution; results from it cannot automatically be credited as deployed-serving evidence.

The live implementation and evidence have crossed that boundary:

- #833's cited `r4 ask` path called `load_accepting_quality`, so it established explicit bypassed local loading rather than strict production admission.
- #908 measured a real 29.702% teacher-argmax result through an explicitly off-serving `R4Engine` harness.
- #910 registered RF-31 as `normative-runtime` / `deployed-serving`, but changed `R4Engine` and compiler emission without adding SKMX/PSIB consumption to `R4G1Runtime`.
- Local chat selects through `R4G1Runtime`, while HTTP greedy, default sampled, API, and WASM paths have split scorer/decode behavior; default sampled generation bypasses the skip-mix reroute.
- The quality report consumed by strict admission scores certifier Rule 1+2, not the exact normative served-token implementation.
- The current attested `smollm2-360m-broad-clean` graph predates #910, lacks SKMX/PSIB, and its report records 24.39% Rule 1+2 versus 28.12% TLA, so it does not establish the promoted serving claim.

Historical evidence remains intact. This issue appends the correction, restores one implementation and one evidence boundary, re-emits the canonical broad graph, and returns S0/RF-31 to a truthful evidence-backed verdict.

## Execution scope

**Normative runtime / deployed-serving** plus the offline compiler/certifier and governance changes required to bind exact production evidence.

## Objective

Establish one CID-bound production pipeline in which every production decode surface obtains ranked candidates and served tokens from `R4G1Runtime`; D4 may decline but cannot substitute an independently selected token. Version deployed-quality evidence around that exact path, re-emit the attested canonical `smollm2-360m-broad-clean` graph, and teacher-free re-ratify, limit, or retire RF-31 and the affected S0 claim.

This is a concrete foundational S0 child, not a tracker.

## Exact scope and non-goals

### A. Correct the current truth ledger

- [ ] Append a correction to #821: the earlier promotion record did not establish strict normative served-token admission/reachability.
- [ ] Append a correction to #833: preserve source/corpus/determinism attestation while correcting the production-quality attribution.
- [ ] Append a correction to #908/#910: preserve 29.702% as valid `R4Engine` reference/off-serving evidence, not an `R4G1Runtime` deployed-serving result.
- [ ] Reconcile `README.md`, `ROADMAP.md`, `docs/RESEARCH.md`, the authoritative completion-plan mirror, ADR references, baseline summaries, and per-issue records without rewriting historical entries.
- [ ] Reaffirm ADR-0001: `R4G1Runtime` remains the sole normative candidate/token selector. A superseding ADR or a third scorer is out of scope.

### B. Put RF-31 in the normative runtime

- [ ] Implement bounded, allocation-free SKMX/PSIB candidate contribution in `uor-r4-graph-runtime`.
- [ ] Preserve fixed-point ordering, saturation, token-ID tie-breaks, bounded probing, no-double-counting, patch precedence, P-4, `no_std`, and witness contracts.
- [ ] Preserve exact absent-section identity: artifacts without SKMX/PSIB produce byte-identical candidates, tokens, statuses, and witnesses.
- [ ] Add witnessed attribution for whether SKMX or PSIB changed the selected candidate.
- [ ] Keep `R4Engine` policy-only on production paths: D4 may Serve/WidenOnce/Abstain but may not replace the normative runtime candidate/token.
- [ ] Fail closed when policy and normative selection cannot be reconciled.

### C. Unify production decode surfaces

Route all production entry points through one shared `R4G1Runtime` candidate/selection adapter:

- [ ] CLI/local `ask` and chat, greedy, sampled, and beam.
- [ ] `src/tless_uor.rs` graph generation.
- [ ] `/v1/chat/completions`, `/v1/responses`, `/api/chat`, `/api/r4g1/predict`, and `/api/r4g1/generate`.
- [ ] Public library and WASM production facades.
- [ ] Greedy, pinned-seed sampled, and beam decode consume the same normative ranked candidates; decode policy is named separately from scorer identity.
- [ ] Cross-surface fixtures assert identical first-step ranked candidates and tokens for identical graph/window/decode mode/seed.
- [ ] Planted divergences fail if any surface uses `GraphScorer` for the served token, ignores SKMX/PSIB, discards the normative token, or bypasses RF-31 in sampled mode.

### D. Version and bind deployed-quality evidence

Introduce a versioned normative deployed-quality profile/report that records:

- [ ] Schema and scorer-semantics versions; execution scope `normative-runtime`; selector identity `R4G1Runtime`.
- [ ] Graph, teacher artifact, corpus, partition manifest, tokenizer, compiler revision/configuration, active-section set, decode mode, and seed identities.
- [ ] Full-census versus sampled status, population/sample sizes, paired both/scorer-only/TLA-only/neither counts, paired delta, and declared uncertainty.
- [ ] TLA comparator definition and exact same-position partition.
- [ ] Witness replay count/failures and negative-control identities.
- [ ] Release-manifest binding to the deployed-quality report CID.

Production admission must:

- [ ] Verify report identities against the loaded graph/artifact/tokenizer/release manifest.
- [ ] Reject missing, mismatched, sampled-as-census, unknown-schema, wrong-scorer, and legacy off-serving reports with typed errors.
- [ ] Keep explicit research loading available with a typed warning; it must never be reported as production admission.
- [ ] Never reinterpret a legacy `relative_tla` report as normative-serving evidence.

### E. Re-emit and teacher-free re-ratify the canonical broad bundle

Using the existing attested `smollm2-360m-broad-clean` artifact, clean corpus, tokenizer, and recorded `t_argmax` labels:

- [ ] Re-emit the graph through the production compile path with SKMX/PSIB and the new report binding.
- [ ] Rebuild twice and prove byte-identical graph/report/manifest outputs.
- [ ] Admit from an empty model root through the normal production loader.
- [ ] Run the binding reachability sample first; run the full 72,130-position held-out census only if it can change the decision.
- [ ] Replay a deterministic report sample independently from the emitted evidence.
- [ ] Exercise every production surface against the re-emitted bundle.
- [ ] Record exact elapsed time, peak RSS, storage, worker count, throughput, phase counters, and progress/ETA so this path is not a black box.

No teacher forward pass, observation run, source-model download, or hours-class parity run is authorized here.

### Non-goals

- No live teacher inference or teacher/corpus rebuild.
- No quality tuning beyond faithfully lowering the already-promoted SKMX/PSIB mechanism.
- No lowering or opportunistic raising of the same-report TLA comparator, the frozen RF-31 +20 per-mille floor, or selective-prediction gates.
- No claim of instruction following, reasoning, general intelligence, or free-running coherence.
- No rewrite of historical evidence.
- No new parallel scorer or superseding ADR.
- No implementation or completion of #932's separate parity-performance harness in this issue.

## Dependencies / blockers

Closed prerequisites: #830, #831, #832, #833, #908, #910. Required canonical local inputs are present and will be identity-checked before use.

This issue is the native prerequisite for #932. The downstream chain is #932 -> #931 -> #839, with #823 and later sequential stages waiting on restored S0 truth. No active PR or branch currently implements this contract.

## Metric and run contract

Current same-distribution historical values:

- canonical TLA comparator: 28.121%;
- canonical Rule 1+2 reference row: 24.393%;
- #908 skip-mix `R4Engine` result: 29.702%, retained only as reference/off-serving evidence;
- frozen RF-31 causal floor: paired improvement over the sections-absent base with 95% lower bound at least +20 per-mille.

Primary metrics:

1. `delta_tla = normative R4G1Runtime top1 - same-position TLA top1`;
2. `delta_lane = normative SKMX/PSIB top1 - normative sections-absent top1`;
3. cross-surface candidate/token mismatch count.

Binding cheap instrument:

- deterministic first 6,000 held-out positions;
- proceed only if SKMX/PSIB is reachable, cross-surface mismatch count is zero, `delta_lane` is positive and compatible with the frozen floor, report binding passes, and all negative controls have teeth.

Full-census positive verdict requires:

- cross-surface mismatch count exactly zero;
- paired 95% lower bound for `delta_tla` at least zero;
- paired 95% lower bound for `delta_lane` at least +20 per-mille;
- label-shuffled SKMX/PSIB control has no positive promotion-class effect;
- absent-section identity is exact;
- witness replay has zero failures;
- production admission succeeds only with the correctly bound full-census report.

Reachability ceiling: only the 27.5% historical non-EXCT/fallback population can move from RF-31, so a +2.845pp headline result is reachable but must be reproduced on the normative selector. The 6,000-position sample is binding; no full census launches when its interval cannot clear the predeclared verdict.

Cost: teacher-free graph re-emission, 6,000-position probe, and conditional 72,130-position census; expected minutes-scale. If any phase projects beyond one hour, post a revised arithmetic/run contract before proceeding.

If positive: adopt the re-emitted canonical graph/report/manifest, ratify RF-31 at deployed-serving scope, and restore a corrected S0 promotion.

If quality-negative: do not activate RF-31; retain the plain/TLA fallback, record LIMIT or RETIRE with exact paired evidence, and do not credit #908 as deployed evidence.

If reachability/binding-negative: stop before the census when decided, keep RF-31 NOT ESTABLISHED, and record the exact architectural limitation.

If a required fixture is absent: record UNAVAILABLE; do not substitute another bundle, distribution, scorer, or sampled report.

## Required falsifiers and negative controls

- [ ] Sections absent: exact identity with the pre-lane normative runtime.
- [ ] SKMX/PSIB present: a planted partner changes the normative candidate/token where predeclared.
- [ ] Corrupt SKMX or PSIB: typed parse/admission failure.
- [ ] Graph/artifact/corpus/tokenizer/partition/report CID mismatch: typed admission failure.
- [ ] Report naming `GraphScorer` or off-serving scope: production rejection.
- [ ] Label-shuffled SKMX/PSIB: conditioning-specificity null.
- [ ] Deliberately divergent chat/server selector: cross-surface test failure.
- [ ] Default sampled mode deliberately routed through base candidates: test failure.
- [ ] Missing canonical fixture: UNAVAILABLE, never PASS.

## Acceptance criteria

- [ ] `R4G1Runtime` is the only implementation selecting production candidates/tokens.
- [ ] Every named production surface is mechanically reachable to it.
- [ ] SKMX/PSIB affects normative serving when present and is exactly inert when absent.
- [ ] Default sampled serving consumes RF-31 rather than bypassing it.
- [ ] The deployed-quality report is versioned, CID-bound, full-census identifiable, and fail-closed at production admission.
- [ ] Canonical broad graph/report/manifest outputs rebuild deterministically.
- [ ] Teacher-free sample and conditional full census produce a truthful RATIFY/LIMIT/RETIRE/UNAVAILABLE verdict.
- [ ] #821, #833, #908, #910, roadmap, research, ADR, ledger, and conformance agree.
- [ ] Conditional fixtures are confirmed present before any green result is credited.

## Compatibility and migration

- Artifacts without SKMX/PSIB remain byte- and behavior-identical.
- Previously emitted artifacts containing dormant SKMX/PSIB must not silently acquire new serving semantics: require the new report/profile/manifest binding or explicit re-emission.
- Legacy reports remain readable for historical/research use but cannot authorize production.
- Version any affected inference/report contract and document the recompile boundary.
- Preserve historical bundle readers where safe; otherwise return a typed migration-required error.

## Conformance mapping

RF-31 must be corrected in the required built-capability order:

1. update `model/ids.toml` so RF-31 names `R4G1Runtime`, actual production surfaces, and the versioned report;
2. replace table-lookup-only Gherkin with executable end-to-end reachability, absent-section, sampled-path, binding, and planted-negative scenarios;
3. make the registered marker/behavior test fail before implementation;
4. implement the normative runtime and serving unification;
5. regenerate `CONFORMANCE.md`; never edit it directly.

Until the positive verdict lands, RF-31 is NOT ESTABLISHED at deployed-serving scope. Maximum claim on completion is structural reachability plus CID-bound empirical quality for the exact measured artifact/population/decode surfaces.

## Verification

- [ ] Issue-specific graph-runtime, serving, report/admission, corruption, and planted-negative tests.
- [ ] `cargo test --workspace --no-fail-fast --offline`
- [ ] `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `cargo check -p uor-r4-graph-format --no-default-features`
- [ ] `cargo check -p uor-r4-graph-format --no-default-features --features alloc`
- [ ] Applicable graph-runtime `no_std` checks.
- [ ] `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`
- [ ] `python3 scripts/check_claim_wording.py`
- [ ] Repository conformance/register validation and generated-file determinism.
- [ ] Allocation census and P-4/GNAF operation scan.
- [ ] Deterministic graph/report/manifest rebuild.
- [ ] Cross-surface greedy/sampled/beam parity and target-binary audit.
- [ ] Conditional fixture existence checks before fixture-gated verdicts.

The protected merge queue supplies the binding full-workspace verdict.

## Documentation reconciliation

Append a new `docs/normative_r4g1_quality_<issue>.md` evidence record. Reconcile current summaries in README, ROADMAP, RESEARCH, the authoritative completion plan, the transformerless baseline, ADR-0001 references, model ledger, and any configuration/compatibility guidance. Add append-only corrections to the historical #833/#908/#910 records rather than replacing their original evidence.

## Definition of Done

Close only after one outcome branch is recorded with exact artifact/report/result CIDs, counts, paired intervals, falsifier verdicts, resource measurements, protected merge commit, issue truth corrections, regenerated conformance, verified state on `main`, and accurate parent/dependency status.


## Comments (4)

### comment 5401806082 — 2026-08-24T21:47:46Z — Casey-allard

### Pre-run contract refinement — canonical graph re-emission and controls

Required canonical inputs are present and byte-checked before launch:

- corpus.meta: 25 B, `blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce`
- corpus.records: 31,761,312 B, `blake3:4692307368fecce481a4aac452fd8df4e63d2f1bd07ee0f2932108f8595f8f62`
- TLA artifact: 1,415,444 B, `blake3:6324aabec22fca5af371333cefc206f9b6762bfb52dccfb8efa0dc8fe5a1efaa`
- tokenizer: 528,975 B, `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1`
- cached cover: 192,432 B, `blake3:df6902bc3ed3e5dee2f93fd8cc08b0614f298a6c059cb5806e9bda0b9f61520b`

Metric/current value: normative SKMX/PSIB reachability and paired top-1; current normative result is NOT ESTABLISHED. Historical comparators are TLA 28.121%, sections-absent Rule 1+2 24.393%, and off-serving #908 29.702%.

Reachability ceiling: historical strict non-EXCT/fallback population is 27.5%; the target +2.845pp movement is inside that ceiling. The run must still reproduce it through `R4G1Runtime`.

Binding cheap instrument: one cached-cover score pass emits the real graph plus two same-generation controls: sections absent, and TRAIN-label rotation by n/2+1. Then the first 6,000 pristine held-out positions run through all three graphs using the exact normative adapter on 8 deterministic workers. Proceed only with nonzero lane reachability, zero cross-surface mismatch, positive lane interval compatible with +20 per mille, exact absent identity, zero witness failures, and a nonpositive shuffled-control effect.

Exit rule: stop before the 72,130 census on any binding/reachability/control failure or if the sample interval cannot clear the frozen decision. Stop and post a revised contract if any phase projects beyond one hour.

If positive: run the full census, deterministic double-build, empty-root admission, and package the CID-bound report. If negative: retain exact evidence and record LIMIT, RETIRE, or NOT ESTABLISHED; do not tune thresholds or substitute a scorer.

Cost: 8 M1 workers; cached inputs; no teacher, observation, download, or source-model execution. Expected minutes, temporary output under 250 MiB per build. Host currently has 29 GiB free and 55% system memory free. Phase logs, progress/rate/ETA, terminal counters, `/usr/bin/time -l` peak RSS, and output sizes are captured.

### comment 5403499657 — 2026-08-25T00:49:53Z — Casey-allard

_edited 2026-08-25T00:50:31Z_

## PARKED HANDOFF — 2026-08-24

I am releasing active ownership. No canonical graph/evaluation run, pull request, merge, production admission, or RF-31 verdict was started.

Checkpoint branch: `issue-933-normative-r4g1-quality`
Checkpoint commit: `fb71e4ce78726481e9f4d60fcf19099935416d96`
Base/current `origin/main`: `9aeb5427c7fd3a965b0e4a0228f137889dbafa9b` (one commit ahead, zero behind at handoff)
Remote branch: https://github.com/UOR-Foundation/uor-r4/tree/issue-933-normative-r4g1-quality
Detailed record: `docs/normative_r4g1_quality_933.md`

Implemented checkpoint: bounded allocation-free SKMX/PSIB in `R4G1Runtime`; production decode unification; schema-2 CID-bound quality/admission; controls, parity, and witness replay; deterministic workers; exact progress/terminal/resource instrumentation; and governance/conformance/BDD/native/HTTP/WASM-facing tests/docs.

Final short validation passed: all-target workspace compile; graph-CLI compile; 3 focused observation/Gate C counter tests; skipmix label-control test; claim wording; 33-id model check; resource-wrapper smoke; format/diff check.

Not completed or claimed: full workspace tests/clippy/no_std/WASM/determinism/κ/allocation/P-4/queue ladders; canonical graph/control double-build; 6,000-position sample; conditional census; surface exercise; packaging/admission; PR/merge; or any empirical verdict. This 19k-line checkpoint is not ready for PR.

The missing foundational baseline investigation is captured as #934. It audits whether an absolute 30% requirement exists; reconciles 28.121% TLA, 24.393% Rule 1+2, 29.702% off-serving SKMX, and RF-31's +20-per-mille gate; reconstructs prior research/negative results; diagnoses the teacher-free same-position deficit; and pre-registers bounded remediation. #934 is a native S0 child of #821 and natively blocks #933. #932 remains downstream.

Next owner: begin with #934. Do not launch #933's canonical evidence phase until #934 resolves threshold provenance and remediation ordering.

### comment 5404289981 — 2026-08-25T02:29:57Z — Casey-allard

ACTIVE EXECUTION PLAN — 2026-08-25

Ownership is active. Current protected main is c29fa0e003aaa2176d7b4024508540d883490141; parked checkpoint is fb71e4ce78726481e9f4d60fcf19099935416d96. No canonical build, sample, census, package, or empirical measurement has started.

Pre-measurement contract amendment, frozen before new normative data:

1. Reconcile the checkpoint with merged #934 and add the missing status x TLA outcome x target-presence/rank x SKMX/PSIB contribution x toward/away diagnostic. Also measure the bounded EXCT/TLA semantic gap between the normative runtime and the reference row.
2. The sample ordering will be deterministic and content-bound but distributed across held-out stories rather than the ascending corpus prefix. Stage 1 evaluates 6,000 positions for structural reachability, binding, null teeth, and non-futility. If statistically inconclusive, the exact same order extends to 18,000 positions. A structural failure or an upper bound that cannot clear a frozen floor is STOP; lower bounds clearing both frozen floors is PROCEED; an interval overlapping a floor after stage 2 may authorize only the full census when the reachable ceiling remains sufficient and its measured projection is at most 60 minutes. A sample never authorizes production.
3. The full-census thresholds are unchanged: paired TLA lower bound at least zero, paired sections-present lower bound at least +20 permille, shuffled effect non-positive, absent identity exact, and zero binding, surface, or witness failures.
4. Emit and bind four diagnostic arms: current canonical base, re-emitted sections-absent base, sections-present SKMX/PSIB, and label-shuffled control. This separates the unresolved base/selector shift from lane credit.
5. Run no teacher, observation, download, parity marathon, or hours-class process. Every bounded phase must emit durable progress and terminal evidence and stop when projected above one hour.

Execution remains exactly one issue: finish #933 to a truthful RATIFY, LIMIT, RETIRE, NOT ESTABLISHED, or UNAVAILABLE outcome through the protected merge queue; #932 and #931 remain parked.

### comment 5406307619 — 2026-08-25T06:12:23Z — Casey-allard

## Definition of Done — completed via protected merge (2026-08-25)

PR #936 merged through the required merge queue as `e346816cb40b089583cbd3fff2cdd84924362c65`. The binding speculative-merge run completed successfully: https://github.com/UOR-Foundation/uor-r4/actions/runs/32814445068

### Final empirical verdict

**RATIFY only for the exact CID-bound canonical bundle, 72,130-position held-out population, `R4G1Runtime` selector, greedy decode, and schema-2 production envelope.**

- normative: 21,293 / 72,130 = **29.5203%**
- same-position TLA: 20,284 / 72,130 = **28.1214%**
- paired normative − TLA: **+13.988 permille**, 95% CI **[11.057, 16.919]**
- same-generation sections absent: 18,806 / 72,130 = **26.0723%**
- paired sections present − absent: **+34.479 permille**, 95% CI **[31.681, 37.277]**
- TRAIN-label-rotated control: 2,288 / 72,130; **−229.003 permille [−232.359, −225.647]**
- absent-section identity: 72,130 checks, 0 mismatch
- cross-surface: 72,138 checks, 0 mismatch
- witness replay: 64 / 64, 0 failure
- served / abstained / declined: 71,488 / 43 / 599

The staged decision contract behaved as predeclared: the 6,000-position sample was `INCONCLUSIVE`; the nested 18,000-position extension was `PROCEED`; only then did the full census run.

### Bound identities

- graph: `blake3:ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`
- deployed-quality report: `blake3:88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`
- generation: `blake3:6a0743691591c553767b25140bdf507de29be7036a968f437ff1d119b0e68f2a`
- sections-absent control: `blake3:83d57b47cd22ac618e3c3c48806f78fa55b3a16af8c6688fe2ee129a90692d8b`
- rotated-label control: `blake3:c2dac66217c85306f9eff8a7e1f6125bed33c317405aaabb9498cd84a9c1f871`
- hardened release-manifest raw BLAKE3: `c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`
- hardened release-manifest SHA-256: `7572e07a1e3722f3ffc0ea749a67b4ac162221de79b5b4b8a315f4e4e6570fde`
- manifest-bound TLA store: `blake3:c1749e62077758c4a098e2a02150b5455e1ca3c02c60b87e6d45fcbb9e2b4404`
- graph/evaluator revision: `74ced4d12a84a176d73665106f88d0aab9407453`
- hardened verifier revision: `f901cd97577da3117fd52c9b1c6dcf075cc4d3a2`

Fresh 4-worker and 8-worker clean emissions reproduced the graph, controls, 6k/full reports, raw parity, raw witness, source/store/tokenizer inputs, and hardened release manifest byte-for-byte. Strict production startup through the ordinary loader admitted the exact bundle from an empty model store.

### Acceptance evidence

Local: full workspace tests with no-fail-fast; warning-deny clippy; format; graph-format no-std/alloc; graph-runtime no-default; wasm32; claim wording; 33-ID conformance; allocation and P-4 scans; non-vacuous κ 3 / 3. Independent final review found no remaining admission-boundary finding.

Protected queue: full nextest, Cucumber BDD, doc tests, no-std, deterministic rebuild, κ, Gate C trend, all-features dormant tests, WebAssembly packaging, nightly fuzz, RustSec/cargo-deny, conformance register, and inference audit all passed.

The repository BDD result was 124 / 124, but the live-teacher parity fixtures were absent and those fixture-gated scenarios vacuously skipped. Therefore live-teacher parity is **NOT_RUN / not established**, not PASS; #932 remains the open, unassigned downstream task.

This outcome raises the production-admitted same-position baseline above its TLA comparator and establishes the bounded RF-31 lane effect. It does **not** create a universal 30% floor or establish instruction following, reasoning, factuality, semantic abstention, free-running coherence, or cross-model parity.

Canonical record: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/normative_r4g1_quality_933.md
PR: https://github.com/UOR-Foundation/uor-r4/pull/936


---
_Comment/review ids in this file: 5401806082, 5403499657, 5404289981, 5406307619_

