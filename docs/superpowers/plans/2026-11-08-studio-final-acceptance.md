# Final Mounted Studio Acceptance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove and repair complete browser and packaged-desktop workflows on the merged mounted Studio runtime, then deliver them through protected review and merge without publishing production.

**Architecture:** Preserve the single Vite `dist`, existing UOR integer backend, and native host boundaries. Use the existing desktop loopback host for full browser capabilities, explicit static/native-runner limitations for unsupported host actions, and native IPC for Tauri. Repair the bounded process exit race and expose only read-only approval state to the browser; all authority remains host-owned.

**Tech Stack:** Existing Rust/Tauri, Preact/Vite/TypeScript, Node CDP checks, macOS accessibility, and repository harness. No new dependency.

**Spec:** `docs/superpowers/specs/2026-11-08-studio-final-acceptance-design.md` and `docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/ORDER.md`.

## Global Constraints

- Preserve engine/model/tokenizer/`studio_api.js`, release defaults, existing static assets, host token/Origin/grant checks, and the 60-second one-shot approval contract.
- No new dependency, external test endpoint, real browser profile/credential, source-TypeScript mount, second UI/service, production publish, or Phase B compute.
- Use only task-owned fixtures and `local/kimi/studio-final-acceptance/` evidence; Cargo output stays under `target/kimi-final-acceptance`.
- Static-WASM and `uor-chat` native-runner limitations must remain explicit rather than being presented as successful native actions.
- Deliver exact-head fresh review → protected PR/checks/queue → actual merge → detached fresh-main checks → guarded cleanup. Do not publish production in this plan.

## Review Focus

- A child closes both pipes before a normal exit: return success without `timed_out`, while a still-running child at the deadline is terminated and marked timed out (Task 1).
- Page RPC attempts to enable terminal or create approval: fail without changing backend state; only explicit host CLI controls can create the 60-second grant (Task 2).
- Wrong cwd/command, stale approval, cancellation, or a second use of one approval: refuse execution and preserve output/state honesty (Tasks 2 and 3).
- Static-WASM/`uor-chat` unsupported Git/terminal and post-reopen model-unselected states: show explicit capability state rather than fabricating success (Tasks 3 and 4).
- Reopened sessions, pasted attachments, editor contents, and Git output must match retained state exactly; no credential duplication or destructive reset (Tasks 3 and 4).
- Raw-TypeScript requests, external page-load requests, wrong-host/token/Origin requests, and out-of-grant paths: reject without mounting or invoking the host (Tasks 2 and 3).

---

### Task 1: Repair bounded process exit truth

**Files:**
- Modify: `src-tauri/src/lib.rs:1301-1405` (`run_bounded_process`)
- Test: `src-tauri/src/lib.rs:1720-1809` (`bounded_process_tests`)

**Interfaces:**
- Consumes: existing `run_bounded_process(command, timeout, max_bytes)` and `BoundedProcessOutput`.
- Produces: unchanged signature/output shape; normal pipe-close-before-exit has `timed_out == false`, while deadline expiry retains `timed_out == true` and termination.

- [ ] **Step 1: Write the failing test**

Add `closed_pipes_before_normal_exit_are_not_a_timeout` beside the existing timeout tests. Run `/bin/sh -c 'printf ok; exec 1>&- 2>&-; sleep 0.05'` with a one-second bound. Assert stdout is `ok`, `output.status.success()`, and `!output.timed_out`.

- [ ] **Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" CARGO_TARGET_DIR=target/kimi-final-acceptance cargo test --manifest-path src-tauri/Cargo.toml closed_pipes_before_normal_exit_are_not_a_timeout -- --exact`
Expected: FAIL because the current post-loop `try_wait` observes the sleeping child and marks `timed_out` immediately.

- [ ] **Step 3: Implement bounded wait-for-normal-exit**

In `run_bounded_process`, when `readers_disconnected && !killed`, poll `child.try_wait()` with a short sleep only until the existing `deadline`. If the child exits, do not set `timed_out`. If it remains running at the deadline, preserve the existing terminate/timeout path. Do not alter the output-cap or descendant-pipe branches.

