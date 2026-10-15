# #801 — fix(#787): honest instruments — Witnessed status, engine-scope audit, full P-4 scan coverage, one canonical scanner

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T21:08:51Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-08-18T21:30:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/801
- **Merged:** 2026-08-18T21:30:38Z
- **Merge commit:** 4ca5519eb5aedb1496ad5045fabc83384c36d514
- **Base / Head:** main / UOR-Foundation:issue-787-honest-instruments
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #787 (E-a per the maintainer's make-it-honest decision; E-b and E-c complete).

- **E-a**: the machine-code audit is honestly scoped as an engine self-test everywhere it is named (CI step, module, API, report fields); proof matrix gains the Witnessed tier and Operation-Set Conformance uses it; verify_all has a working falsifier for the first time; the certify obligation mirror follows.
- **E-b**: dot_argmax scalar differential (the scalar reference now compiles on aarch64, where its absence was the gap); delegation-count pin; ALL 11 graph-runtime modules + both previously-unscanned ACTIVITY_OWNERS modules under the P-4 scan with an exactly-pinned 3-entry load-time allowance list (vp_tree construction); contract versions reconciled with a 3-way doc/constant/report sync test; the audit's phantom-generate_into finding corrected on verification (the function exists).
- **E-c**: four divergent source-scanner copies consolidated onto one canonical, literal-aware implementation with its own falsifier suite (per the issue's decision rule, every instrument here demonstrably fails on a seeded violation: scanner falsifiers, the matrix falsifier, and the allowance-list pin all fire red when seeded).

Gates: fmt, clippy -D warnings (workspace), core 58 / graph-runtime 11 / certify-msa 26 / graph-format 7 / proof-model suites green, registered 29/29, R1/R4/R5, wasm32 lib, bdd binary builds (the changed contract scenario runs in CI).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

