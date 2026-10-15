# #840 — observation/compiler: compile deterministic student-prefix corrections with equal-budget controls (item B of #824)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:37:41Z
- **Updated:** 2026-08-22T01:25:47Z
- **Closed:** 2026-08-22T01:19:52Z
- **Labels:** enhancement, graph-compiler
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/840
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824

## Context

The compiler observes teacher/corpus contexts, but free-running deployment visits prefixes shaped by its own earlier choices. Exposure to those states may be necessary, but a corrective arm must beat ordinary additional teacher-forced evidence under equal resources.

## Problem

Naive rollout correction can leak evaluation prompts, amplify model errors, become nondeterministic, erase natural-corpus evidence, or consume multi-hour runs without a decision-relevant control.

## Goal

Record deterministic student-generated states, obtain pinned offline teacher corrections, compile bounded corrective evidence, and compare it with equal-record/byte teacher-forced and shuffled controls over a predeclared number of rounds.

## Scope

- [ ] Generate exact prefixes with pinned artifact, prompts, state, decoder, seeds, candidates, resolution paths, and witnesses.
- [ ] Query only the pinned offline teacher/compiler path for corrective top-k/logprob or target evidence; no inference-time teacher dependency.
- [ ] Bind teacher, tokenizer, artifact, prompt suite, decoder, trace, correction, shard, and output CIDs.
- [ ] Use resumable deterministic shards, content-addressed sample IDs, ordered merge/reduction, and explicit deduplication.
- [ ] Exclude all held-out evaluation prompts/documents/templates from corrective observation and add leakage checks.
- [ ] Compile a corrective arm from visited states plus an equal-record, equal-byte, and equal-compiler-cost additional teacher-forced arm.
- [ ] Add shuffled-correction, no-correction, and ordinary-more-data controls.
- [ ] Define deterministic weighting/retention so corrections cannot silently erase natural-corpus evidence.
- [ ] Run only the frozen bounded number of correction rounds and report each round's sequence quality, teacher-forced quality, bytes, compile resources, and path mix.

## Non-goals

- Online learning in the deployed runtime.
- Unbounded iterative self-training.
- Use evaluation prompts in observation or select the best round after the frozen stop.

## Dependencies and entry gate

Blocked on sibling item A, the promoted S1 path, and the #833 canonical source chain. S2 need not be promoted for the first experiment but its current verdict must be reported.

## Acceptance criteria

- [ ] Every observed/corrected record replays to the exact prefix/state/witness and pinned teacher evidence.
- [ ] No evaluation leakage is detected and shard/reduction order cannot change artifact bytes.
- [ ] Corrective and teacher-forced controls are equal under all declared budgets.
- [ ] Each round produces comparable sequence, teacher-forced, selective-risk, byte, and cost reports.
- [ ] The result supports `CORRECTION HELPS`, `NO ADVANTAGE OVER MORE DATA`, or `CORRECTION REGRESSES` under the frozen rule.

## Verification

- [ ] Trace/correction replay, CID/tamper, leakage, duplicate, interruption/resume, and reordered-shard tests.
- [ ] Small end-to-end observe→correct→compile→serve fixture.
- [ ] Double-build byte comparison for representative shards/artifacts.
- [ ] Workspace/clippy/fmt gates plus wasm check for affected core; graph-format no_std if bytes change.
- [ ] Independent equal-budget audit and report recomputation.

## Run contract

- **Metric/current value:** Primary frozen sequence statistic and first-failure horizon from sibling item A, with teacher-forced quality, selective risk, bytes, and compile cost as non-regression constraints; current exact baseline comes from that issue.
- **Reachability ceiling:** Use attribution to compute the fraction of failure steps represented by observable/correctable student prefixes and the maximum headline movement available within the bounded rounds.
- **Pinned identities:** Teacher/source, tokenizer, canonical artifact, prompt/training partitions, decoder/seeds, state/scorer, trace/correction shards, compiler configs, controls, outputs, and reports.
- **Nulls/falsifier:** No correction, ordinary additional teacher-forced evidence, equal-record/byte/cost controls, shuffled corrections, held-out prompts, decoder held fixed.
- **Binding cheap instrument:** Small trace replay and compile must be deterministic; leakage scan clean; controls non-degenerate; reachable failure fraction exceeds the effect floor; disk/RSS projection fits budget.
- **Exit rule:** Corrective arm lower confidence bound beats the strongest equal-budget control on the primary sequence metric within the frozen round count, while all non-regression/resource limits pass.
- **If positive:** Adopt corrective observation as an offline compiler stage and proceed to sibling item C for any diagnosed residual trajectory state.
- **If negative:** Record whether more data, representation, or decoding is limiting; stop corrective rounds and classify generation as not established if the stage gate remains unmet.
- **Cost estimate:** Post teacher calls/tokens, wall time, peak RSS/storage, checkpoint size, artifacts per round, and blocked shared resources.

Post exact values before execution. The full run requires a passing cheap instrument, non-degenerate controls, and distinct outcome branches; missing results are `UNAVAILABLE`/`null`.

## Compatibility and migration

Corrective records are compiler inputs with a versioned provenance schema. If the chosen output requires new artifact sections, those changes belong to the triggered runtime work and must preserve historical behavior.

