# REPORT: preserve normal process exits in the bounded native host, kimi-76

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `35f55d334e654047821b4fe66226e8e4ea3d3069`, tree `40f4e1084d8c09945a2e851789c6ab0c6024a742`.

## What did not work

- Exact-head default desktop acceptance failed `scoped_file_and_git_commands_enforce_granted_folder_and_refuse_dangling_symlinks`: an unborn repository reported `HEAD (timeout)` instead of `HEAD (unborn)`. The control binary took 5.55 seconds, not the declared 30-second process deadline; the focused control passed in 0.94 seconds.
- `run_bounded_process` could break on output-channel disconnection and then terminate a child whose pipes had closed but whose normal exit was not yet visible to `try_wait`. This explains the scheduling-dependent false Git timeout without blaming Git performance.
- One default run also returned the previously disclosed loopback MCP `401 bad credential`; the focused transport test and complete default rerun passed without an MCP source change. One GUI run killed two 25 ms descendant fixtures before their shells wrote PID files; both focused controls and the complete GUI rerun passed without changing their deadlines or assertions. These instrument failures remain recorded, not rewritten as clean first attempts.
- Parallel draft #3879 contains an equivalent helper hunk within broader acceptance work. This repair remains independently red-first and deliberately excludes that draft's unrelated serving and acceptance changes.

## What changed

- Added `closed_pipes_before_normal_exit_are_not_a_timeout`. On the old helper it failed in **0.01 seconds** because a normal exit after pipe closure was mislabeled as a timeout. After the helper change it passes and the child exits successfully.
- When readers disconnect without a prior kill, the helper now waits for child exit only until the original deadline before entering the existing timeout/termination path. No deadline, process-group, output-cap, approval, UI, model, engine, worker, tokenizer, or MCP behavior changed.
- Added `closed_pipes_with_a_live_child_still_time_out` to prove that closed pipes cannot hide a genuinely live child past the deadline. Existing descendant-pipe and output-cap controls still pass.

## Measurements

The false-timeout regression failed in 0.01 seconds before the repair. The repaired package is identified by executable SHA-256 `5453941831b8afa136f488efbadb998d077f993d86237269c0948813b3d5e92d`, and native CDP used 13 initial requests with zero external requests or runtime errors.

## Checks run at the head

- Focused repaired Git control: **1/1 passed**. Default desktop: **9 library, 0 main, 19 controls, 7 MCP, 13 review, 8 sidecar, 0 doc-tests passed**. GUI desktop: **11 library, 0 main, 23 controls, 8 MCP, 13 review, 9 sidecar, 0 doc-tests passed**. `cargo fmt --check` and `git diff --check` pass.
- Fresh unsigned app built with `cargo tauri build --features gui`; executable SHA-256 is `5453941831b8afa136f488efbadb998d077f993d86237269c0948813b3d5e92d`. The package contract and both real-app Python tests passed with that expected hash.
- The preserved packaged window loads `tauri://localhost/index.html?typed=1` and displays **`HEAD (unborn) · 2 changed files`** with the complete `note.txt` and `untitled.md` diff. Neither `HEAD (timeout)` nor `diff timed out` appears after the repair. The test app PID was stopped by exact ownership.
- Evidence is under `local/kimi/studio-shared-shell/process-control-repair/` and the exact-head logs. The app is preserved under `local/kimi/studio-shared-shell-repaired-54539418/` in the main checkout's local lab area. These are unsigned development observations, not a signed release, production publication, useful-chat, M7/M8, accessibility-certification, or Phase B claim.
- Native CDP passed: one typed shell, 13 initial requests, two independent sessions, exact UOR `42`, byte-identical transcript restoration, all four migration cases, and zero external/raw-TypeScript/runtime errors. The loopback runner was stopped by exact task ownership. The complete committed-head product/legacy/repository matrix, fresh exact-head review, protected PR delivery, and fresh-main verification are **NOT_RUN** at this record edit.

## Skills used

Systematic process-boundary diagnosis, red-first test-driven repair, macOS accessibility verification, isolated native-browser verification, and verification before completion. A fresh exact-head `uor-delivery-reviewer` is still required before protected delivery.

## Still open

Complete the current committed-head matrix and independent review before protected delivery. Broader final browser/desktop acceptance, mobile/accessibility acceptance, signed release, production publication, and Phase B remain separate; #3500 remains open.

## Plain summary

A normal child can close its output pipes just before exiting. The host mistook that scheduling window for a timeout, causing intermittent false Git timeout labels. A deterministic regression now distinguishes that case from a genuinely late child, and the repaired packaged Changes view returns the real unborn branch and full diff without timeout flags.
