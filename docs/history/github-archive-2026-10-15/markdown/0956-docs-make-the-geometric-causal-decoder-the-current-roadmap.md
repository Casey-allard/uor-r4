# #956 — docs: make the geometric causal decoder the current roadmap

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-26T00:04:03Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-26T00:05:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/956
- **Merged:** 2026-08-26T00:05:36Z
- **Merge commit:** abff35071bffdb2b6e27949d3346341339e82bfd
- **Base / Head:** main / UOR-Foundation:issue-948-geometric-decoder-roadmap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

UOR-R4's repository entry points and open GitHub programme still sequenced S5-S7 graph scale/proof/release work even though S3/S4 evidence showed that the deployed table/graph representation does not produce coherent free-running language.

This PR makes the geometric causal decoder the authoritative programme. It returns to prime-router's useful context/memory role, establishes the local source/`uor-matmul` path as the control to restore, binds persistent memory before training, and replaces source attention progressively only while bounded student-prefix behavior survives.

## Scope delivered

- Adds `docs/geometric_causal_decoder_plan.md` and ADR-0002 as the current architecture and sequencing authority.
- Reconciles AGENTS, README, ROADMAP, RESEARCH, CONTRIBUTING, lifecycle/configuration, explainers, glossary/formal vocabulary, research archive guidance, crate READMEs, issue forms, and historical-plan banners.
- Defines transformerless operationally: zero source-attention calls and no dense full-prefix Q·K matrix/softmax kernel; bounded geometric support must be causally load-bearing under disabled/permuted interventions. Multiplication-free remains a separate post-viability decision.
- Makes focused checks plus one bounded product transcript/operator report the normal engineering loop; certification remains manual/nightly or contract-triggered.
- Requires tokenizer-bound persistent memory before all-layer replacement, a five-prompt G0 control, bounded mixer-only G1 training, student-prefix evidence, negative stop rules, and native reblocking before a negative predecessor closes.
- Corrects stale user-facing claims about historical output quality, Gate C in PR CI, R4G1 serving ownership/order, router retrieval defaults, and old plan authority.

## Live GitHub reconciliation

- Reused #820 as programme root and created #949 as the sole active intelligence tracker.
- Created exact native chain #950 → #951 → #952 → #953 → #954 → #955 under milestone 21; #950 remains blocked by this issue until merge.
- Closed obsolete S5-S7/F0 work as not planned/not triggered without relabeling its historical evidence.
- Parked #940 as separate administrator-only CI cleanup.
- Closed obsolete milestones; milestone 21 is the sole active engineering milestone.

## Acceptance status

- [x] GitHub and repository identify exactly one immediate engineering issue after merge: #950.
- [x] Old S5-S7/F0 work is no longer ambiguously actionable.
- [x] Entry points name the new roadmap and mark old plans historical.
- [x] Mechanism reuse/freeze/replace decisions are explicit.
- [x] Research gates reject unreachable/vacuous geometry without adding a testing programme.
- [x] Current production remains ADR-0001/R4G1 until #953 earns promotion.

## Verification

- `git diff --check`
- `python3 scripts/check_claim_wording.py`
- YAML parse of both changed issue forms
- Local-target validation for changed Markdown links
- Live native parent/`blockedBy`/milestone/assignment audit for #948-#955
- Independent final architecture review: no remaining P0/P1 contradiction

Protected docs CI is the remaining merge gate. No Rust/build code changed, so workspace/BDD/certification suites were intentionally not run.

## Compatibility, conformance, and claims

No runtime bytes, artifact format, generated conformance file, RF capability, serving default, or release claim changes. Historical R4G1/TLA/P-4/`no_std`/allocation-free evidence remains scoped and intact. This PR establishes governance and architecture only; it establishes no language-quality, geometry-advantage, transformerless-runtime, performance, or deployment result.

No user-owned model/corpus artifacts or unrelated worktree changes are included.

Closes #948
References #820
References #949


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

