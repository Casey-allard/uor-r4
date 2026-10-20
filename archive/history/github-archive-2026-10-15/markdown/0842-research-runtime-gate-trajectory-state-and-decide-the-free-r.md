# #842 — research/runtime: gate trajectory state and decide the free-running generation claim (item C of #824)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:37:43Z
- **Updated:** 2026-08-22T02:38:29Z
- **Closed:** 2026-08-22T02:36:23Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/842
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824

## Context

Even after corrective student-prefix evidence, remaining errors may come from evidence gaps, candidate support, decoding, or insufficient bounded trajectory state. State is justified only if per-step attribution isolates that cause.

## Problem

Adding topic/entity memory or anti-cycle state without a binding diagnostic can improve surface diversity while leaving semantic coherence unchanged and expanding the trusted runtime unnecessarily.

## Goal

Use the frozen diagnostics to decide whether a minimal trajectory-state mechanism is triggered, implement only the triggered bounded state, and issue the stage's production sequence verdict.

## Scope

- [ ] Define a cheap diagnostic that distinguishes state starvation from evidence/candidate/decoder failure using sibling A/B traces.
- [ ] Post a binary `TRIGGERED`/`NOT TRIGGERED` decision before implementation.
- [ ] If triggered, specify the minimum topic/entity/trajectory lanes, deterministic update/eviction, candidate/score effect, and witness fields.
- [ ] Include an equal-budget compiler-only control and a simple repetition/cycle-control baseline.
- [ ] Lower triggered state to fixed-capacity integer/table operations through the normative scorer; otherwise make no runtime change.
- [ ] Run reference/packed/production differential and planted-state-shuffle tests.
- [ ] Re-run the frozen horizon ladder, teacher-forced metrics, prompt causality, and selective-risk checks.
- [ ] Issue `PROMOTE`, `REVISE-STATE`, `GENERATION-NOT-ESTABLISHED`, or `NOT TRIGGERED` with exact evidence and limitations.

## Non-goals

- Call a repetition penalty semantic state.
- Add state when the diagnostic points to missing candidates/evidence or decoding.
- Continue correction/state iterations beyond the posted budget.

## Dependencies and entry gate

Consumes sibling items A and B. Runtime work additionally requires #831 and the promoted S1 implementation. Public promotion requires the relevant S2 selective-risk verdict.

## Acceptance criteria

- [ ] The activation diagnostic is predeclared, non-vacuous, and independently reproducible.
- [ ] No runtime code lands when the mechanism is NOT TRIGGERED.
- [ ] If implemented, packed state is bounded, deterministic, P-4, allocation-free, witnessable, and old-artifact compatible.
- [ ] The production horizon ladder satisfies the frozen sequence threshold and all non-regression budgets, or the negative claim is published.
- [ ] README/ROADMAP/RESEARCH and conformance scope match the final verdict.

## Verification

- [ ] Diagnostic replay on raw traces plus planted state-starvation/evidence-gap/decoder-gap cases.
- [ ] Reference/packed/production differential and state-shuffle/constant-state negatives.
- [ ] Workspace no-fail-fast, clippy, fmt, no_std, wasm, deterministic rebuild, allocation census, P-4 scan, witness replay.
- [ ] Frozen horizon-ladder run with independent primary-metric and confidence recomputation.
- [ ] Old-artifact, reset/session, streaming, corrupt-state, and capacity-boundary tests.

## Run contract

- **Metric/current value:** Residual primary sequence gap after the best bounded corrective arm, stratified by diagnosed cause and horizon; baseline is fixed by sibling B.
- **Reachability ceiling:** The maximum effect of added state is the fraction of failures classified as state-starvation and reachable by the proposed state contribution; post exact attribution arithmetic.
- **Pinned identities:** Sibling traces/reports, selected corrective artifact, state diagnostic/spec, reference/packed builds, decoder/seeds, production binaries, and final reports.
- **Nulls/falsifier:** No state, equal-budget compiler evidence, repetition-only, state shuffle, constant state, candidate/evidence controls, old artifact.
- **Binding cheap instrument:** Diagnostic confusion fixture and reachability ceiling must pass; reference state must move diagnosed examples and leave non-state examples stable before packed implementation.
- **Exit rule:** Promote only if the production lower confidence bound clears the frozen sequence gate across horizons and all prompt/selective/teacher-forced/resource non-regression limits pass.
- **If positive:** Freeze the state/observation semantics as the S3 baseline and unlock S4 within the measured generation scope.
- **If negative:** Publish GENERATION-NOT-ESTABLISHED or the narrower limiting diagnosis; do not claim general free-running coherence.
- **Cost estimate:** Post implementation/run wall time, bytes/state, operations/token, latency, peak resources, and shared hardware requirements.

