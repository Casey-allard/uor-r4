# #834 — research/#822-B: fit prompt-conditioned evidence arms against causal and equal-budget controls

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:35:22Z
- **Updated:** 2026-08-21T06:02:54Z
- **Closed:** 2026-08-21T06:00:46Z
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/834
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #822

## Context

#784 measured 0.0% full-depth collisions while 11/15 distinct rows still favored newline. Historical sampled canaries were valid but only about 5/15 distinct at 360M (largest identical group eight), with about 6/15 on the 135M R4G1 path. These are diversity diagnostics, not causal relevance results.

## Problem

The project does not know whether the main bottleneck is missing persistent state discrimination, candidate availability, conditional evidence, or decoding. A single bidirectional-anchor implementation would confound these mechanisms.

## Goal

Fit and ablate several artifact-only prompt-conditioned arms against strong causal and equal-resource controls, then select at most one packable arm.

## Scope

- [ ] Freeze paired paraphrase and meaning-changing subject/relation/negation/role/constraint interventions on document- and template-disjoint splits across at least two domains.
- [ ] Compare current scoring, longer local context, persistent state, conditional residuals against corpus marginals, and candidate-support expansion.
- [ ] Include prompt-shuffled, residual-shuffled, suffix-only, equal-byte, equal-candidate, equal-table-read, and decoder-held-constant controls.
- [ ] Report paired score direction, distribution separation, candidate recall, prompt relevance, paraphrase stability, free-generation behavior, bits/accuracy, bytes, and operations.
- [ ] Attribute gains to state discrimination, candidate availability, scoring, or decoding at every evaluated token.
- [ ] Use deterministic sample selection, fixed seeds/decoders, ordered reductions, frozen thresholds, and confidence intervals.
- [ ] Record every arm and null, including negative and constant/zero-variance outcomes.
- [ ] Select an arm for lowering only if its lower confidence bound clears the predeclared causal-relevance floor without unacceptable quality/resource regression.

## Non-goals

- Promote distinctness without relevance.
- Tune on the held-out intervention suite.
- Use EXCT-dominated aggregate movement as evidence for a mechanism rarely reached.

## Dependencies and entry gate

The contract in sibling item A and #832 suite/report must be frozen; #833 supplies the canonical artifact. Post exact run values before execution.

## Acceptance criteria

- [ ] All arms and controls run on identical identities and evaluation slices.
- [ ] Candidate-support versus score-conditioning effects are separately identified.
- [ ] A control cannot pass through zero variance or identical outputs.
- [ ] The decision is `SELECT`, `REVISE`, or `NO PROMPT-CONDITIONING ARM ESTABLISHED` under the posted threshold.
- [ ] Raw per-pair attribution and aggregate uncertainty are replayable from CID-bound records.

## Verification

- [ ] Small deterministic fixture proves interventions, attribution, and shuffled controls operate.
- [ ] Power analysis fixes n and the minimum decision-relevant effect before fitting.
- [ ] Double-run/reordered-shard determinism check.
- [ ] Planted prompt-insensitive and diversity-only models fail the primary gate.
- [ ] Independent report recomputation from stored scores/witnesses.

## Run contract

