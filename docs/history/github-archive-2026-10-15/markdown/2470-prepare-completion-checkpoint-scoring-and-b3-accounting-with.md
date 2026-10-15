# #2470 — Prepare completion checkpoint scoring and B3 accounting without model compute

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T14:24:19Z
- **Updated:** 2026-10-14T14:28:29Z
- **Closed:** 2026-10-14T14:25:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2470
- **Merged:** 2026-10-14T14:25:56Z
- **Merge commit:** 062313d63b36c451454bc86a8a02427f84832f10
- **Base / Head:** main / UOR-Foundation:codex/completion-scoring-readiness-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: evaluation integrity · count 1/3 unchanged · headline 82/14287 → 82/14287 (archived pilot top-1, unchanged; receipt-only readiness, no new model result).

Prereg: #issuecomment-6170315113 records the owner-assigned receipt-only scope and budget. This task performs no model compute and requests no new FIT or release.

## What and why

Prepare completion-scale receipt scoring before the long run finishes. A standalone Rust tool validates saved checkpoint metadata and both full 14,287-row DEV evaluations, reports B3 admission and admitted strict wins separately, and performs the registered integer comparisons. The source audit confirms that final batch index 16,403 saves completed cursor 16,404; four evaluations each contain 224 DEV batches. Legal cell occupancy below eight is accepted. Frozen D5/model source remains identical to reviewed `33564b75e`.

## Result and limits

The actual archived pilot replay took 1.04 s wall with 76,349,440 bytes maximum RSS and reproduces its recorded row totals and loss decomposition. This is host receipt-processing cost only. It does not execute the CUDA-gated score/run branch, qualify Claude's new build, forecast GPU latency, authorize compute or manufacture a completion/fresh-transfer result. KEEP the receipt tool for the registered checkpoint outputs; model status remains NOT YET PROMOTED.

12 scorer tests pass, including the actual archived checkpoint parser fixture and refusal mutations; 25 unchanged default-feature D5 metadata tests pass. Reusing an output path refuses without changing its digest. Exact-source independent review approves scorer SHA256 `0fe2bde7a4f24fbc2a4f45291a91e793c110a3c0626ddac40a57b46686b6919e`. Final-head validation and harness findings are posted below.

## Checklist

- [x] Harness check and lint run; any pre-existing coordination findings are reported explicitly below, not model negatives.
- [x] No model compute; no FIT or owner release requested or created.
- [x] Checks run at this exact head: `cargo test --offline --manifest-path docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/Cargo.toml` (12 PASS, real archived input environment paths), `cargo test -p uor-r4-discrete --offline --lib` (25 PASS); source hashes reverified at final head `53c5580b74c44f1fb6db970a77065a73e5f3c16c`. Commits after validation changed only claim state.
- [x] Canonical record, operator commands, source pins, limits and retained rehearsal JSON: `docs/labs/completion-scoring-readiness-2026-10-14/README.md`.
- [x] `docs/integration/current-state.md` updated with Next.
- [x] STATUS / ROADMAP / tracker headline unchanged; README capability narrative unchanged.
- [x] D23/D25 ratchets and `python3 scripts/check_claim_wording.py` pass. No new Python dependency.
- [ ] Protected merge, fresh-main content verification, owned branch/worktree cleanup and hygiene; results will be posted on #2029.

Next: Claude retains the separately qualified completion build and execution; Kimi uses the scorer on the real registered checkpoint receipts. Named mechanism successors remain unadopted. Training authorization is unchanged.

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6170647851 — 2026-10-14T14:25:02Z — Casey-allard

Codex/Sol final-head review: APPROVE at `53c5580b74c44f1fb6db970a77065a73e5f3c16c` for receipt-only scoring readiness. Independent source reviewer also APPROVES scorer SHA256 `0fe2bde7a4f24fbc2a4f45291a91e793c110a3c0626ddac40a57b46686b6919e`; final source and retained output digests were rechecked after the claim-release commit. No frozen D5/model source delta from `33564b75e`.

Actual validation on these exact source bytes:
- Rust scorer tests: 12 PASS; archived report and checkpoint-1640 supplied through `SCORING_AUDIT_PILOT_REPORT` and `SCORING_AUDIT_PILOT_CHECKPOINT`. Includes in-memory parser fixture, malformed arithmetic/aggregate/count reference/cursor/lineage/model metadata refusals, legal lower cell occupancy, final16404/20Route accounting.
- Unchanged discrete crate, `cargo test -p uor-r4-discrete --offline --lib`: 25 metadata tests PASS. No CUDA feature and no model forward.
- Optimized actual archived pilot replay: 1.04s wall; 76,349,440B maximum RSS. Retained output SHA256 `b543dbf48df839d523c65644c9c52869ac7e1f29a38c646f1ce4b1a1beafa03e`. Existing output refusal exit1 leaves that digest unchanged.
- `cargo fmt --check` for scorer, D23/D25 Rust ratchets, claim-wording gate, `git diff --check`, and shard integrity PASS. Six shards/358 receipts/no losses at verification.
- Harness `check`: exit0, “50 rules, 15 failed (0 error, 14 warn)”; pre-existing merged peer claims, unidentified historical receipt, stale peer presence and unstarted/PENDING panels. No peer state modified. This is not reported as a warning-free check.
- PR lint: exit0, zero errors, one warning because the unchanged archived pilot top-1 metric differs from M1's separate registry reply-quality metric. No new model result or registry change is claimed. Initial lint's formatting/reference errors were corrected in the PR body; no scientific threshold was changed.

Remaining boundaries: future CUDA evaluation latency and the new binary remain Claude's qualification; typed inner checkpoint/evaluation snapshot hashes and seed2 original-baseline authentication remain production-validator scope. Stored batch epoch/target IDs are absent and are explicitly not invented. No owner release, model compute, two-seed verdict or fresh transfer. Training authorization unchanged.

Proceed through protected merge. Claim/release is recorded in this PR's history and the final claims file is unchanged from base, avoiding a stale merged claim.



---
_Comment/review ids in this file: 6170647851_

