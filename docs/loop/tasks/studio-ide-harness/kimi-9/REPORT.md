# REPORT: correct final record discrepancies, kimi-9

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Record-repair source: `9b14dc5d0659e1edc169a724929d81f9aa9b2e28`, tree `47f79032708941c2bb11a1ae72f9e2f69e3d33a9`; merged main `fbf15e56f3ab1961b30610893392e7a97fd0ab18`. Application acceptance remains `c66377b8fa77bbd225e52acba6539885dedc0c4a`, tree `16a6f5e6392310b6dae71f213e826226e61b9919`. Fresh reviewer agent-70 returned CHANGES at b8c on records only; the verdict is posted on PR #3890, which remains draft and unqueued.

## What did not work

- One `499/499 PASS` label remained in the historical kimi-6 report despite the actual integer result of 499 passed, 1 ignored. The PR body omitted the commit-order advisory while the reconciliation report said it was disclosed.
- README/current-state evidence pointers used a generic final-head path. They did not distinguish merged typed-shell delivery, c663 workbench/PTY application acceptance, and b8c record-only reconciliation evidence.
- Current roadmap/product records still called kimi-8 the final reconciliation without naming this remaining correction.
- At the interim record-repair source, `pr-check --local` returned FAIL solely because this REPORT was not yet present. That red result is retained in the 9b14 evidence and is not represented as a final policy pass; the report-only head requires a fresh rerun.
- The first report-only head used `## Checks and evidence`, which local policy rejected because the required heading is exactly `## Checks run at the head`. The red log is retained under `local/kimi/harness-parity/faed449a0f03c7f3d9dce60da66d7765d97502ab/`; the heading is corrected without changing any application input.

## What changed

- The remaining integer label now says **499 passed, 1 ignored; 0 failed**. The unsupported-default/all-target negatives remain disclosed and are not converted into passes.
- The PR body names full-surface scope, commit-order and FILEMAP advisories. No advisory is treated as protected queue acceptance.
- The typed-shell pointer now names merged PR #3879 head `8e06a6f3e0b5852ffb5a133fac149eb9a42a7582`, tree `7a85df3afaee00e8c15c0e77c8552051195620e1`, and its existing identity.json. Workbench/PTY records explicitly distinguish c663 application acceptance from b8c record-only reconciliation and subsequent PR-bound delivery heads; no nonexistent b8c identity.json is claimed.
- Current roadmap, product, notebook and historical/active-role pointers name kimi-9 where they describe this correction. No application, model, API, engine, worker, tokenizer, dependency or test source changed.

## Measurements

- At 9b14, `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **21/21 PASS**.
- At 9b14, `uor-harness loop gates --full`: **11/11 GREEN**.
- The complete c663..9b14 application/artifact comparison covered root Cargo/config, full `src-tauri`, full `web/studio` except only README, launcher/site scripts, typed-runtime test and integer/tokenizer/vision crates. Quiet diff returned **0** and name-only diff returned **0 with empty output**.
- Existing c663 evidence remains integer **499 passed/1 ignored**, default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, typed **453/453 across 52 files**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, **7/7** WASM, interactive and combined loopback, and packaged native PTY acceptance. No native rebuild was needed for these record-only edits.

## Checks run at the head

- `local/kimi/harness-parity/9b14dc5d0659e1edc169a724929d81f9aa9b2e28/` retains terminal, full-gates and interim missing-REPORT policy logs. Final report-only head logs and application-input identity are bound in the PR body rather than assigned a self-referential hash here.
- Agent-70's complete CHANGES review is preserved as `local/kimi/harness-parity/review-b8caa6a8-agent70.log` in both the isolated worktree and main checkout; `cmp` returned 0.
- The final report-only head reruns terminal 21/21, gates 11/11, local policy, formatting/diff and the complete input comparison before fresh review. Uploaded evidence is read back before any cleanup.

## Skills used

Adversarial review reception, exact source/evidence identity checking, documentation synchronization, artifact preservation, and verification before completion.

## Still open

Final report-only head checks, fresh exact-head review, required GitHub checks, protected queue acceptance of the full head scope, actual merge and fresh-main verification. Full persistent runtime, parallel tasks, project memory, stop/redirect, approvals orchestration, Dispatch/scheduling, browser/computer workflows, connectors, live R4 API integration, production publication, signed release and useful-model capability remain unestablished.

## Plain summary

The correction resolves the three record findings without changing the accepted application. It preserves the real ignored test and prior negatives, binds evidence to existing identities, and leaves protected delivery and full Cowork capability open rather than implied.