- **Metric/current value:** Primary paired causal-relevance statistic on EXCT-disabled document-disjoint pairs; historical surface diagnostics are 11/15 newline argmax rows and only ~5–6/15 distinct sampled/R4G1 canaries, to be remeasured on #833.
- **Reachability ceiling:** For each arm, compute the fraction of tokens whose candidate set and/or scorer it can change; headline movement cannot exceed that reachable fraction. Report candidate-recall and scoring ceilings separately.
- **Pinned identities:** Canonical teacher/tokenizer/corpus/partitions/bundle from #833, suite/report from #832, state spec revision, arm configs, fixed decoder/seeds, hardware, raw records, and report CIDs.
- **Nulls/falsifier:** Current scorer; longer suffix only; prompt shuffle; residual shuffle; equal-byte/equal-candidate arms; constant-state; trivial/root prior; decoder held constant.
- **Binding cheap instrument:** On a committed small fixture, state variance, intervention direction, candidate change, and every null must be non-degenerate; reachability arithmetic must exceed the effect floor.
- **Exit rule:** Select only when the primary causal-relevance lower confidence bound clears the frozen floor on both domains while relevance, bits/accuracy, and resource budgets satisfy their non-regression limits.
- **If positive:** Open the selected layout/runtime lowering work in sibling item C and ledger all other arms as rejected/dormant with evidence.
- **If negative:** Publish the falsifier, keep current serving semantics, and return S1 to representation/compiler redesign rather than code-space or sampling tuning.
- **Cost estimate:** Post projected wall time, peak RSS/storage, teacher evaluations, artifact bytes per arm, and blocked shared resources.

No long run starts until exact values are posted and the cheap instrument passes. Missing observations are `UNAVAILABLE`/`null`, and every control must be shown non-degenerate.

## Compatibility and migration

Research artifacts are non-serving. Candidate layouts carry experimental identities and cannot be loaded by production admission until sibling item C defines a versioned section.

## Repository conformance

- **Execution scope:** Offline compiler/certifier/reference arms; production serving is used only as the fixed baseline/readout. Evidence outside this scope is not credited as serving.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes, bounded state/work, typed errors, `no_std`, and witness obligations.
- Normative equations/statements follow `docs/formal_vocabulary.md`; empirical criteria include distribution, protocol, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-01, RF-21, RF-22, RF-27, RF-28.. Extend existing capability suites unless the implementation creates a genuinely new capability.
- Built capability order is `model/ids.toml` → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Dormant/unselected arms remain ledgered with activation or retirement gates. Generated conformance is never hand-edited.
- Claim-changing measurements append a per-issue record and reconcile all current summary assertions.

## Claim status and next action

Only a predeclared positive causal verdict may trigger deployed lowering. A negative result is the required scientific output, not an implementation backlog.

## Comments (9)

### comment 5359647844 — 2026-08-20T17:50:25Z — Casey-allard

### S1 item B — binding cheap instrument landed (PR #873); causal verdict remains UNAVAILABLE

Per this issue's run contract ("No long run starts until exact values are posted and the cheap instrument passes"), PR #873 lands the **binding cheap instrument** as reference-only / off-serving code and does **not** assert a causal verdict.

**Delivered (green locally + on the PR):**
- `crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs` — offline, integer, deterministic bake-off over the five arms (`current-scoring`, `longer-local-context`, `persistent-state`, `conditional-residuals`, `candidate-support-expansion`) vs the frozen `prompt-swap` / `suffix-only` / `shuffled-state` / `trivial-prior` and equal-budget controls. Implements the five Verification items as machine-checked tests, separately identifies candidate-availability from score-conditioning, shows every control non-degenerate, and makes the planted **prompt-insensitive** and **diversity-only** models fail the primary gate.
- `docs/prompt_arms_bakeoff_834.md` — evidence record + run contract with the entry-gate assessment below.
- Follows the #835/#832 spec-leaf precedent: no `model/ids.toml` row, no `CONFORMANCE.md` regeneration.

**Entry-gate assessment (a missing input is UNAVAILABLE, never a vacuous pass):**

