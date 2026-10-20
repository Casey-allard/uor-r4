# #747 — docs: honest overhaul of core documentation + track-viability assessment + issue roadmap (maintainer request)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T22:55:34Z
- **Updated:** 2026-08-16T23:21:55Z
- **Closed:** 2026-08-16T23:20:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/747
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Per the maintainer's direction after the #655 coherence sweep (which produced #743/#744/#745): perform a deep audit of the repo and live code, do a thorough honest overhaul of the core documentation to reflect current reality (no changes to goals or existing measurement evidence), answer \"which of our actual tracks seems like it could even work\" given the project's goal of an LLM that uses no transformers and no matrix multiplication in deployed inference, and scope the order the remaining issue backlog should be tackled in.

# Scope

- Audit the deployed-inference P-4 enforcement (`p4_contract_owned_graph_runtime_source_scan`, `matrix_operation_census.rs`) against README/RESEARCH.md claims — confirm what's machine-checked vs narrative.
- Audit each engine track exposed via `engine=` on `/api/chat` (transformerless, r4g1, attention, r4-attention, geometric) plus the dormant #604 route-attention kernel and GNAF (#653), and classify which are genuinely goal-aligned (no transformer/matmul in the deployed path) vs comparison baselines vs out of scope.
- Update README.md, ROADMAP.md, docs/RESEARCH.md (and any other core doc found stale) to fold in: #743's landed fix, the #655 sweep's sharper generation-quality evidence, GNAF phase 1 (#742) landed / phase 2 deferred, and #744/#745 as open.
- Add an explicit, durable answer to \"which track could work\" honestly bridging the positive offline per-position signal (#509/#320) against the negative live end-to-end generation finding (#745) — without editing or removing any existing measurement record.
- Recommend a sequencing for the open backlog (#744, #745, #653 phase 2, #740, #741).

# Non-goals

- Not rewriting the ~70 numbered research-note docs under `docs/` (design/measurement records) — those are append-only measurement history per RESEARCH.md's own discipline. This issue is scoped to the core narrative docs (README, ROADMAP, RESEARCH, and cross-links) plus any doc found to actively misdescribe live code.
- Not resolving #744 or #745 themselves — this issue documents the current state honestly, it doesn't fix the underlying generation-quality gap.
- Not changing the P-4 contract, the deployed serving default, or any existing measurement conclusion.

# Acceptance criteria

- [ ] README/ROADMAP/RESEARCH.md accurately reflect main@current state (post-#743, post-#742, with #744/#745 referenced as open)
- [ ] A clearly-flagged section answers the track-viability question with citations to the underlying evidence
- [ ] A recommended issue sequencing is recorded (in ROADMAP.md and/or the closing comment)

## Comments (2)

### comment 5310094099 — 2026-08-16T22:55:50Z — Casey-allard

Starting on this one. Plan: (1) finish the code audit I already began (P-4 enforcement, engine tracks, GNAF, dormant #604 route-attention), (2) update README/ROADMAP/RESEARCH.md to fold in #743/#742/the #655 sweep findings, (3) add an explicit track-viability section bridging the #509 positive offline signal against the #745 negative live-generation finding, (4) record a recommended sequencing for #744/#745/#653/#740/#741. Working in .worktrees/i747-docs-overhaul off main@6505d72f (includes #743).

### comment 5310196715 — 2026-08-16T23:21:55Z — Casey-allard

Landed in #748 (main@ed83e7e2). Summary of what changed and what the audit found:

**Found and fixed one real drift**: README claimed geometric-router retrieval stays word-overlap "until issue #490 clears its gate" — #490 (and #502) actually closed 2026-08-08, over a week before this audit, and the content-vector query has been the deployed default since. Corrected.

**Found an incomplete project-layout table**: `uor-r4-naf`, `repo-model`, `repo-conformance`, and `xtask/` are real workspace members with no README entry; `proofs/wasm-gemm-gnaf/` (#742) wasn't mentioned in the layout at all. Added all four.

**Added the track-viability answer** at `docs/RESEARCH.md#which-track-can-actually-produce-coherent-text`: the offline per-position signal from the broad-teacher programme (#509/#320) is real, replicated, and P-4-clean (deployed inference is machine-checked matmul-free — `p4_contract_owned_graph_runtime_source_scan`, `matrix_operation_census.rs`), but it hasn't yet composed into coherent end-to-end generation on the best locally compiled bundle (#745, open). Classified the engine tracks: transformerless/R4G1 are the only goal-aligned ones with any positive signal; the geometric router is a real, validated component but a retrieval mechanism, not a candidate to *be* the LLM; `attention`/`r4-attention` are explicitly comparison baselines (real teacher matmul/float attention); GNAF is formal-verification infrastructure for a WASM GEMM proof, unrelated to text generation. Flagged the dormant, P-4-legal #604 route-attention kernel (built, differentially tested, wired into no serving path) as an unevaluated lever worth trying against #745.

**Recorded the recommended sequencing** in ROADMAP.md's new "Next up" section: #744 (fix the quality gate — cheap, and everything else depends on trusting it) → #745 (root-cause the degenerate generation — the load-bearing open question) → #653 phase 2 (GNAF wiring — infrastructure, doesn't move generation quality) → #740/#741 (deferred, correctly, until generation quality lands).

No goals or existing measurement conclusions were changed — this was corrections and additions to drifted/missing narrative text, verified against the live code and closed issues, not a rewrite.

Moving on now to deliver the maintainer's requested final report of recommendations.


---
_Comment/review ids in this file: 5310094099, 5310196715_