- [ ] **Step 4: Run focused and existing boundary tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" CARGO_TARGET_DIR=target/kimi-final-acceptance cargo test --manifest-path src-tauri/Cargo.toml bounded_process_tests`
Expected: the new test and all existing process timeout/descendant/output-cap tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "[kimi] distinguish closed pipes from process timeout (kimi-71)"
```

### Task 2: Serve reviewed dist and expose read-only approval state

**Files:**
- Modify: `src-tauri/src/main.rs` (CLI controls and `--studio-root`)
- Modify: `src-tauri/src/lib.rs` (approval query and RPC read-only responses)
- Test: `src-tauri/src/main.rs` (CLI control tests)
- Test: `src-tauri/tests/desktop_client_controls.rs` (RPC/loopback contract tests)
- Modify: `docs/products/desktop-client.md` (exact CLI approval behavior)

**Interfaces:**
- Consumes: `DesktopBackend::native_set_terminal_enabled`, `host_approve_terminal_command`, `handle_studio_http_request`, and existing `createDesktopActions` methods.
- Produces:
  - CLI `--studio-root <dir>` defaulting to current `web/studio`.
  - CLI `--enable-terminal` and `--approve-command <cwd> <command>` using existing host methods only.
  - `DesktopBackend::host_terminal_command_is_approved(&self, cwd: &Path, command: &str) -> bool`, a non-mutating grant/expiry/exact-match query.
  - RPC `native_enable_terminal` returns current `{ enabled }` without mutation; `native_confirm_terminal_command` returns current `{ approved }` without creating or consuming approval. `native_run_terminal_command` consumes one valid grant once.

- [ ] **Step 1: Write failing host/permission tests**

Cover: default root compatibility; a temporary typed dist serves its hashed JS with token bootstrap and no raw `.ts`; page `native_enable_terminal` cannot flip false to true; page `native_confirm_terminal_command` cannot create approval; CLI preauthorization is exact and expires/gets consumed once; wrong cwd/command and second use refuse; out-of-grant approval refuses.

- [ ] **Step 2: Run tests to verify they fail**

Run: `PATH="$HOME/.cargo/bin:$PATH" CARGO_TARGET_DIR=target/kimi-final-acceptance cargo test --manifest-path src-tauri/Cargo.toml --test desktop_client_controls typed_dist_loopback_and_read_only_approval`
Expected: FAIL because `--studio-root` and the approval query/read-only RPC behavior do not exist.

- [ ] **Step 3: Implement the narrow host contract**

Add the optional root and host CLI controls using the existing methods, the non-mutating approval query, and read-only RPC responses. Do not expose grant/enable/approve mutation to RPC and do not alter the one-shot `prepare_terminal_command` consumption. Document the CLI's explicit operator approval, 60-second lifetime, and exact match requirement.

- [ ] **Step 4: Run focused host and adapter checks**

Run: `PATH="$HOME/.cargo/bin:$PATH" CARGO_TARGET_DIR=target/kimi-final-acceptance cargo test --manifest-path src-tauri/Cargo.toml --test desktop_client_controls && npm --prefix web/studio test -- src/adapters/desktopActions.test.ts src/components/DesktopTools.test.tsx`
Expected: all new host controls and existing desktop adapter/panel tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/src/lib.rs src-tauri/tests/desktop_client_controls.rs docs/products/desktop-client.md
git commit -m "[kimi] serve typed dist with read-only approval state (kimi-71)"
```

### Task 3: Prove the complete real-browser workflow

**Files:**
- Create: `web/studio/verify_studio_loopback.mjs`
- Modify: `web/studio/verify_studio_page_load.mjs` only if a helper must be shared without duplication
- Create task-owned browser fixture under `local/kimi/studio-final-acceptance/browser-workspace/` (uncommitted)

**Interfaces:**
- Consumes: `cdpConnect`, `launchChrome`, `trapPageLoad`, `--studio-root`, CLI grants/preauthorization, and `createDesktopActions`.
- Produces: machine-readable `LOOPBACK STUDIO ACCEPTANCE PASSED` report plus screenshot/network evidence. No production browser profile or external endpoint.

- [ ] **Step 1: Add a failing browser acceptance scenario**

Start `uor-desktop-client --serve-studio 127.0.0.1:8765 --studio-root web/studio/dist --grant <fixture>` and a fresh temporary Chrome profile. Require the mounted typed shell, workspace grant/import, two sessions and resume, retained pasted-context attachment, actual UOR `42`, native Git diff without timeout, approved `pwd` with exact cwd/output, new-file edit/save with exact disk bytes, and reopen restoring the same session/attachment/editor state. Require zero external/raw-TypeScript/runtime errors and assert page RPC cannot enable or approve itself.

- [ ] **Step 2: Run the scenario and identify remaining gaps**

Run: `node web/studio/verify_studio_loopback.mjs`
Expected: FAIL on any remaining root/approval/workflow gap; stop and repair it rather than substituting fixture invocations for real browser actions. The Task 2 host tests already provide the red-first contract, so do not manufacture a failure after that repair passes.

- [ ] **Step 3: Implement the CDP scenario and repeat it**

Use existing CDP helpers, bounded polling, inert text checks, and actual page controls. Keep the test folder disposable and explicit; close the browser and host by owned PID afterward. Inspect the same URL in the built-in browser and capture the final mounted workflow where host dialogs are not automatable through browser tools; retain the CLI approval evidence rather than inventing a dialog.

- [ ] **Step 4: Run native/static capability checks too**

