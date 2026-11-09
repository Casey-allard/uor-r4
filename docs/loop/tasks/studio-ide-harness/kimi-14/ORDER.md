# ORDER: record the verified typed Studio publication, kimi-14

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `dfee63a0449863ce12a6549aaf8beaccddf3c4af`, tree `66270e868a68c9c4ee81192db4700be5cf776d27`; merged PR #3897 is `a8e51c0f4d0ed6e999b6d9e13f8289215efbd684` with tree `0f41d59fb5fe10291f5b5c4bef0a190081c50484`. Fresh-main publish commit is `6540e2717d3a606330dbd141e3642f3f13b87ecf`, rollback head `ab752d59586c595f9409d8c9ba59026a51d4a613`.

## Verified result and decision review

- Fresh-main typed dry run and explicit staged acceptance passed: Playwright 45/45, direct staged WASM 7/7, 14 requests with zero external/raw-TypeScript/runtime errors, and 11/11 repository gates. The reviewed local commit was pushed to `gh-pages` after all checks passed.
- Pages `pages-build-deployment` run `42668187497` completed successfully. The public URL now renders the neutral UOR Harness workbench at desktop and restored mobile primary. Mobile split/secondary close works, and live Terminal explicitly reports `Interactive PTY is unavailable in this browser build.`
- Evidence and screenshots are preserved under `local/kimi/typed-publish-main/a8e51c0f4d0ed6e999b6d9e13f8289215efbd684/` and in project store revision `a1814d1e3fe0c9b109aab3de7e77ce395d929700`, downloaded and recursively compared.
- Repository records must now distinguish completed static publication from still-missing full agent runtime, hosted native workflows, live R4 API, signed release and useful-model behavior. Issue #3569 remains open; this status PR uses References, not Closes.

## Files in scope

- `web/studio/README.md`, `docs/products/{web-client,desktop-client,harness-parity}.md`, `docs/integration/current-state.md`, `docs/roadmap/items/studio-{ide-harness,typed-shell-upgrade}.md`, this task folder and the status PR body/evidence.
- No application/model/API/engine/worker/tokenizer/publisher change, new dependency, `.github/**` edit, second publish, public terminal service or deletion of prior evidence/other labs' work.

## Done when

- Current records name `6540e2717d3a606330dbd141e3642f3f13b87ecf` and `ab752d595` as the previous publish, link the successful Pages run and live desktop/mobile evidence, and no longer describe typed publication as pending.
- The item stays `ready` with full parity open. Local policy and 11/11 gates pass, a fresh exact-head reviewer approves, required checks and protected merge complete, and evidence is uploaded/read back before merged branch/worktree and reproducible-output cleanup.
