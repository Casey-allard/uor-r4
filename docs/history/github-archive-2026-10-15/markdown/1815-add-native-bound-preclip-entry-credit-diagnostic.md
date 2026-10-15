# #1815 — Add native-bound preclip entry credit diagnostic

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T00:52:06Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-10-07T00:53:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1815
- **Merged:** 2026-10-07T00:53:21Z
- **Merge commit:** 27c9e84a8d5d06ce0e2bbe9cc289a67673948951
- **Base / Head:** main / UOR-Foundation:codex/preclip-entry-credit-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The current offline vocabulary marginal clips the differentiable raw score graph, so a wrong Copy atom above +8 receives no direct score credit through that clamp. Add a separate opt-in preclip margin diagnostic using the same native trace admission, without changing the default loss or serving behavior.

The new CUDA example restores retained floating-point masters and verifies their hashes and native exports. It independently reloads integer inference and checks full-bank Copy, original quarter bridge, Generate and vocabulary-pool parity before consulting answer labels. It performs zero updates and records projected local gradient directions plus unchanged parameter and packet identities.

At this exact head, the retained step-128 checkpoint passed all eight predeclared construction rows in a 28-second RTX 5090 run. Four highest wrong Copy atoms exceed the clamp. Both old and diagnostic losses have nonzero potential gradients; the diagnostic reaches the blocked score, but its larger norms do not establish a better direction, clipping causation, exported improvement, attention success or language capability. The next discriminator compares token-alias credit policies before selecting a fit.

Validation: release CUDA example build PASS; three new component tests and three named existing marginal regressions PASS; touched-file rustfmt, diff check and claim-wording PASS. Actual checkpoint inference/backward PASS with independent integer forward parity, zero updates and unchanged masters. No Metal execution. Canonical receipts: /workspace/uor-r4/codex/reconciliation/entry-credit-0cbf32c/. Source 0cbf32c5c30d9ef93f7592fc33bb186b7508588e; CUDA executable SHA256 632d1fae916a2b39b12208e08c339a45c5f8bdea46f0c4fddf1c187747282a6c.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6028454483 — 2026-10-07T00:52:27Z — Casey-allard

Exact-head self-review: 0cbf32c5c30d9ef93f7592fc33bb186b7508588e.

Default loss retains its prior clipping and native probability anchor; its shared full-pool admission is unchanged. New margin is opt-in and diagnostic only. The complete target-free native pool precedes target use; Copy-covered labels and mismatched traces/raw score graphs are rejected, with first-occurrence integer ties retained. Callers must establish empty-prefix entry scope.

Checked at this head on Linux x86_64: release CUDA example build PASS, three new tests PASS, three named existing marginal regression tests PASS, rustfmt --check on all three touched Rust files PASS, git diff --check PASS, claim-wording PASS. This is focused validation, not a whole-workspace formatting or Metal claim.

Actual RTX 5090 run: source-bound executable SHA256 632d1fae916a2b39b12208e08c339a45c5f8bdea46f0c4fddf1c187747282a6c; eight predeclared indices 0..7; full independent integer bank/bridge/Generate/pool parity; original floating-point master restoration and export equality; zero updates; unchanged parameter hashes and native score/action/state packets; seal/verify PASS. GPU log 20261007T004754Z-entry-margin-gpu0.log exits 0 at 00:48:22Z after 28s. Pod released/deleted.

Adversarial review independently inspected source and actual receipts. Old potential gradients are nonzero in all eight cases; the margin restores a direct alternative credit path but does not establish a total credit gap, better direction, finite exported improvement, clipping causation or language qualification. Next compare raw-score token-alias credit with scale-normalized directions before admitting another fit. No default serving path, geometry artifact, GPU trainer softmax policy or other lab seam changes.



---
_Comment/review ids in this file: 6028454483_