## Repository conformance

- **Execution scope:** Offline observation/compiler/certifier with measured output through the normative deployed R4Engine path; reference/offline evidence is not credited as deployed generation.
- Preserve P-4, allocation-free steady state, deterministic bytes/ordered reductions, bounded state/work, typed errors, no_std, and replayable witnesses where applicable.
- Empirical criteria bind distribution, horizons, decoder/seeds, n, uncertainty, provenance, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-14, RF-21, RF-22, RF-29.; extend existing capabilities before adding a new one.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant trajectory mechanisms require an explicit diagnostic activation gate; generated conformance is never hand-edited.
- Append measurement records and reconcile live claims whenever the generation boundary changes.

## Claim status and next action

A better teacher-forced fit is insufficient. Only a predeclared sequence-level advantage over equal-budget controls changes the generation claim.

## Comments (2)

### comment 5368815704 — 2026-08-21T10:42:28Z — Casey-allard

### Maintainer decision (2026-08-21): HOLD — corrective rounds wait for the approved S1 redesign direction

#841's run-1 (PR #894, `docs/free_running_841_result.json`) shows the root cause is representational: generation is memoryless beyond the 2-token suffix (99/100 free-running rollouts are token-identical to suffix-only rollouts; median first-divergence step 0; 710‰ cycle collapse). Corrective observation compiled against the CURRENT representation would risk spending the frozen 3-round budget teaching more suffix patterns — the exact failure mode this stage's kill criterion names ("stop defaulting to more same-distribution teacher-forced records").

**Decision:** #840 starts only after the S1 (#822) representation-redesign RFC is approved, and then targets the NEW representation. The #841 §6 improvement bar (median first-divergence +≥2 steps AND diverged-at-0 −≥100‰ per round, no TF-agreement regression >10‰, ≤3 rounds else `GENERATION-NOT-ESTABLISHED`) is frozen and does not move. Unassigned until then; this comment is the hold record.


### comment 5377061243 — 2026-08-22T01:25:46Z — Casey-allard

### Definition of Done — GENERATION-NOT-ESTABLISHED (do-not-launch); merged 2026-08-22

Merged via **PR #912 → main `35f8cc87`** (merge queue, 01:19:51Z). This closes #840 with an evidence-backed negative disposition.

**Disposition.** Per the #840 run contract's binding cheap instrument, the corrective student-prefix run is **not launched**: the reachable ceiling is ~100× below the frozen §6 bar, and the promoted skip-mix representation (RF-31) *regresses* free-running. Recorded **GENERATION-NOT-ESTABLISHED** (limiting factor = representation, not more-data or decoding). This meets the programme global falsifier — bounded student-prefix correction does not reduce the frozen free-running gap → the deployed artifact is a teacher-forced continuation/retrieval system, not a generative engine at this scope.

**Instrument (teacher-free, deployed R4Engine, frozen #841 prompt-family v1, n=100).**
- Harness `crates/uor-r4-api/tests/free_running_reachability_840.rs`; record `docs/free_running_reachability_840.md`; CID-bound result `docs/free_running_reachability_840_result.json` (`result_cid blake3:e9f48e20…`; corpus.meta `blake3:aa9d1767…`, the attested #833 broad-clean bundle).
- **Validated:** the `base` engine reproduces #841 exactly (TF 304‰, suffix-locality 99/100, path histograms) and its base-graph CID `blake3:aaf98b68…` is byte-identical to #908's `base_artifact_cid`.

| metric (H=32, greedy) | base | skip-mix RF-31 |
|---|---|---|
| median first-divergence | 0 | 0 |
| diverged-at-step-0 | 590‰ | 620‰ |
| ≤4-period cycle collapse | 710‰ | 1000‰ |
| matched teacher-forced agreement | 304‰ | 348‰ |
| suffix-locality (FR ≡ suffix-only, /100) | 99 | 19 |
| correctable (graph) footprint | ~1‰ | ~1‰ |
| free-running rollouts changed by the lane | — | 100/100 |

The lane helps *teacher-forced* prediction (+44‰, consistent with #908's +28.45‰ deployed causal) but moves *free-running* away from coherence (exposure-bias amplification). Correctable footprint ~1‰ vs the §6 bar's required 100‰ diverged-at-0 drop.

**Verification.** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `python3 scripts/check_claim_wording.py`, `cargo run -p xtask -- validate` (R1/R4/R5/gnaf; CONFORMANCE.md current), `cargo test -p uor-r4-api` (3 new fixture teeth pass; instrument bundle-gated/ignored in CI) — all green locally; full ladder green through the merge queue. Test + docs only — no serving/runtime/format change, no CONFORMANCE regeneration (RF-14/21/22/29 evaluation infrastructure). Reconciles `docs/RESEARCH.md`.

**Scope.** The formal S3 generation-claim decision (promote/revise/limit/retire) belongs to **#842** (item C), which this result feeds; #842's native blockers (#836, #840) are now both closed. The frozen #841 §6 bar and the #838 selective-prediction gates were not moved (the #887 discipline).



---
_Comment/review ids in this file: 5368815704, 5377061243_

