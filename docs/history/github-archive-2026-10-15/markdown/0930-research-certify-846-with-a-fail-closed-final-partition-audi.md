# #930 — research: certify #846 with a fail-closed final-partition audit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-23T01:49:03Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-23T02:12:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/930
- **Merged:** 2026-08-23T02:12:21Z
- **Merge commit:** 9aeb5427c7fd3a965b0e4a0228f137889dbafa9b
- **Base / Head:** main / UOR-Foundation:issue-846-certify-compositional-planning-and-bound-the-reasoning-claim
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Issue #846 requires final certification on untouched, CID-bound composition and topology partitions. The binding pre-access audit found that the committed benchmark has no valid final cell: fitting opened topology cells 0-3, prior joint evaluation opened 4-7, SplitCell has no materialized composition coordinate, and the suite binds neither a final-partition CID nor an access log.

The reachable ceiling is therefore 0 x 512 = 0 certifiable samples. The full grid is NOT_RUN, dependent evidence is UNAVAILABLE, and the evidence-backed verdict is REASONING NOT ESTABLISHED. The narrower RF-33 result remains LIMITED on 12/20 prior joint-split cells; S4 closes LIMIT.

Closes #846

## Scope and non-goals

- Add a source-bound, non-vacuous access-audit test and content-bound JSON result.
- Freeze the #846 execution contract and append the negative certification record.
- Correct the historical #844 final-partition handoff and update current roadmap/research summaries.
- Do not manufacture a post-selection seal, rerun W(3,3), change production behavior, or broaden any reasoning claim.

## Acceptance criteria

- Candidate, source baseline, benchmark, candidate, and audit-input identities are frozen: MET.
- Final-partition seal, CID, access log, disjointness, and materialized composition axis: FAIL before task generation.
- Positive and planted-overlap controls prove the audit can pass and fail.
- Required final slice, witness, integration, second-domain, multiple-comparison, and resource evidence: UNAVAILABLE under the failed prerequisite, never reported as zero or pass.
- Precise public claim boundary and allowed negative verdict: MET.

## Tests and verification

- cargo test -p uor-r4-graph-certify --test compositional_planning_certification_846 --offline
- RF-33 compiler, format, runtime, witness, allocation, and P-4 regressions
- cargo nextest run --workspace with the CI binary(bdd) exclusion: 1811/1811 passed
- cargo test --doc --workspace --offline
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo fmt --check
- graph-format no_std and alloc ladders
- python3 scripts/check_claim_wording.py
- xtask validate, xtask audit-limits, repo-conformance, deterministic-rebuild, and mutation smoke checks

The default local cargo test activated the checkout-only 269 MB live-teacher fixture path and was stopped after it had run for about 55 minutes. GitHub does not provision that fixture; its teacher-parity scenarios vacuously skip by the documented convention. The CI-equivalent nextest split passed, and the protected merge-group run remains authoritative.

## Evidence artifacts

- docs/compositional_planning_certification_spec_846.md
- docs/compositional_planning_certification_846.md
- docs/compositional_planning_certification_846_result.json
- benchmark CID blake3:5ec9e6d885d8c211b3842e9a5d62a5e7a0c02ea5788ca9d079ecfc0c389debe2
- candidate CID blake3:5c16a08dd8684494ee7a68f081a6a215643d8e20eb6f1a9c69197245586c9f59
- audit-input CID blake3:2ede56d0204570c42392efe14afdbf4da5440cf9194e5c3514e42ef15ba288c0

## Compatibility, migration, conformance, and claims

No compiler, artifact, runtime, API, CLI, server, format, or serving semantics change. No migration is required. CONFORMANCE.md remains generated and unchanged at 33 IDs. RF-33 and RF-32 mappings remain unchanged. No general-reasoning, unseen-final-partition, free-running, universal-intelligence, or calibrated-confidence claim is made.

## Intentionally excluded

The user-owned untracked obs-text/ and tf1-pkg/ directories are untouched. A new benchmark and post-hoc final partition are outside this issue; re-entry requires a separately versioned benchmark frozen before candidate fitting and selection.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

