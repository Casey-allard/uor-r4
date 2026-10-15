# #2328 — Use masked TRAIN count reference and diagnostic DEV oracle

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T13:26:06Z
- **Updated:** 2026-10-13T13:34:42Z
- **Closed:** 2026-10-13T13:30:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2328
- **Merged:** 2026-10-13T13:30:26Z
- **Merge commit:** 9e904a83448c451f738d5c1d45224dfdf73eb84e
- **Base / Head:** main / UOR-Foundation:codex/rung0-count-reference-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The rung-0 rank reference counted the whole DEV token stream, allowing unmasked prompt frequencies to distort the response-only comparison. It now counts only explicitly masked TRAIN targets on CUDA. A separately labelled DEV-response oracle is diagnostic only, and the native CLI requires and binds the TRAIN mask/count/reference to settings, resume lineage, checkpoints and standalone evaluation.

References #2029, #2028, #2030. Line: unified geometric route/action/learning integration; count11/3→12/3 for this completed integrated unit, no reset. Headlines unchanged: M1 reply37/232 memory20/40; M2 saved437/512 opened0/128 freshNOT_RUN. All training/M2 holds remain HELD. Scientific Amendment5 remains PENDING until Claude confirms the delivered source plus Kimi command pins; only Casey releases training.

Validation at qualified source2fd6358bbc52d2a202a872ad0954ebb12894606f: CUDA test compilation PASS2.05s; ordinary CLI build PASS55.04s; 11 focused controls PASS (9 metadata,2 CUDA). CUDA exclusion/invariance/distinct-distribution control and M1/M8 evaluator/artifact round-trip both pass on RTX5090/Ubuntu24.04.3/CUDA12.8.93/Rust1.97.1. Final PR source, Cargo.toml/lock and selected core/model dependencies equal that qualified source; newer harness work is preserved. Exact source review approved by independent subagent, corrected JSON binding re-reviewed. No optimizer update, training, CPU model/reference or corpus-quality compute. Old full-path controls remain inherited at their original source scope, not rerun evidence.

Actual no-softmax/no-matmul/claim-wording ratchets PASS (1648/93 unchanged); formatting/diff and append-only shard check PASS. Historical compatibility checks are acknowledgements, not these tests. Failed omitted-metadata builds, corrected compile error and zero-test filter are preserved, with zero selected tests explicitly NOT_RUN.

The canonical record is docs/labs/count-reference-2026-10-13/README.md with current-state/STATUS/ROADMAP/operator-policy/cache updates. Superseded whole-DEV source has an indexed restoration patch, verified against an isolated Git index. HF qualificationfa6f56f4ac6864e82030ee45365e679faaaa1a6e is download-verified; iCloud archive is round-trip/index verified. Pod released/deleted; estimated lifetime cost$0.2041 (invoice unverified). Kimi owns the scientific sheet; peer source, receipts, claims and jobs remain preserved. Protected merge/source proof and claim cleanup follow actual delivery.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6146627397 — 2026-10-13T13:27:15Z — Casey-allard

Exact-head review and validation: APPROVED at `b2c16532b3708e66b5d353f17e44f7a8957c4d1e`.

Independent source reviewer approved qualified source2fd6358bb (including the corrected JSON binding). This PR head's complete training-crate tree is `85817cc162028b07476f3b3732818964b5c0fe58`, exactly equal to2fd6358bb. Cargo.toml/lock, core, integer, model-source and third_party also have no changes from that qualified source. Peer harness commits were merged with all three newest peer current-state entries preserved; only the Codex prefix was added. Foreign claim entries equal fresh main; no peer source/job/artifact was touched.

Actual qualification: 11 focused tests PASS, CUDA compilation PASS, ordinary CUDA CLI build PASS. Source/test/ordinary executable hashes and raw failed/corrected compiler/control receipts are pinned in the canonical record and download-verified HF qualificationfa6f56f4. The old source restoration patch independently restores both original files. Actual checks: rustfmt/diff, no-softmax1648, no-matmul93, claim-wording, JSON, ledger shard guard5shards/107receipts/no loss PASS. No optimizer/update/corpus-quality or CPU model run. Compatibility statuses carry no test claims.

Review scope: masked TRAIN eligibility, deterministic CUDA reference scoring, separately diagnostic DEV oracle, CLI initial/periodic/final wiring, before-model binding/refusal, checkpoint/artifact identities and preservation. No P1/P2 found. Global/M2 holds HELD; scientific sheet remains PENDING until Claude confirms the actual merged source and Kimi pins the explicit TRAIN mask/count. Owner alone releases training.



---
_Comment/review ids in this file: 6146627397_

