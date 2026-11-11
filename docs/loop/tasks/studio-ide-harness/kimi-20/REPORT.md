# REPORT: register the LexLean MVM workbench item, kimi-20

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569, registration parent only). New item: `studio-lexlean-mvm-workbench`; official issue sync is pending. Order: [ORDER.md](ORDER.md). Base: `93e089ff256654e75a19641b7e2fbd2bc8e24524`. Draft PR: #3938. References #3569; leaves its status and acceptance unchanged.

## What did not work

- The initial plan omitted Tauri adapter paths from scope, named a hand edit of generated roadmap output, proposed a committed initramfs binary and omitted `status` from the worker protocol. Those defects were corrected before review.
- The first independent architecture review returned **NOT APPROVED / request revision**, with four critical and twelve important findings. It found that the existing editor cannot list MVM files, desktop WASM packaging/execution was unproven, GitHub targets lacked repository pinning and stale-base protection, and the smoke/authority/persistence boundaries were underspecified.
- The reviewer confirmed the MVM core is credible safe portable Rust, but no native/browser build was run in this registration unit. Its read-only GitHub CORS probe supports technical feasibility only; no direct static-web token write was exercised and it remains UNVERIFIED.
- The first fresh exact-head review returned **CHANGES**: the vsock provenance did not name its generated-file source explicitly; static-web companion evidence was ambiguous; the generated roadmap view is intentionally stale until official post-merge sync/render; and smoke evidence belonged in Git rather than ignored `local/`. It also requested a PR-body refresh and two wording corrections. These findings are addressed in the current documents; re-review is pending.
- No application, model, API, service, browser, MVM or Tauri implementation was performed. No smoke repository, branch, commit or PR has been selected or written.

## What changed