Run: `node web/studio/verify_studio_loopback.mjs` and `node web/studio/verify_native_runner.mjs` with the local runner active.
Expected: full loopback browser acceptance passes; native-runner session/UOR/migration passes with its explicit host-action limitations; static-WASM keeps local-only file and model behavior with no native Git/terminal claim.

- [ ] **Step 5: Commit**

```bash
git add web/studio/verify_studio_loopback.mjs web/studio/verify_studio_page_load.mjs
git commit -m "[kimi] verify complete loopback Studio browser workflow (kimi-71)"
```

### Task 4: Repeat packaged Tauri acceptance with restored state

**Files:**
- Create task-owned verifier/evidence only under `local/kimi/studio-final-acceptance/packaged-evidence/` (uncommitted)
- Create: `docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/PACKAGED.md` (measured evidence summary)
- Modify: `scripts/studio/verify-tauri-app.sh` only if a missing assertion is found; do not weaken package checks

**Interfaces:**
- Consumes: the repaired process runner, shared dist, native Tauri IPC, and task-owned workspace.
- Produces: hash-linked packaged acceptance evidence including raw AX/screenshots, exact saved bytes, non-timeout Git output, and approved `pwd` output.

- [ ] **Step 1: Build and launch the unsigned package**

Run typed preparation, `cargo tauri build --bundles app --features gui`, and `verify-tauri-app.sh` with the computed expected executable SHA. Record the exact hash and stop any prior agent-owned app process before launching.

- [ ] **Step 2: Exercise every packaged workflow**

Through macOS accessibility: select only the task-owned folder; create/resume a session; attach pasted context and activate the current-file attachment where available; obtain exact UOR `42`; inspect Changes without timeout after the process repair; enable terminal and approve exact `pwd`; create/edit/save a native file and read its exact bytes on disk.

- [ ] **Step 3: Close, reopen, and verify restored state**

Relaunch the same executable and require the same session transcript, retained attachment, and saved editor content. Record `Artifact: unselected` if that is the actual post-reopen state rather than claiming model readiness. Capture the typed URL and hash-linked evidence manifest.

- [ ] **Step 4: Repeat focused and complete desktop checks**

Run: default/GUI desktop Rust suites and the packaged resource contract.
Expected: all pass with no false Git timeout. Preserve any genuine intermittent MCP transport failure and its focused/full rerun, rather than hiding it.

- [ ] **Step 5: Commit**

```bash
git add docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/PACKAGED.md
# Add only source/verifier changes if a concrete gap required them.
git commit -m "[kimi] record packaged Studio acceptance (kimi-71)"
```

Generated app/fixture outputs remain uncommitted.

### Task 5: Records, full acceptance, review, and protected delivery

**Files:**
- Modify: `web/studio/README.md`, `docs/products/web-client.md`, `docs/integration/current-state.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`
- Create: `docs/loop/tasks/studio-typed-shell-upgrade/kimi-71/REPORT.md`
- Evidence: `local/kimi/studio-final-acceptance/<HEAD>/`

**Interfaces:**
- Consumes: all prior task results and exact executable/package hashes.
- Produces: a truthful acceptance record with static/native-runner/Tauri capability differences, all negatives, and the separate production-publication branch left explicit.

- [ ] **Step 1: Update measured records**

Document the actual browser/loopback/Tauri workflows, restored state, process-race repair, approval boundary, and static/native limitations. Do not claim production publication, signed release, full Phase A, useful chat, M7/M8, accessibility certification, or Phase B. Include any intermittent test rerun and the exact unresolved limits.

- [ ] **Step 2: Run the complete exact-head matrix**

Run `npm test`, all plain-JS tests, typecheck, Vite build, all four legacy checks, native-runner CDP, loopback browser acceptance, typed/Tauri runtime tests, default/GUI desktop Rust suites, typed preparation/package verification, harness build, `loop report --check`, `loop pr-check --local`, and `loop gates --full`. Capture HEAD/tree/merge-base/status/diff/stat/files under `local/kimi/studio-final-acceptance/<HEAD>/`.

- [ ] **Step 3: Obtain a fresh exact-head delivery review**

Give `uor-delivery-reviewer` only the item chain, ORDER/REPORT, exact diff, and evidence logs. Repair any finding and rerun affected checks plus a fresh review. Do not push on CHANGES.

- [ ] **Step 4: Deliver through the protected path**

After APPROVE, push without force, create a draft PR with `References #3500`, post `uor-review v1` with the reviewer's agent ID, run `loop close --session`, require exact-head GitHub checks and remote policy/queue dry-run, then queue. Confirm actual MERGED state, not just acknowledgement.

- [ ] **Step 5: Verify fresh main and clean only owned outputs**

Verify squash content/tree identity on detached fresh `main`, rerun required product/legacy/browser/desktop/package/harness checks, preserve all evidence, and delete only this unit's branch/worktree. Update `local/kimi/PICKUP.md` with the separate typed-production-publication/final-launch unit. Do not invoke the production publisher or start Phase B.
