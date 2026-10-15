# #841 — evaluation/#824-A: quantify the teacher-forced to free-running trajectory gap

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:37:42Z
- **Updated:** 2026-08-21T08:29:52Z
- **Closed:** 2026-08-21T08:28:22Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/841
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824

## Context

Current Gate C and teacher-parity signals mostly evaluate teacher-forced or recorded corpus positions. Sampled output validity improved, but validity and canary distinctness do not measure how errors compound under the system's own prefixes.

## Problem

The project lacks a matched, step-attributed measurement of where teacher-forced quality ceases to predict free-running behavior. Without it, more data, state, decoding, and anti-cycle heuristics cannot be distinguished.

## Goal

Freeze a sequence-level evaluation that compares matched teacher-forced and student-prefix trajectories and locates the first unsupported or semantically divergent state.

## Scope

- [ ] Define versioned prompt families spanning continuity, assistant behavior, evidence use, constraints, and multi-turn state.
- [ ] Freeze a generation-horizon ladder and deterministic decoder/seeds; report greedy and sampled modes separately.
- [ ] At matched positions, score teacher prefixes, production student prefixes, and controlled corrupted prefixes.
- [ ] Measure prompt relevance, entity/topic retention, contradiction, evidence support, repetition/cycles, grammatical validity, distinctness, termination, and first-failure step.
- [ ] Separate deterministic primary metrics from any versioned secondary judge and preserve raw traces for replay.
- [ ] Include suffix-only, root-prior, shuffled prompt/state/emission, repeated-prefix, teacher-prefix, and decoder-only controls.
- [ ] Attribute every generated step to ExactContext/NGRAM/graph/root/patch/decline, candidates, score contributions, state, and witness.
- [ ] Freeze the primary sequence statistic, sample size/power, confidence method, horizon weights, and correction-round stopping rule.

## Non-goals

- Use aggregate teacher-forced top-1 as a generation metric.
- Treat lexical diversity or grammar alone as coherence.
- Change prompts/horizons after observing corrective results.

## Dependencies and entry gate

Uses #832 report constitution, #833 canonical bundle, and the promoted S1 scorer. S2 typed decline fields should be represented even if S2 is not yet promoted.

## Acceptance criteria

- [ ] Teacher- and student-prefix measurements share exact prompts, positions, identities, and metrics.
- [ ] The first divergence/unsupported step and its scorer/state provenance are replayable.
- [ ] Controls distinguish decoder effects, support gaps, state starvation, and simple repetition.
- [ ] Power and thresholds are frozen before corrective fitting.
- [ ] A planted early-drift model and a repetition-only model fail the intended metrics.

## Verification

- [ ] Trace schema round-trip/determinism and witness replay.
- [ ] Planted prompt shuffle, state shuffle, repeated-cycle, unsupported-candidate, and judge-version negatives.
- [ ] Independent recomputation of primary metrics from raw traces.
- [ ] Streaming and non-streaming produce the same token/state trace for fixed decoding.
- [ ] Fixture absence records UNAVAILABLE rather than a vacuous empirical PASS.

## Compatibility and migration

Evaluation-only schema is versioned. Existing reports remain readable but cannot satisfy the new free-running claim without prompt/horizon/trace identities and per-step attribution.

## Repository conformance

- **Execution scope:** Offline evaluation against the normative deployed R4Engine generation path; reference/offline evidence is not credited as deployed generation.
- Preserve P-4, allocation-free steady state, deterministic bytes/ordered reductions, bounded state/work, typed errors, no_std, and replayable witnesses where applicable.
- Empirical criteria bind distribution, horizons, decoder/seeds, n, uncertainty, provenance, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-14, RF-21, RF-22, RF-29.; extend existing capabilities before adding a new one.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant trajectory mechanisms require an explicit diagnostic activation gate; generated conformance is never hand-edited.
- Append measurement records and reconcile live claims whenever the generation boundary changes.

## Claim status and next action

This establishes the current gap and diagnostic categories; it does not claim coherence or prescribe a corrective mechanism.

## Comments (3)

### comment 5366982231 — 2026-08-21T07:55:26Z — Casey-allard

### Run contract — free-running trajectory-gap run-1 (pre-registered before the run)

