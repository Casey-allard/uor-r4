# REPORT: record the verified default Studio publication, kimi-17

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `4160a5347717d7e6786848895b758bfc5dc663f7`, tree `ace49417fef35d3ea321b2332cc2bb9cbc07cbb6`. Record implementation head: `7524cd6195a78c186814e854c4e537e837b5a390`, tree `1076ece577a3804023f6b52cb4ce351326adc726`. The following REPORT commit changes only this record; final identity is bound in the PR body and `identity.txt` after final checks.

## What did not work

- Current records called the default cutover pending after it was already deployed. This unit records the measured public result rather than making another application or deployment change.
- A fresh worktree's first runtime-contract run had four failures and one error because generated `pkg`/`dist` inputs were absent. The standard typed publisher generated and verified them; the rerun passed **15/15**. That setup failure is not a merged-source negative.
- The first public verifier invocation lacked `process.argv[1]` and exited before Chrome. The next used a repository path as the HTTPS origin and misclassified same-site requests as external; the actual page had one shell, no legacy MCP, exact arithmetic and restored sessions. The corrected origin run passed. Both instrument issues remain documented in `live-acceptance.json` and the mismatch log.
- The first fitted mobile browser captures produced duplicated mosaics. Native-scale reloads produced clean images, independently matching Playwright mobile output; no application fix is claimed for a capture-state artifact.
- The public `models.json` is 1,346 bytes with SHA-256 `fa33aa230b37b1d788983f9adb910f35d4562679db5351a5cf32e758d0c5ed79`, containing only the synthetic demo and identical before/after this cutover. Older text implied a current five-model/M1-card catalog. Records now describe the actual synthetic-only catalog; no model/catalog change or M1-card restoration is claimed.

## What changed

- PR #3903 is **MERGED** as `4160a5347717d7e6786848895b758bfc5dc663f7`, tree `ace49417fef35d3ea321b2332cc2bb9cbc07cbb6`. Its first-parent diff is the approved 19 Studio/record files; Studio source matches reviewed head `ca8700bf0b8180bd91af581262a83192a18e1193` byte-for-byte. Independent model work in the merge was not edited.
- Fresh-main checks: **455/455** typed tests across 52 files, **47/47** Playwright checks, **15/15** runtime contracts after generated-input preparation, **11/11** full gates, static page/worker checks, both runtime routes and migration cases, and direct **7/7** staged WASM checks.
- Typed publisher dry run produced `4a435ab8af9c1da6d49225c30f9dda0a4ad50b87`, changing seven built files. The documented site-worktree push advanced gh-pages from `6540e2717d3a606330dbd141e3642f3f13b87ecf` to that commit without force. Rollback is `6540e271…`; older `ab752d595…` remains preserved.
- Pages run `42714479496` succeeded; the Pages API reports built at `4a435ab8af9c1da6d49225c30f9dda0a4ad50b87`. Public `index.html` is byte-identical to the staged file (SHA-256 `889f1368799a084740d305964bad55300ccb6b966aed804d5e615c57db8bfe12`).
- Plain `/studio/` renders the neutral workbench at desktop and mobile sizes. `?legacy=1` and `?typed=0` retain the old interface; static Terminal explicitly reports `Interactive PTY is unavailable in this browser build.`
- Isolated public runtime: **14 requests, zero external/raw-TypeScript/runtime errors**, one shell, `legacyMcp=0`, `typedMcp=1`, `typedMcpAfterWorkflow=0`, artifact ready, exact `17 + 25 = 42`, and isolated/restored transcripts. `public-runtime.json`, `live-acceptance.json`, and public desktop/mobile screenshots are under `local/kimi/default-cutover/fresh-main/`.
- No GPU, paid compute, real credential/profile, second deployment, model/API/application change or useful-model result belongs to this record unit.

## Measurements

- Current-state, web-client, parity, serving-notebook and Studio roadmap records now name the actual merge, publish commit, successful Pages run, rollback head and live evidence. Static preview is a completed default-route result only; hosted native workflows and full Cowork/API parity remain open.
- Historical typed-preview entries remain identifiable. Current publish and catalog references no longer contradict the deployed `4a435ab8…`/synthetic-only read-back.
- No application, model, engine, worker, tokenizer, API, Tauri, native-consent, dependency, `.github/**`, owner roadmap metadata or second-publication change was made.

## Checks run at the head

- All **11** full repository gate commands passed at record implementation `7524cd6195a78c186814e854c4e537e837b5a390` / `1076ece577a3804023f6b52cb4ce351326adc726`; see `record-implementation-gates.log`.
- `git diff --check` and the scoped file audit pass. This PR has eight scoped files including ORDER/REPORT, no generated binaries and no owner-path changes. Automatic claim was not forced past the item's pre-existing shared roadmap metadata.
- After this REPORT-only commit, `loop report --check`, `loop pr-check --local`, and `loop gates --full` results are retained as `record-final-report.log`, `record-final-policy.log`, and `record-final-gates.log`, with `identity.txt` and clean status under `local/kimi/default-cutover/fresh-main/`. Fresh exact-head approval, required GitHub checks and actual protected merge are verified separately, not inferred from this record.

## Skills used

Protected branch delivery, exact artifact and runtime acceptance, real built-in-browser and Playwright inspection, scope control, systematic instrument diagnosis, and verification-before-completion. Fresh-context delivery review is required before merge.

## Still open

Fresh exact-head approval, required GitHub checks, actual protected merge, fresh-main record verification, and scoped branch/worktree cleanup. Public trained-model catalog restoration is not part of this unit. Shared agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, connectors, embedded browser/computer use, live R4 API, signed release and useful-model behavior remain unbuilt. Issue #3569 stays open and this PR says References, not Closes.

## Plain summary

The old hosted plain URL has been replaced by the verified neutral workbench. This PR records the actual public deployment, its rollback and the synthetic-only catalog read-back. It does not claim full Cowork parity, a hosted terminal, live R4 API integration or useful conversation.
