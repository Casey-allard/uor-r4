# ORDER: reconcile publication status records and exact-head evidence, kimi-15

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `dfee63a0449863ce12a6549aaf8beaccddf3c4af`, tree `66270e868a68c9c4ee81192db4700be5cf776d27`; current main `2c5d2a407f54ef43169507ffec598999dbb241fd`. Agent-76 returned CHANGES on PR #3899 at eb806 and the verdict is posted. The actual neutral workbench remains live at `6540e2717d3a606330dbd141e3642f3f13b87ecf`; this repair does not republish or revert it.

## Findings and decision review

- `docs/products/web-client.md` still calls typed production publication a following unit, and `docs/roadmap/items/studio-typed-shell-upgrade.md` still requires mobile/accessibility/cutover checks before publication although static publication is complete. Make those scopes current without erasing history.
- `kimi-14/ORDER.md` names 7303 while its REPORT names dfee as base. Both must use the actual status-PR base `dfee63a0449863ce12a6549aaf8beaccddf3c4af` / tree `66270e868a68c9c4ee81192db4700be5cf776d27` before later merge evidence is distinguished.
- The gates ledger's working-tree hash is not HEAD's tree. Rerun local policy and full gates only after all record edits and merge resolution, with an explicitly empty `git status --porcelain`, exact `HEAD`/`HEAD^{tree}`, and retained ledger/log. Do not claim a dirty-tree receipt for HEAD.
- The PR body must include the native-window/fresh-exit-0 limitation and the exact two stale `issues.json` ids `float-kernel-fusion` and `infra-desktop-client-tauri`. Other unrelated roadmap failures remain disclosed, not fixed or presented as passing.
- Merge current main after record correction, resolve the README/current-record conflict by preserving actual live commit `6540e271` and all unrelated model changes, and confirm the resulting GitHub diff contains only status records. A fresh exact-head review and required checks precede queue.

## Files in scope

- `docs/products/web-client.md`, `docs/integration/current-state.md`, `docs/products/desktop-client.md`, `docs/products/harness-parity.md`, `docs/roadmap/items/studio-{ide-harness,typed-shell-upgrade}.md`, `docs/research/notebook/m4-serving.md`, `docs/loop/tasks/studio-ide-harness/kimi-{14,15}/`, and the status PR body/evidence.
- No application/model/API/engine/worker/tokenizer/publisher change, new dependency, `.github/**` edit, second publish, public terminal service, or deletion of prior evidence/other labs' work.

## Done when

- No current record calls static publication pending or requires more pre-publication checks; static publish `6540e271` and rollback `ab752d595` are consistent everywhere. Historical rows remain historical.
- ORDER/REPORT base and later merge identities are distinct and correct; PR body includes both required disclosures and ends `References #3569`.
- Current main is merged without losing model work, the GitHub diff is records-only, and local policy/full gates pass at the exact committed clean head with empty status and matching ledger/log evidence.
- Fresh exact-head reviewer approves, required checks and protected queue acceptance complete, actual merge is verified, and evidence is preserved before cleanup of merged worktrees and reproducible outputs.