Post exact values before execution. The full run requires a passing cheap instrument, non-degenerate controls, and distinct outcome branches; missing results are `UNAVAILABLE`/`null`.

## Compatibility and migration

Triggered state uses a versioned optional section and caller-owned request state; absent sections preserve exact historical behavior. Protocol/session changes require explicit version negotiation and migration notes.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine generation path; diagnostic/reference work is separately scoped; reference/offline evidence is not credited as deployed generation.
- Preserve P-4, allocation-free steady state, deterministic bytes/ordered reductions, bounded state/work, typed errors, no_std, and replayable witnesses where applicable.
- Empirical criteria bind distribution, horizons, decoder/seeds, n, uncertainty, provenance, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-14, RF-21, RF-22, RF-29 and any justified trajectory-state extension.; extend existing capabilities before adding a new one.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant trajectory mechanisms require an explicit diagnostic activation gate; generated conformance is never hand-edited.
- Append measurement records and reconcile live claims whenever the generation boundary changes.
- Actual production request and streaming paths are mandatory; a reference/certifier-only result cannot promote the claim.

## Claim status and next action

The final issue comment is the authoritative S3 generation verdict. Negative or NOT TRIGGERED outcomes are valid closure states.

## Comments (1)

### comment 5377413756 — 2026-08-22T02:38:29Z — Casey-allard

## Definition of Done — #842 (S3 item C): trajectory-state gate

**Disposition: `NOT TRIGGERED` / `GENERATION-NOT-ESTABLISHED`.** Merged via PR #913 → main `abcad4e5d084289f053907284185f3c5908e9237` (2026-08-22T02:36:22Z, single queue pass). This issue auto-closed on merge.

### Verdict
A predeclared, non-vacuous diagnostic classifies every free-running failure of the frozen prompt-family v1 (n=100, greedy, deployed `R4Engine`, teacher-free) by pairing the student-prefix rollout against the teacher-prefix rollout and probing the deployed candidate list. Result (`result_cid blake3:86376394…`; corpus `blake3:aa9d1767…`; graph `blake3:bc2366f1…`):

- **Reproduces #841 exactly:** 304‰ TF, 99/100 suffix-local, student median 0 / diverged-at-0 590‰ / paths 0/3197/3.
- **Failure diagnosis (H=32, n=100):** survived 0 · single-step-at-0 **59** · candidate-gap **16** · rank-limit **25** · **state-starvation 0**.
- **Reachability:** state-addressable 0‰; at0 reachable-drop 0‰ (step-0 invariance) vs the §6 100‰ bar; the teacher-prefix upper bound is itself median 0 / at0 590‰. `ceiling_clears_bar = false`.

Zero of 100 free-running failures are addressable by bounded trajectory state, and even the drift-free reference (teacher-prefix) does not clear the frozen §6 bar. Per the #842 non-goals, no runtime code lands.

### Acceptance criteria
- Diagnostic predeclared, non-vacuous, independently reproducible — **MET** (pure classifier + confusion fixture that reports `TRIGGERED` on a planted clearing shape, i.e. it can fail; CID-bound result reproduces #841).
- No runtime code lands when NOT TRIGGERED — **MET** (test + docs only).
- Production horizon ladder clears the frozen threshold OR the negative is published — **negative published** (GENERATION-NOT-ESTABLISHED).
- README/ROADMAP/RESEARCH + conformance match the verdict — **MET** (RESEARCH.md S3 item C added; conformance unchanged, RF-14/21/22/29).

### Verification
Full local ladder green (fmt; clippy `-D warnings`; `cargo test --workspace --no-fail-fast --offline` — 177 suites, 0 failures; claim-wording; register conformance R1 check-model 31 ids/R2/R3/R4/R5/R6 bans; gnaf) plus the full merge-queue ladder on the speculative merge.

### Evidence artifacts
- `docs/state_trajectory_gate_842.md` — the authoritative S3 verdict record
- `docs/state_trajectory_gate_842_result.json` — CID-bound result (`result_cid blake3:86376394…`)
- `crates/uor-r4-api/tests/state_trajectory_gate_842.rs` — the diagnostic harness (fixture teeth CI-run; heavy run `#[ignore]`/bundle-gated)

### Claim boundary
The S3 free-running generation claim is **not established** at this representation scope. It meets the programme global falsifier (no bounded correction or state reduces the frozen free-running gap → the deployed artifact is a teacher-forced continuation/retrieval system, not a generative engine at this scope) and is consistent with #840. Re-entry requires a representation with cross-step free-running memory under the frozen #841 §6 bar.

**Next (separate action, not this issue):** the S3 tracker #824 records its stage verdict (`PROMOTE`/`REVISE`/`LIMIT`/`RETIRE`) over its three completed children — #841 (gap quantified), #840 (corrective observation not launched), #842 (state not triggered).



---
_Comment/review ids in this file: 5377413756_