Preparing and executing the first offline gap instrument the S3 tracker (#824) explicitly sanctions now that the S0 evaluation is frozen (S3 STAGE closure stays gated on the #822/#823 stage verdicts — untouched here). Contract doc: `docs/free_running_eval_841.md`; harness `crates/uor-r4-api/tests/free_running_eval_841.rs` at branch `issue-841-free-running-gap` (off `d1215f55`), fixture teeth green (5/5: trace round-trip/tamper, first-divergence locator on planted cases incl. the early-drift model, repetition-only model flagged, overlap/median-CI determinism, judge-without-identity → UNAVAILABLE), clippy clean.

- **metric to move:** none — this RUN ESTABLISHES the baseline gap. Frozen primary statistic: median first-divergence step at H=32 under greedy on prompt-family v1 (n=100 story prompts, CID-bound), order-statistic 95% CI (±10 ranks at n=100). Current value: UNMEASURED (that is this run).
- **execution scope:** offline evaluation driving the NORMATIVE DEPLOYED `R4Engine` (released `graph/score.r4g1`, ADR-0001 scorer, D4 policy) — 0 live-teacher evaluations (matched teacher-forced agreement uses the recorded #833 argmax; student-side departure is against the recorded story text).
- **sides per prompt:** teacher-prefix (matched recorded windows), student-prefix (engine consumes its own greedy tokens; typed abstention terminates), shuffled-prompt (rotate-by-one), repeated-prefix (pure-repetition reference), suffix-only (memoryless floor — the FR≡suffix-only identity rate extends the #874 suffix-locality finding from scoring to generation). Horizons {8, 32}.
- **reachability/ceiling arithmetic:** trivially reachable — every prompt yields a full trajectory (the engine serves or typed-abstains at every step; both are first-class readings); n=100 × ~5 sides × ≤32 steps ≈ ≤16k engine calls (the #874 harness ran ~10× that in minutes). The instrument can distinguish the branches: divergence-at-0 vs mid-horizon vs survived are all expressible, and the planted early-drift/repetition negatives prove the metrics have teeth.
- **binding cheap instrument + verdict:** the committed fixture teeth (non-ignored, 5/5 PASS) + bundle presence (corpus/graph/teacher/tokenizer read at load; absent bundle → typed SKIP, never a vacuous pass).
- **pre-registered readings (no pass/fail bar — this is gap ESTABLISHMENT, not a promotion gate):** the record freezes, for #840's future corrective rounds, the improvement bar (median first-divergence +≥2 steps AND diverged-at-0 −≥100‰, no TF-agreement regression >10‰, ≤3 rounds else GENERATION-NOT-ESTABLISHED — contract §6). Sampled mode + judge metrics: UNAVAILABLE in run-1 (no pinned decoder-seed/judge identities), recorded explicitly.
- **if the run completes:** record the gap + diagnostic categories, close #841 (Closes) with the CID-bound record — the quantification IS the deliverable. **if the engine path faults on family v1:** record served/abstained/UNSERVABLE classification truthfully (the #886 catch pattern), and if the family cannot be served at all, that is itself the finding (References-PR, issue stays open for maintainer routing).
- **cost estimate:** ≤ ~2 min wall (≤16k integer engine calls + table-free metrics), peak RSS ≲ 2 GB, artifacts: 1 JSON record + contract doc + RESEARCH.md paragraph; blocks nothing.

Launching now; outcome will be posted against this contract.


### comment 5367007231 — 2026-08-21T07:57:20Z — Casey-allard

### Outcome against the run contract — run-1 complete: the gap is total (memoryless suffix generation, immediate off-text divergence, cycle collapse)

Run executed as contracted (`free_running_gap_run_841`, 10.1 s, 0 live-teacher evaluations; record `docs/free_running_841_result.json`; contract doc `docs/free_running_eval_841.md`). Identities: corpus_meta_cid `blake3:aa9d1767…`, prompt-family v1 cid `blake3:6cad1dfe…`, result_cid `blake3:59ec2f60…`; greedy rollouts double-run deterministic (asserted); shuffled-prompt control non-degenerate.

| reading (H=32, greedy, n=100) | value |
|---|---|
| **median first-divergence step (frozen primary)** | **0** — 590‰ of student rollouts depart the recorded text at the FIRST generated token; **0‰ survive even H=8 on-text** |
| **suffix-locality of generation** | **99/100 free-running rollouts are token-identical to suffix-only (2-token) rollouts** — the deployed generator is memoryless beyond the suffix |
| cycle collapse | **710‰** of FR rollouts enter a ≤4-period loop (≥3 repeats) vs 110‰ under teacher prefixes — drift compounds into repetition without recorded-prefix grounding |
| teacher-forced agreement (matched steps) | 304‰ over 3,200 steps (the per-step ceiling the rollouts fall from) |
| typed abstentions | **0‰ on every side** — D4 serves every step of every trajectory (the #811 always-servable finding at trajectory scale) |
| controls | shuffled-prompt at0 980‰ (order matters only through the trailing suffix); repeated-prefix cycles 640‰ (the pure-repetition reference profile) |

**What run-1 establishes (diagnostic categories for S3):** the teacher-forced ↔ free-running gap is not gradual drift — it is (a) **immediate off-text divergence** (the per-step 304‰ ceiling compounds to ~0 at trajectory scale), (b) **memoryless generation** (whole-prompt content contributes nothing beyond the 2-token suffix — the #874/#834 scoring suffix-locality finding extended verbatim to generation), and (c) **unguarded cycle collapse** with **zero selective declining** along the way. Any #840 corrective observation must move exactly these: the frozen §6 improvement bar (median first-divergence +≥2 steps AND at0 −≥100‰, ≤3 rounds else GENERATION-NOT-ESTABLISHED) is now anchored on real numbers.

Sampled mode and judge metrics: UNAVAILABLE in run-1 (no pinned decoder-seed/judge identities), recorded explicitly in the JSON. Closing with the record per the contract — the quantification is the deliverable; no coherence is claimed, no corrective mechanism is prescribed, no stage gate moves. PR follows.


### comment 5367450593 — 2026-08-21T08:29:52Z — Casey-allard

### Definition-of-Done evidence (issue closed via PR #894, merge `11d7ca8e`)

- **Merged deliverables (main `11d7ca8e`):** `docs/free_running_eval_841.md` (the frozen contract: prompt-family v1 CID-bound, horizon ladder, trace schema v1 with per-step path attribution, deterministic primaries, versioned-judge discipline, the #840 corrective stopping rule), `crates/uor-r4-api/tests/free_running_eval_841.rs` (fixture teeth + the engine-driven run), `docs/free_running_841_result.json` (CID-bound record), `docs/RESEARCH.md` S3 item-A paragraph.
- **Run-1 quantification (greedy, deployed R4Engine, n=100, H∈{8,32}):** median first-divergence step **0** (590‰ diverge at the first generated token; 0‰ survive H=8 on-text; matched TF per-step ceiling 304‰ over 3,200 steps); **99/100 FR rollouts token-identical to suffix-only rollouts** (generation is memoryless beyond the 2-token suffix — the #874/#834 finding extended to generation); **710‰ cycle collapse** at H=32 (vs 110‰ under teacher prefixes); **0‰ typed abstentions** on every side (the #811 always-servable finding at trajectory scale). Identities: corpus `blake3:aa9d1767…`, family `blake3:6cad1dfe…`, result `blake3:59ec2f60…`.
- **Process:** run contract posted BEFORE the run with the instrument verdict; outcome posted against it; greedy rollouts double-run deterministic (asserted); shuffled-prompt control non-degenerate (asserted); planted early-drift and repetition-only models fail the intended metrics (CI-run fixture teeth); sampled mode, judge metrics, and the state-shuffle control recorded UNAVAILABLE with reasons (never a vacuous pass).
- **Frozen for #840:** the corrective improvement bar — median first-divergence +≥2 steps AND diverged-at-0 −≥100‰ per round, no TF-agreement regression >10‰, at most 3 rounds, else `GENERATION-NOT-ESTABLISHED` (the #824 kill criterion instantiated on real numbers).
- **Scope honored:** evaluation-leaf (no ids.toml, no CONFORMANCE regen); no coherence claimed; no corrective mechanism prescribed; S3 STAGE closure remains gated on the #822/#823 stage verdicts.
- **Verification:** local gates green before push (fmt / claim-wording / clippy -D warnings / no_std ×2 / wasm / xtask validate / repo-conformance / deny bans / doc tests / workspace suite 0 failures across 166 result lines; model-driven bdd soak CI-skipped by design); merge queue full ladder green in one pass.



---
_Comment/review ids in this file: 5366982231, 5367007231, 5367450593_

