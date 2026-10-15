# #1810 — Reconcile native H4 prediction runtime and vocabulary pool

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:49:12Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-10-06T23:05:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1810
- **Merged:** 2026-10-06T23:05:22Z
- **Merge commit:** 99b1f1e43f97d6fbd17dd3301b6e1621b75bac1b
- **Base / Head:** main / UOR-Foundation:codex/reconcile-native-runtime-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Reconcile the reusable native prediction runtime from the research stack onto current main. Add signed-H4 Generate scoring, exact selected-key state transport and a full legal Generate plus physical-Copy vocabulary pool with alias reduction. Reuse authenticated historical H4 tables and retain sparse-token, artifact, arithmetic, clipping and shape guards.

This is replacement 1 of 3 for the KEEP source in #1792: eight runtime/export files, 2,006 added lines. It preserves component contracts; useful conversation and learned attention remain unestablished. Historical fit/probe commands and run outputs remain retained evidence. Current main's bf16/flash implementation is preserved.

References #820. Exact-head Linux pod checks and source self-review are posted before queueing. The remaining CUDA prerequisites and learning/bank integration are separate PRs based directly on main. Historical research branches and durable artifacts remain preserved.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6027052590 — 2026-10-06T23:04:30Z — Casey-allard

Exact-head self-review: `3d746e8db36ff631b34a58e70c78b61a52eac771`, clean committed source on Linux x86_64 pod `ianty3o30vhgew`. Merged current main into the pushed branch; no rebase. Eight changed implementation files (+2,006/-2).

Validation at this head, exit 0 unless explicitly noted:
- `cargo check --release --locked --offline -p uor-r4-core -p uor-r4-integer`: PASS.
- Core library `native_generate_`: 4 passed, 0 failed; actual directed native scores, sparse domain, conditional replacements and artifact rejection exercised.
- Core library `geometric_read_state_bridge::tests`: 2 passed, 0 failed.
- Integer library `geometric_vocabulary_actions::tests`: 7 passed, 0 failed; aliases, clipping/positive mass, sparse bounds and cache substitution exercised.
- Touched-file `rustfmt --check` for all eight files: PASS.
- Workspace `cargo fmt --check`: FAIL due to existing formatting on main. Its complete output is byte-identical to the separately run baseline at `7c92674a`; no formatting regression introduced. This is not recorded as workspace PASS.

Receipts: `/workspace/uor-r4/codex/reconciliation/runtime-3d746e8d-attempt7/`. Four CPU build jobs, own target directory, GPUs masked. The first test invocation needed a missing locked dev dependency; that dependency was downloaded before the successful run. Earlier failed attempts are retained as execution receipts, not model evidence. Metal-only tests NOT_RUN on Linux; these runtime changes do not alter a Metal path.

Source review: trusted H4 table/order admission, reserved -8/padding rejection, artifact digests, bounded dimensions, signed transport direction, sparse legal-ID reduction, Copy occurrence aliases, checked sums and shared clipping were reviewed. Independent source review of the unchanged implementation at parent `4fe2ec95` found no blocker; merging main changed only AGENTS.md and upstream MQAR code. `canonical_exp` visibility remains crate-local to reuse authenticated table admission.

APPROVE for the declared runtime component integration. This is not a whole-binary instruction audit, attention/chat qualification or laptop performance/energy result. Constructors and diagnostics allocate; numerical scoring/reduction use the declared integer/table operations.



---
_Comment/review ids in this file: 6027052590_

