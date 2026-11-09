# ORDER: reconcile native approval-flow wording, kimi-10

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `11c7bbf0d1c8c9a4151f0aeb2261f8e34d6d7213`, tree `0df08f64c7042dcc539dcbc9f58554d4704524c9`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. PR #3890 remains draft and unqueued. Fresh reviewer agent-71 returned CHANGES on the approval-count claim; lifecycle, bounds, executed evidence, prior record corrections and application-input identity passed.

## Finding and decision review

- The PR body, current-state, README and local PICKUP describe one host approval per opened shell as if it were the only dialog. The c663 native acceptance actually observed a new **Enable terminal** dialog followed by a distinct **Approve interactive shell** dialog on the second open.
- Source confirms the observed flow: `TerminalSession.open()` calls `actions.enableTerminal()` every time, and `native_enable_terminal` unconditionally shows its confirmation before `openTerminal` requests the exact working-directory/shell approval.
- Preserve the tested consent behavior. This unit corrects the claims to distinguish the Enable terminal confirmation from the one shell-specific cwd/shell approval per open; it does not claim one total dialog or silently make enablement idempotent. No application rebuild is needed for record-only wording.
- Agent-70's CHANGES record is confirmed on live PR #3890. Agent-71's CHANGES is posted at 11c7 and preserved locally before this repair.

## Files in scope

- `web/studio/README.md`, `docs/integration/current-state.md`, `docs/products/desktop-client.md`, `docs/roadmap/items/studio-{ide-harness,typed-shell-upgrade}.md`, and `docs/loop/tasks/studio-ide-harness/kimi-6/REPORT.md` where active-role pointers name the current correction.
- `local/kimi/pr-cowork-body.md`, live PR body, local PICKUP, preserved agent-71 review, and this task folder with exact-head logs under `local/kimi/harness-parity/`.

No application/model/API/engine/worker/tokenizer change, dependency change, test change, native rebuild solely for wording, changed approval behavior, scope waiver, or deletion of evidence/other labs' work.

## Done when

- Every current claim says that each open presents the native Enable terminal confirmation and then one exact cwd/shell approval; no current wording implies one total native dialog. The second-open native observation is disclosed.
- Current roadmap/product/active-role pointers name kimi-10 without erasing kimi-8/kimi-9 history. The PR body still ends `References #3569` and retains full Cowork/API limitations.
- Terminal regression 21/21, repository gates 11/11, local policy, formatting/diff and the complete c663 application/artifact input comparison pass at the corrected head; only README/records differ after acceptance.
- Evidence and agent-71's review are preserved and uploaded/read back. A fresh exact-head reviewer approves before ready/queue. Required GitHub checks, protected queue acceptance, actual merge and fresh-main verification precede branch/worktree/build-output cleanup. Do not queue on agent-71's CHANGES.
