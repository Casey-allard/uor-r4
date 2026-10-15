# #948 — roadmap: recenter UOR-R4 on an end-to-end uor-matmul geometric causal decoder

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T23:27:20Z
- **Updated:** 2026-08-26T00:06:33Z
- **Closed:** 2026-08-26T00:05:37Z
- **Labels:** documentation
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/948
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The table/graph path is a scoped teacher-forced retrieval/continuation system, not a coherent free-running geometric language model. The original prime-router value survives as identity-scoped geometric memory and retrieval, while the repository's source weights/tokenizer/causal path provide the shortest controlled route to local syntax through `uor-matmul`.

This issue published the maintainer-directed architecture reset as the repository and GitHub source of truth.

## Objective

Recenter UOR-R4 on one product outcome: a local CPU-first geometric causal decoder that produces coherent text, progressively removes source self-attention, uses `uor-matmul` for learned projections, and makes persistent R4 geometry causally load-bearing in next-token selection.

## Scope

- [x] Replace programme root #820's S0-S7 sequencing with the geometric-decoder dependency chain.
- [x] Create the complete bounded child-issue list with native parent and `blockedBy` relationships.
- [x] Explicitly disposition every formerly open roadmap issue as reused, separately blocked, or closed not planned/not triggered.
- [x] Publish an authoritative geometric causal-decoder roadmap and architecture decision.
- [x] Reconcile AGENTS, README, ROADMAP, RESEARCH, CONTRIBUTING, historical plans, lifecycle/configuration, explainers/glossary, research guidance, crate READMEs, and issue templates.
- [x] Split invariants by lane: preserve P-4/`no_std`/allocation-free contracts for frozen R4G1 artifacts while allowing the active `uor-matmul` decoder to use learned floating-point/matrix computation.
- [x] Make focused product verification the default; certification stays nightly/manual or promotion-triggered.
- [x] Require product-path reachability, non-degenerate controls, student-prefix evidence, bounded training, and decision-changing stop rules.

## Non-goals

- Implement the decoder in this governance issue.
- Delete historical graph, proof, conformance, or measurement artifacts.
- Relabel pointwise evidence as coherent generation.
- Weaken frozen R4G1 invariants.
- Add a test framework or launch a long experiment.

## Acceptance criteria

- [x] #820 and the native graph identify exactly one immediate engineering issue: #950.
- [x] No obsolete S5-S7/F0 issue remains ambiguously active.
- [x] Entry points name the new roadmap and classify old plans as historical evidence.
- [x] Transformerless is distinguished operationally from multiplication-free.
- [x] Reuse/freeze/replace decisions are explicit.
- [x] Research and verification policy supports fast iteration while rejecting vacuous geometry.
- [x] `python3 scripts/check_claim_wording.py` passes.

## Verification

- [x] Repository entry points reviewed against #820/#949 and the live native graph.
- [x] `git diff --check`.
- [x] Claim-wording gate.
- [x] Issue-form YAML parse and changed Markdown target validation.
- [x] Protected pull-request and merge-group `fast build + product smoke` checks.

## Completion

Merged by PR #956 as `abff35071bffdb2b6e27949d3346341339e82bfd`. Governance and architecture decision only: no language-quality, geometry-advantage, transformerless-runtime, performance, or deployment result is claimed.


## Comments (1)

### comment 5418716559 — 2026-08-26T00:06:32Z — Casey-allard

Completed by #956 and merged through the protected queue as `abff35071bffdb2b6e27949d3346341339e82bfd`.

Evidence:
- pull-request `fast build + product smoke`: PASS;
- merge-group `fast build + product smoke`: PASS in 13s;
- local diff/claim/YAML/Markdown-target gates: PASS;
- #820/#949 and native #950 → #955 dependency chain verified live;
- obsolete S5-S7/F0 issues and milestones dispositioned without rewriting their evidence;
- milestone 21 is the sole active engineering milestone.

#950 is now the sole immediate engineering leaf. It is unassigned, has no open implementation PR, and its only native blocker (#948) is closed.



---
_Comment/review ids in this file: 5418716559_

