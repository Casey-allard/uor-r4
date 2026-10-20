# #945 — runtime(#944): record the production prefix-memory canary

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T21:12:14Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-25T21:25:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/945
- **Merged:** 2026-08-25T21:25:42Z
- **Merge commit:** cf22879a35c896ece3df886ccdaca8716d4b3ee6
- **Base / Head:** main / UOR-Foundation:issue-944-production-memory-canary
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Run the bounded S3 non-vacuity canary authorized after #942 against the exact schema-2 production envelope ratified by #933.

Verdict: **INERT** for this artifact.

- 512 deterministic one-step cases
- 492 explicit context-row resolutions
- 20 primary context-signature probes, all admitted
- 0 primary probe misses
- 0 secondary session probes attempted/admitted
- 0 candidate-list changes
- 0 served-token changes
- ordered-observation CID `blake3:c6876af09d4c416f10f7098b18eda94cf8e6e4217e9cf28bb66a73f8f86748fe`

The production runtime now exposes fixed-capacity routing attribution so the canary can distinguish an actually exercised session probe from signature-byte inequality. The exact envelope was re-verified before the 53.6-second teacher-free scan. Per the predeclared stop rule, the broader #841 rerun is **NOT_RUN**.

## Focused verification

- `cargo fmt --all --check`
- `git diff --check`
- `python3 scripts/check_claim_wording.py`
- `cargo test -p uor-r4-graph-runtime --test r4g1_runtime_test session_signature_enters_rout_fallback_as_secondary_probe --offline`
- `cargo test -p uor-r4-api --test production_prefix_memory_canary_944 --offline` (classifier PASS; exact evidence run is intentionally ignored by default)
- exact #933 envelope canary: `INERT`, 512 cases, 53.6s
- `cargo check -p uor-r4-api --all-targets --offline`
- `cargo check -p uor-r4-graph-runtime --no-default-features --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`

No teacher, compile, observation, full #841 evaluation, BDD, fuzz, audit, or certification run was launched locally.

## Claim boundary

This establishes a negative product result for the exact admitted artifact. It does not establish coherent generation or change the frozen #841 bar. The next S3 representation change must make complete-prefix information behaviorally reachable before this canary is repeated.

Closes #944

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

