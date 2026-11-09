# REPORT: reconcile native approval-flow wording, kimi-10

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Wording-repair source: `541ac6e36cabe348701353ffdcbb6a7769ab011d`, tree `13b0cef0c0f4b751ca5235a3f66b947a7f601ed8`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. Application acceptance remains `c66377b8fa77bbd225e52acba6539885dedc0c4a`, tree `16a6f5e6392310b6dae71f213e826226e61b9919`. Fresh reviewer agent-71 returned CHANGES at 11c7 on the approval-count claim; the verdict is posted on PR #3890, which remains draft and unqueued.

## What did not work

- Current claims described one host approval per opened shell as though it were the only native dialog. In the actual c663 second-open acceptance, a new **Enable terminal** dialog appeared before a distinct **Approve interactive shell** dialog.
- Source verifies that behavior: `TerminalSession.open()` calls `actions.enableTerminal()` on every open, and `native_enable_terminal` unconditionally presents its confirmation before the separate exact cwd/shell approval. Saying one total host dialog was therefore inaccurate.
- The inaccurate wording appeared in the PR body, current-state, README and local PICKUP. Current roadmap/product pointers also needed to name this correction rather than stop at kimi-9.

## What changed

- Current records now state that every open presents the native Enable terminal confirmation followed by one exact cwd/shell approval. Later opens repeat both dialogs; there is one shell-specific approval per open, not one total host dialog.
- The PR's native acceptance description explicitly names both second-open dialogs and cancellation of the shell approval. README, current-state, desktop product records and local PICKUP use the same behavior-bound wording.
- Current roadmap/product/active-role pointers name kimi-10 while retaining kimi-7 implementation, kimi-8 reconciliation and kimi-9 record-correction history.
- The tested consent flow is unchanged. No application, model, API, engine, worker, tokenizer, dependency or test source changed, and no native rebuild is claimed for this record-only repair.

## Measurements

- At 541ac, `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **21/21 PASS**.
- At 541ac, `uor-harness loop gates --full`: **11/11 GREEN**.
- The complete c663..541ac application/artifact comparison covered root Cargo/config, full `src-tauri`, full `web/studio` except only README, launcher/site scripts, typed-runtime test and integer/tokenizer/vision crates. Quiet diff returned **0** and name-only diff returned **0 with empty output**.
- Existing c663 evidence remains integer **499 passed/1 ignored**, default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, typed **453/453 across 52 files**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, **7/7** WASM, interactive and combined loopback, and packaged native PTY acceptance including both second-open dialogs.

## Checks run at the head

- `local/kimi/harness-parity/541ac6e36cabe348701353ffdcbb6a7769ab011d/` retains the wording-source terminal and full-gates logs. Final report-only head policy, terminal, gates and application-input identity are bound in the PR body rather than assigned a self-referential hash here.
- Agent-71's complete CHANGES review is preserved as `local/kimi/harness-parity/review-11c7bbf0d-agent71.log` in both worktree and main checkout; `cmp` returned 0. Agent-70's posted CHANGES record was also confirmed on live PR #3890.
- The final report-only head reruns local policy, terminal 21/21, gates 11/11, formatting/diff and the complete input comparison before fresh review. Agent-71's review and final evidence are uploaded/read back before cleanup.

## Skills used

Adversarial review reception, source/native-evidence verification, behavior-bound claim correction, documentation synchronization, artifact preservation, and verification before completion.

## Still open

Final report-only head checks, fresh exact-head review, required GitHub checks, protected queue acceptance of the full head scope, actual merge and fresh-main verification. Full persistent runtime, parallel tasks, project memory, stop/redirect, approvals orchestration, Dispatch/scheduling, browser/computer workflows, connectors, live R4 API integration, production publication, signed release and useful-model capability remain unestablished.

## Plain summary

The records now describe the actual two-step native consent flow without weakening or changing it. The accepted application remains input-identical to c663; protected delivery and full Cowork capability remain open rather than implied.
