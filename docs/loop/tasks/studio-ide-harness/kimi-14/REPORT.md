# REPORT: record the verified typed Studio publication, kimi-14

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `dfee63a0449863ce12a6549aaf8beaccddf3c4af`; merged PR #3897 `a8e51c0f4d0ed6e999b6d9e13f8289215efbd684`, tree `0f41d59fb5fe10291f5b5c4bef0a190081c50484`. Publish commit `6540e2717d3a606330dbd141e3642f3f13b87ecf`; rollback head `ab752d59586c595f9409d8c9ba59026a51d4a613`.

## What did not work

- Current records still described typed publication as pending and `ab752d595` as the current public build after the actual deployment. They needed to distinguish completed static publication from still-open full agent runtime and native workflows.
- Earlier evidence/identity findings are preserved: two stale eb832431a claims and a third 454bb30fc claim were corrected before the protected merge. The implicit Playwright server was replaced with an explicit retained-stage server, the failed browser cleanup was labeled separately, and live/local models.json identity was executed rather than inferred.

## What changed

- Fresh-main typed dry run produced publish commit `6540e2717d3a606330dbd141e3642f3f13b87ecf` after Playwright **45/45**, direct staged WASM **7/7**, 14 requests with zero external/raw-TypeScript/runtime errors, and **11/11** repository gates. The reviewed local commit was pushed to `gh-pages` only after those checks passed.
- `pages-build-deployment` run `42668187497` completed successfully. The live URL now renders the neutral UOR Harness workbench at desktop and restored mobile primary. Mobile split/secondary close works, and Terminal explicitly reports `Interactive PTY is unavailable in this browser build.`
- Current-state, README, roadmap, notebook, product and parity records now name the actual publish and successful Pages run, preserve `ab752d595` as rollback, and point to live evidence instead of calling publication pending.
- This status PR only updates records; it does not republish, alter the application, add a service, or claim full Cowork parity, live R4 API, signed release or useful-model behavior.

## Measurements

- Merge `a8e51c0f4d0ed6e999b6d9e13f8289215efbd684`, publish `6540e2717d3a606330dbd141e3642f3f13b87ecf`, rollback `ab752d59586c595f9409d8c9ba59026a51d4a613`.
- Fresh-main staged Playwright **45/45**, direct staged WASM **7/7**, 14 requests with zero external/raw-TypeScript/runtime errors, and **11/11** gates. Artifact SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`, 806,759 bytes.
- Live `models.json` remains 4,365 bytes with SHA-256 `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a` and includes both M1 and research FBS seeds.
- Desktop and restored mobile primary screenshots plus `publish-identity.txt` are under `local/kimi/typed-publish-main/a8e51c0f4d0ed6e999b6d9e13f8289215efbd684/`. Store revision `a1814d1e3fe0c9b109aab3de7e77ce395d929700` was downloaded and recursively compared: 61 files matched.

## Checks run at the head

- `pages-build-deployment` run `42668187497`: **SUCCESS** at `6540e2717d3a606330dbd141e3642f3f13b87ecf`.
- Hard-reloaded public `/studio/?typed=1&publish=6540e271`: neutral UOR Harness at desktop and restored mobile primary; split/secondary close works; explicit static-terminal unavailability.
- Status-record final `uor-harness loop pr-check --local` and `uor-harness loop gates --full` are rerun after this REPORT is committed. Fresh exact-head review and required GitHub checks remain outstanding.
- `cargo xtask roadmap check` retains **four problems total**: two unrelated existing roadmap-file problems (`float-kernel-fusion.md` missing `## What is missing` and `infra-desktop-client-tauri.md` using unsupported `in_progress`) plus two stale `issues.json` ids (`float-kernel-fusion` and `infra-desktop-client-tauri`). Those files were not edited or represented as passing.

## Skills used

Fresh-main source verification, explicit-root browser and Playwright acceptance, live desktop/mobile verification, evidence preservation, and documentation synchronization.

## Still open

Status-record reconciliation in kimi-15, then fresh exact-head review, required checks, protected queue acceptance and actual merge, then evidence preservation and branch/worktree/build-output cleanup. Full agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, hosted native workflows, live R4 API, signed release and useful-model behavior remain unbuilt.

## Plain summary

The neutral workbench is published and verified live. The remaining work is to land the status record and continue the larger harness runtime, not to claim that static publication completed the whole harness or model.