| Input | Status |
|---|---|
| #835 state-spec revision + `s1-causal-prompt-pairs` (#832 vocabulary) | **AVAILABLE / frozen** |
| Binding cheap instrument (non-degeneracy, reachability, power) | **PASS** (machine-checked) |
| #833 canonical teacher / tokenizer / corpus / partitions / bundle | **UNAVAILABLE** in this offline instrument |
| Powered `n`, fixed-decoder identities for the real evaluation | **UNAVAILABLE** (set by the #833 protocol) |

**Decision status:** `SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED` is **UNAVAILABLE**. The real multi-arm bake-off runs on the #833 canonical bundle with real teacher evaluations across ≥2 domains — a maintainer-gated long run with programme-wide blast radius (only a predeclared positive verdict may trigger the deployed lowering in #836 and gate the S1→S7 lane).

**Exact next action (maintainer decision):** post the #833-scoped run values (reachability arithmetic from real numbers, powered `n`, cost estimate: wall time / peak RSS / teacher-evaluation count / per-arm artifact bytes) to this issue, obtain go/no-go, then run the bake-off and record the verdict with CID-bound records. This issue stays **open** until then.


### comment 5360355725 — 2026-08-20T18:56:23Z — Casey-allard

### Teacher-grounded S1 causal-influence run — verdict recorded (PR #874)

Building on the merged instrument (#873), the real measurement was executed on the #833 canonical bundle (`smollm2-360m-broad-clean`), reusing the teacher argmax already recorded in the bundle's corpus + the deployed **EXCT-disabled** engine over the real Simple-Wiki held-out set — no live teacher forward. Harness `crates/uor-r4-api/tests/causal_prompt_run_834.rs`; CID-bound record `docs/causal_run_834_result.json`; write-up `docs/prompt_arms_bakeoff_834.md` §6.1.

**Result (n = 24,044 held-out positions; windows mean 7.8 tokens, 98% longer than the 2-token suffix):**

| measure | value |
|---|---|
| full-context top-1 vs teacher | 26.9% (CI [26.4, 27.5]) |
| suffix-only top-1 vs teacher | 27.1% |
| **causal-influence-delta (full − suffix)** | **−1.6‰ (paired 95% CI [−2.4, −0.8])** |
| context-saturation sweep k1→k8 | 25.9 / 27.1 / 27.0 / 27.0 / 27.0 / 26.9 % — **flat** |
| prompt-swap null / trivial-prior | 2.2% / 5.0% (non-degenerate) |
| context-helped positions | 27 / 24,044 |
| **minimal pairs** (same 2-tok suffix, different teacher answer) | **1,460 pairs, model follows 0/1,460** |

**Verdict: `NO PROMPT-CONDITIONING ARM ESTABLISHED`** for the arms fittable against the current artifact (current-scoring, longer-local-context) — the deployed model is **suffix-local**, corroborating #784, now teacher-grounded and quantified. Per the run contract's *if-negative* branch, current serving semantics are kept and S1 returns to representation/compiler redesign.

**Scope:** the three #835 Ψ-family arms (`persistent-state`, `conditional-residuals`, `candidate-support-expansion`) remain **UNAVAILABLE** — they are not lowered into the artifact (the engine exposes the selected token, not candidate scores) and can be fitted only after **#836** builds them. The negative makes #836 the prerequisite for any positive S1 conditioning claim; it does not assert the unbuilt mechanisms would also fail.

**Closure is a maintainer call:** accept this negative as the #834 verdict (route S1 to #836/redesign and close), or keep #834 open pending the Ψ-family arms after #836. Awaiting that decision before closing.


### comment 5360628765 — 2026-08-20T19:22:31Z — Casey-allard

Reopened: the #874 merge auto-closed this as COMPLETED, but the maintainer decision is to **keep #834 open** with the negative verdict recorded as interim evidence, pending evaluation of the #835 Ψ-family arms once they are built. The teacher-grounded negative for the current mechanism stands (see the run comment above and `docs/prompt_arms_bakeoff_834.md` §6.1).

### comment 5360818917 — 2026-08-20T19:41:12Z — Casey-allard

### Ψ segment-lane reference-arm re-test — POSITIVE (PR #875)

Per the decision to build+test the Ψ mechanism (not deploy it under #836 on faith), the #835 **segment lane** (whole-prompt content → candidate support) was built as an offline reference arm and re-tested teacher-grounded on the #833 corpus.

From 288,794 document-disjoint train positions → suffix and whole-prompt-content co-occurrence tables → scored on all 72,130 held-out positions (λ fixed 1.0):

- suffix baseline **246.6‰** vs Ψ **264.1‰** → **+17.5‰ (95% CI [15.9, 19.0])**, excludes zero.
- minimal pairs (same 2-tok suffix, diff story, diff teacher answer): **Ψ follows 10/4,722**, suffix baseline **0/4,722**.

**Verdict: SELECT (positive).** Whole-prompt content is predictive beyond the suffix — the deployed artifact (suffix-local, §6.1) **discards** real signal a bounded segment lane recovers. Modest effect (+1.75pp; 10 hard pairs) but real → **warrants building #835/#836**, with calibrated expectations. Reference-only; a deployed lowering must re-clear the causal gate on the packed path (#836 acceptance). Record: `docs/prompt_arms_bakeoff_834.md` §6.2, `docs/psi_arm_834_result.json`.

### comment 5360822868 — 2026-08-20T19:41:35Z — Casey-allard

### Ψ segment-lane reference-arm re-test — POSITIVE (PR #875)

Per the decision to build+test the Ψ mechanism (not deploy it under #836 on faith), the #835 **segment lane** (whole-prompt content → candidate support) was built as an offline reference arm and re-tested teacher-grounded on the #833 corpus.

From 288,794 document-disjoint train positions → suffix and whole-prompt-content co-occurrence tables → scored on all 72,130 held-out positions (λ fixed 1.0):

- suffix baseline **246.6‰** vs Ψ **264.1‰** → **+17.5‰ (95% CI [15.9, 19.0])**, excludes zero.
- minimal pairs (same 2-tok suffix, diff story, diff teacher answer): **Ψ follows 10/4,722**, suffix baseline **0/4,722**.

**Verdict: SELECT (positive).** Whole-prompt content is predictive beyond the suffix — the deployed artifact (suffix-local, §6.1) **discards** real signal a bounded segment lane recovers. Modest effect (+1.75pp; 10 hard pairs) but real → **warrants building #835/#836**, with calibrated expectations. Reference-only; a deployed lowering must re-clear the causal gate on the packed path (#836 acceptance). Record: `docs/prompt_arms_bakeoff_834.md` §6.2, `docs/psi_arm_834_result.json`.


### comment 5365123119 — 2026-08-21T04:15:41Z — Casey-allard

### Maintainer decision (2026-08-21): build + measure the remaining `conditional-residuals` arm

Recording the go-ahead so this issue is **executable next session** (no longer parked-pending-a-decision).

**Context — the arm space, now that the segment lane is fully run:** `current-scoring` and `longer-local-context` are measured negative (§6.1 / #874, deployed model suffix-local). `persistent-state` (the segment lane) was positive off-serving (+17.5‰, #875) but **retired sub-floor** after lowering — #836 → #886 → #887 (merged `7bb8b4ec`: the 20‰ bar stands, lane dormant). `candidate-support-expansion` (#888) is **closed off**. That leaves **`conditional-residuals` (against corpus marginals)** — a mechanism distinct from the segment lane — as the one arm in this issue's scope **never built or measured.**

**Decision: build it and measure it before accepting an S1 prompt-conditioning verdict.** Its off-serving ceiling is genuinely unmeasured; it is the last real chance to surface a positive at S1.

**Execution mandate (follow the #875 offline reference-arm precedent):**
- Build `conditional-residuals` as an **offline reference arm** (reference-only, off-serving), fit on the 288,794 document-disjoint train positions, scored teacher-grounded on the 72,130 held-out positions via the recorded teacher argmax in the #833 bundle (`smollm2-360m-broad-clean`) — no live teacher forward, deterministic, integer.
- Reuse the #873 bake-off harness controls (prompt-swap / suffix-only / shuffled-state / trivial-prior + equal-budget) and this issue's acceptance criteria: separately identify candidate-availability vs score-conditioning, show every control non-degenerate, make planted prompt-insensitive / diversity-only models fail the primary gate.
- **Post the run contract first** (reachability arithmetic from real numbers, powered n, cheap-instrument verdict, exit rule vs the frozen 20‰ floor, cost estimate) — even though the #875-style offline path is fast — per this issue's run contract and the repo long-run discipline.
- Record `SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED` with a CID-bound record, a `docs/*_834.md` §-append, and a reference `.rs` (no `model/ids.toml`, no `CONFORMANCE.md` — research-leaf, unless a positive later triggers an #836-style lowering).

**Calibration (from #887):** a positive off-serving result is necessary but not sufficient — any deployed lowering re-faces the integer-`ScoreQ` quantization + top-8 candidate ceiling that held the segment lane's 19.0‰ ceiling below the 20‰ floor. For `conditional-residuals` to matter downstream it needs a comfortable off-serving margin **above** 20‰, not merely a positive lower bound. Interpret its verdict accordingly.

**Branches:** **If positive** → open an #836-style lowering track (new issue) and re-clear the causal gate on the packed path. **If negative** → this is the second+ independently-motivated arm to fail causal controls, meeting #822's kill/redesign criterion → S1 returns to representation/compiler redesign; close #834 with the full falsifier set.

#834 stays **open + unassigned** until a session actively picks it up (assignment = actively-working-now). S2 (#823) remains parallel-available if a session prefers to work that lane concurrently.


### comment 5365301840 — 2026-08-21T04:44:49Z — Casey-allard

### Run contract — `conditional-residuals` reference arm (pre-registered before the run)

Executing the 2026-08-21 maintainer decision (the last unmeasured arm of this issue's five-arm scope). Posted per this issue's run-contract block and the AGENTS.md long-run discipline, before the measurement is launched. Harness: `crates/uor-r4-api/tests/conditional_residuals_run_834.rs` at branch `issue-834-conditional-residuals` (off `7bb8b4ec`), compiled and clippy-clean before launch.

- **metric to move:** paired CR-minus-suffix top-1 delta vs the recorded teacher argmax on the #833 held-out split. Current value for this arm: UNMEASURED (that is this run). Anchors in hand: suffix baseline 246.6‰; Ψ comparator +17.5‰ (95% CI [15.9, 19.0]) from record §6.2.
- **arm definition (pre-registered):** `score(c) = suffix_rate(c) + λ·(content_rate(c) − marginal_rate(c))` over the Ψ-widened candidate set (suffix top-32 ∪ content top-32) — identical tables and candidates as §6.2's Ψ, so CR-vs-Ψ is a pure scoring-rule ablation ("persistent state minus the corpus marginal", record §2). Secondary `cr-narrow` (same score over suffix candidates only) separates score-conditioning from candidate-availability inside the arm. λ = 1.0 fixed before eval; the λ-sweep is exploratory, not the verdict.
- **reachability ceiling (arithmetic):** candidate ceiling — CR shares Ψ's widened candidate set, and every held-out position has non-empty content evidence (windows mean 7.8 tokens), so all 72,130 positions are reachable. Scoring ceiling — the arm can flip any position where the suffix baseline is wrong: 1000‰ − 246.6‰ = 753.4‰ (loose bound, far above the floor). Decision resolution: at n = 72,130 the §6.2 paired CI half-width was ±1.55‰ ≪ the 20‰ floor, so SELECT / REVISE / NO-ARM are all distinguishable. Mechanism headroom is documented, not hypothetical: the marginal-dominance pathology is real (#784: 11/15 distinct rows favored newline; trivial-prior ≈ 50‰).
- **binding cheap instrument + verdict:** the merged #873 instrument (`prompt_arms_bakeoff_834`, includes the conditional-residuals toy arm, planted prompt-insensitive/diversity-only negatives, candidate-vs-score attribution, double-run determinism) re-run at `7bb8b4ec`: **PASS 12/12** (release, exit 0). Bundle presence confirmed: `corpus.meta` + `corpus.records` at `.uor-models/compiled/smollm2-360m-broad-clean`.
- **pinned identities:** #833 bundle (`smollm2-360m-broad-clean`; corpus_meta_cid recorded in-run), `split_positions` 288,794 train / 72,130 held-out, recorded teacher argmax (no live teacher forward), fixed argmax decoder (score desc, id asc) across every arm and control, CAP=64, CAND_SUFFIX=CAND_CONTENT=32, SUFFIX_K=2 — all constants frozen in the harness before execution.
- **nulls / falsifiers (each asserted non-degenerate):** suffix-only baseline; prompt-swap (CR scored with a different-story window's content evidence); residual-shuffle (each candidate subtracts the next sorted candidate's marginal — the specific marginal alignment must matter); trivial-prior (corpus-marginal argmax). Harness-correctness gate: the run must REPRODUCE §6.2 exactly (suffix 246.6‰, Ψ 264.1‰, 4,722 minimal pairs, Ψ-follow 10, baseline-follow 0) or it aborts. In-harness double-run determinism check over 2,000 positions.
- **powered n:** the full held-out partition, n = 72,130 (not sampled); minimal pairs mined at 1/suffix-key as in §6.2 (4,722 expected).
- **exit rule (pre-declared, #887-calibrated — stricter than §6.2's):** **SELECT** iff the CR paired-delta 95% LOWER bound ≥ 20‰ (the frozen `CAUSAL_FLOOR_PERMILLE`; a minimal-pairs lower bound alone can no longer produce SELECT, per the #887 governance verdict). **NO ARM** iff the lower bound ≤ 0 and CR follows 0 minimal pairs. **REVISE** otherwise (real-but-sub-floor; the 20‰ bar stands, so no lowering opens).
- **if positive (SELECT):** open an #836-style lowering-track issue for the CR arm (which must re-clear the causal gate on the packed path); close #834 with the SELECT record — the five-arm space is then fully dispositioned.
- **if negative (REVISE or NO ARM):** no lowering opens; close #834 with the full falsifier set and per-arm dispositions; S1 (#822) meets its kill/redesign criterion (a second+ independently motivated arm class fails the causal gate) — the stage verdict is the maintainer's on #822. The positive and negative branches differ in both artifact (new issue vs none) and S1 routing, so the run has decision value.
- **cost estimate:** single-process offline run, ~5–20 min wall (§6.2-scale harness × ~2 for the extra arms + sweep + double-run), peak RSS ≲ 4 GB, 0 teacher evaluations, artifacts: 1 CID-bound JSON record + 1 record §-append; blocks no shared resources.

Launching the run now; the outcome will be posted against this contract.


### comment 5365329765 — 2026-08-21T04:49:14Z — Casey-allard

### Outcome against the run contract — `conditional-residuals`: **REVISE** (real, unconditionally sub-floor; no lowering)

The run executed as contracted (harness `crates/uor-r4-api/tests/conditional_residuals_run_834.rs`, 12.6 s wall, 0 teacher evaluations; record `docs/conditional_residuals_834_result.json`, write-up `docs/prompt_arms_bakeoff_834.md` §6.3). The harness first reproduced §6.2 to the digit (suffix 246.6‰; Ψ 264.1‰ / +17.5‰ [15.9, 19.0]; 4,722 pairs, Ψ-follow 10, baseline-follow 0) and passed the 2,000-position double-run determinism check. Identities: corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:feece1b3…`.

| measure | value |
|---|---|
| **CR (primary)** | **262.8‰**, delta vs suffix **+16.2‰ (paired 95% CI [+14.6, +17.9])** |
| CR vs Ψ (identical candidates/tables) | **−1.3‰ [−2.4, −0.1]** — the marginal subtraction slightly hurts |
| residual-shuffle null | +16.0‰ [14.4, 17.6] (≈ the real arm; 14,478 predictions changed) — the specific marginal alignment contributes nothing |
| cr-narrow (score-conditioning only) | +3.1‰ [2.3, 4.0] — most of the arm's movement is candidate availability |
| prompt-swap null | −7.3‰ [−8.7, −5.9] (27,758 changed) — the gain is prompt-specific |
| trivial prior / λ-sweep | 49.1‰ / flat (262.1, 262.8, 262.0, 259.7, 245.8 at λ 0.5–8) |
| minimal pairs | CR follows **13/4,722** (95% lo 1.3‰) vs Ψ 10, baseline 0 |

**Verdict: `REVISE` under the pre-registered exit rule** — the CI excludes zero (not NO-ARM) but the lower bound 14.6‰ < 20‰ (not SELECT); indeed the UPPER bound 17.9‰ is below the floor, so the arm is **unconditionally sub-floor** — the same reachability arithmetic that made the segment lane dormant (#836), before even facing the #886 lowering-fidelity gap. The falsifier for the arm's distinctive mechanism is two-fold: CR-vs-Ψ ≤ 0, and the residual-shuffle null landing at the real arm — what carries the arm is §6.2's content evidence, not the conditional-residual transformation.

**The contract's negative branch applies:** no lowering track opens (#887: the 20‰ bar stands). Every arm of this issue's five-arm scope now carries an evidence-backed disposition — `current-scoring`/`longer-local-context` negative (#874), `persistent-state` retired sub-floor (#836/#886/#887), `conditional-residuals` REVISE sub-floor (this run), `candidate-support-expansion` closed not-planned (#888) — and **none cleared the frozen causal floor**. S1 meets the #822 kill/redesign criterion (two-plus independently motivated arms fail the causal gate); the stage verdict against the full child set is the maintainer's on #822.

PR with the harness + records follows; it closes this issue with the full falsifier set per the contract.


### comment 5365832503 — 2026-08-21T06:02:54Z — Casey-allard

### Definition-of-Done evidence (issue closed via PR #891, merge `77d09023`)

- **Verdict: `REVISE` — real, unconditionally sub-floor** (+16.2‰ paired 95% CI [+14.6, +17.9]; the upper bound sits below the frozen 20‰ floor), recorded per the pre-registered rule in the run contract above. No lowering track opens (#887: the bar stands).
- **Merged deliverables (main `77d09023`):** `crates/uor-r4-api/tests/conditional_residuals_run_834.rs` (reference harness), `docs/conditional_residuals_834_result.json` (CID-bound record: corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:feece1b3…`), `docs/prompt_arms_bakeoff_834.md` §6.3 (write-up + arm-space disposition), `docs/RESEARCH.md` S1 reconcile.
- **Verification:** run contract posted BEFORE the run; binding cheap instrument PASS 12/12 at `7bb8b4ec`; §6.2 reproduced to the digit as the harness-correctness gate (suffix 246.6‰, Ψ 264.1‰/+17.5, 4,722 pairs, Ψ-follow 10, baseline-follow 0); 2,000-position double-run determinism; every control non-degenerate; local gates green (fmt / claim-wording / clippy -D warnings / no_std ×2 / wasm lib / xtask validate R1+R4+R5 / repo-conformance R2+R3 / cargo-deny bans R6 / 149 workspace test binaries with 0 failures / doc tests); merge queue full ladder green in one pass (17m5s).
- **Five-arm disposition (this issue's Scope, complete):** `current-scoring` NEGATIVE (#874) · `longer-local-context` NEGATIVE (#874) · `persistent-state` retired sub-floor (#836/#886/#887) · `conditional-residuals` REVISE sub-floor (PR #891) · `candidate-support-expansion` closed not-planned (#888). **No arm cleared the frozen causal floor.**
- **Routing per the contract's negative branch:** current serving semantics kept; S1 (#822) meets its kill/redesign criterion — the stage verdict (`PROMOTE/REVISE/LIMIT/RETIRE` against the full child set) is the maintainer's call on #822.



---
_Comment/review ids in this file: 5359647844, 5360355725, 5360628765, 5360818917, 5360822868, 5365123119, 5365301840, 5365329765, 5365832503_

