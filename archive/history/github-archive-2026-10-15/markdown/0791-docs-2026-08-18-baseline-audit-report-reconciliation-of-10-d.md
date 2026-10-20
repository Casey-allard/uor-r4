# #791 — docs: 2026-08-18 baseline audit report + reconciliation of 10 documents to verified ground truth

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:56:26Z
- **Updated:** 2026-08-18T15:15:27Z
- **Closed:** 2026-08-18T15:12:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/791
- **Merged:** 2026-08-18T15:12:39Z
- **Merge commit:** 4b9f0562145b54d1d2c9f05deb92be0e4d367b22
- **Base / Head:** main / UOR-Foundation:audit-2026-08-18-docs-reconciliation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

The 2026-08-18 full-repository baseline audit, delivered as `docs/project_baseline_audit_2026_08_18.md` (774 lines: inventory, serving-engine reachability + attention/operator activation matrices, runtime-invariant findings, verification command ledger, fixture/artifact ledger, live Gate C + inference probes, generalization assessment, finding register, prioritized remediation, open experiments with run contracts) — plus the documentation reconciliation its claim-discrepancy table (§16) requires, every edit mapped to finding IDs (§17).

## Reconciled documents

| File | What changed | Maps to |
|---|---|---|
| `README.md` | E2 cascade/profile reality (production default, typed declines, `engine_profile.txt`); dead `FallbackRouter` sentence corrected; #745 state; `transformerless` silent-no-op note; `uor_standards` wording; `--quality-profile` | D-01/-02/-11/-15/-17 |
| `docs/RESEARCH.md` | #744/#745/#750/#759 states reconciled to GitHub; #759's corrected diagnosis (argmax attractor basins, not the fallback tier) recorded on the row | D-07 |
| `ROADMAP.md` | "Next up" reconciled (open set = #655/#741); #653/#460 untracked remnants called out; capabilities row updated | D-08 |
| `docs/CONFIGURATION.md` | `engine_profile.txt` row; `PORT` + `UOR_R4_RELEASE_BUNDLE_PATH`; #637 phase-3 landed | D-03/-06/-14 |
| `docs/MODEL_LIFECYCLE.md` | stale "compile-path dispatch" section rewritten to the landed #657 `Teacher` dispatch; #637 phase-3 landed | D-05/-06 |
| `docs/SERVING_MODEL_DISCOVERY.md` | dated post-E2 addendum (profile gate; corrected status-surface claim; `select_synthesis_engine` answer) | D-04 |
| `docs/transformerless/BASELINE.md` | fresh 2026-08-18 Gate C row (rule12 36.55%/8.3222 census n=100,306; generalization slice 1.81%/16.89); §6 checklist reconciled | D-12/-17 |
| `docs/transformerless/PROOF.md` | appended dated P-4 scope note (`simd.rs` delegation; Witnessed status per the contract doc) | D-09 |
| `docs/serving_release_packaging_655_d.md` | appended D3 implementation note (synthetic CI round-trip; real-bundle arm env-gated) | D-13 |
| `AGENTS.md` | `uor_standards` wording (tracked-despite-gitignore) | D-11 |

Historical measurement records are appended to, never rewritten, per the repo convention.

## Verification

- `python3 scripts/check_claim_wording.py` — green over the result (including the new report file).
- `git diff --check` — clean.
- Docs-only diff; neither `include_str!` document (`docs/hologram_r4_formal_monograph.md`, `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`) is touched, so the PR-side docs-only fast path applies; the merge queue re-verifies the merged content in full.
- The audit itself ran every CI-equivalent gate at `aea30bae` (κ-reproduction non-vacuous, Gate C trend green); the full command ledger is §11 of the report.

References #655 #741 #783 #784 #785 #786 #787 #788 #789 #790

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