- Added a ready M7 item with `done_class: host` and a corrected seven-task plan for the LexLean three-pane web/desktop workbench, real RV64IMAC Linux-userland emulator, virtual files/Git, C compilation, CoW/vsock and approved non-production GitHub workflow.
- Pinned upstream commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` and exact hashes for `mvm.rs`, `mvm_cc.rs`, initramfs, generated source and upstream tests. Only `lexlean_generated.rs:45,751-46,087`, the self-contained 337-line vsock tail module, is selected; every other generated line and the AST Cargo evaluator are excluded.
- Resolved the review's critical findings in the documents: explicit MVM `WorkspaceEditor.fileSource`, local Vite `assets/*.wasm` plus required real desktop status/cc, per-session `RepoTarget` allowlists, and companion-only desktop writes through `AgentHost` under unchanged CSP.
- Resolved the important findings in the documents: per-file editor revisions, base ref/blob SHA CAS, idempotent single-flight actions, numeric persistence/protocol caps, per-tab atomic persistence, memory-only history, secret-input refusal, allowlisted worker exports, bounded repository tree/content seeding, host-receipt `gh` calls, default-skipped smoke and exact upstream output tests.
- Added explicit lazy-worker, accessibility/IME/RTL/query-mode, no-new-lock-dependency and software-emulator wording. The smoke is a human-approved project fixture edit, not a bug-fix or model-capability claim.
- Only the new item, its design/plan and this parent registration folder changed. Size is reported, not capped.

## Measurements

- Base and fetched `origin/main` were both `93e089ff256654e75a19641b7e2fbd2bc8e24524` at the probe. No `crates/uor-r4-mvm` exists at that base. Live GitHub found no open PR overlapping Studio/Tauri/MVM/root Cargo paths; #3569 is OPEN.
- Independently downloaded pinned upstream bytes match the recorded hashes: `mvm.rs` 98,525 bytes / `2789ec57…7206`; `mvm_cc.rs` 75,765 / `19390475…e12f`; initramfs 12,164 / `071e9c8d…c65c`; generated source 2,206,963 / `acc9a89d…47e8`; upstream test 9,573 / `9b99e2d6…3c7b`. Full values are in the design.
- Disk at probe: 40 GiB free according to `df -h`; this is a filesystem reading, not a cleanup claim. No model artifact, paid compute, credential or deployment was used.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check`: **PASS** after the corrected scope/design, 144 items (ready 25, held 11, decision 1, done 107).
- `git diff --cached --check`: **PASS** on the approved design before this REPORT update; rerun at the final head.
- Independent design sequence: first **CHANGES** (C1–C4/I1–I12 plus minors), second **CHANGES** (I13 CoW/message-cap and I14 duplicated-tab ownership), final **APPROVE** with no unresolved Critical/Important design inconsistency. The final corrections and named tests are in the design/plan/item/ORDER.
- Residual implementation tests, not design blockers: Web Locks availability/fallback in the desktop WebView, Vite emission from `node_modules/.cache`, AgentHost GitHub tool registration, exact smoke-target availability and Tasks 1–2 native/browser preflight.
- The first policy command omitted `--bin uor-harness`; the report check then required the parent-registration `--order` and passed with it. `loop pr-check --local` reads committed diffs and reported no ORDER before the first commit as expected. These invocation/staging diagnostics are not policy passes or product failures.
- `loop report --check --order docs/loop/tasks/studio-ide-harness/kimi-20/ORDER.md`: **PASS**. `loop pr-check --local` at `0f9be9435`: **PASS**; the intermediate trimmed ORDER briefly lost the required literal `Item:` label and failed, then `4246472a62fa4cde495339caaf85df45e768c887` restored it and **PASS**ed with only expected registration/pre-existing advisories and a 4,088-byte ORDER. Final policy is rerun after this REPORT-only commit.
- `loop gates --full` at `4246472a6`: **12/12 GREEN** (`stack-contract`, two fmt gates, claim-wording, loop watch/drive/next sources/next wait/owner direction, git-hooks, migrate and review-changes-blocks), exit 0. Logs: `local/kimi/gates/1794434185-98058` and session task `bash-751d97pu`.
- `uor-harness check` at the same head: exit **0**, begins `476 rules, 445 failed (0 error, 444 warn)`; warnings are existing multi-lab/status/prep rows plus this branch's unclaimed status. No warning is presented as a clean whole-project bill of health.
- Disk during checks changed from the earlier 40 GiB probe to 6.9 GiB `df -h`; hygiene reported 4.9 GiB before its dry run and about 0 GiB automatic reclaim. Active/unuploaded work was kept and no deletion was performed. Post-delivery cleanup is limited to identified reproducible artifacts from this lab.
- `cargo xtask roadmap render --check` at `729aa5287`: **EXPECTED_DEFERRED FAIL**, ready 25 versus rendered 24 because the new item cannot be rendered before registration. Kimi owns immediate post-merge `roadmap sync --only studio-lexlean-mvm-workbench --apply` and the separate official render path before implementation; no hand edit or render PASS is claimed.
- First fresh exact-head review at `729aa5287`: **CHANGES**, with the provenance/web-evidence/render/evidence-location findings recorded above and host checks otherwise green. Corrections are in the current candidate; exact-head re-review and required GitHub checks are pending and not claimed.
- Final HEAD/tree, actual protected merge, official issue sync/render, fresh-main verification and cleanup remain pending until observed.

## Skills used

- Isolated worktree, working discipline, scope guard, context guard, solution integrity, executing-plans preparation, test-driven-development preparation and systematic invocation-failure diagnosis. No production code was written in this registration unit.
- Independent `uor-architect-reviewer` design review examined the real source, current Studio/Tauri paths and upstream provenance across three reviews. The top session applied corrections; no subagent decided acceptance or scope. Fresh exact-head review is separate.

## Still open

- Final loop/full-gate checks and protected delivery remain pending. All seven implementation tasks remain pending until official issue sync/render completes through the official path.
- Real R4 generation, useful-model evaluation, model-driven autonomous bug fixing, production deployment/mutation, direct static-web token writes and full Cursor/LexLean parity are NOT_RUN or UNVERIFIED as labelled. A future smoke PR is limited to a dedicated non-production target and is not evidence of model capability.

## Plain summary

This is a registration unit, not a product capability delivery. The independently approved design addresses the concrete editor, desktop, GitHub, persistence, concurrency and provenance gaps found by review. Roadmap validation passes; final gates, protected merge and official issue creation are still required before implementation begins.
