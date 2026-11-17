# LexLean MVM Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver LexLean's three-pane workbench with a real RV64IMAC Linux-userland emulator, virtual files/Git and an explicitly approved branch/edit/commit/PR workflow on web and desktop, without creating a second runtime, faking builds or exposing credentials.

**Architecture:** Vendor the pinned LexLean MVM core into a native/WASM Rust crate, then adapt it behind one lazy browser worker and local Vite `assets/*.wasm`. Compose existing UOR task/editor/assistant components into the LexLean layout with an MVM file source and revision-checked writes. Keep all model/task state in `AgentStore`/`AgentTaskStore`; MVM owns only per-tab sandbox files, virtual Git and CoW history. GitHub writes use pinned targets, base-SHA compare-and-swap and separate approvals.

**Tech Stack:** Safe Rust, native Rust + `wasm32-unknown-unknown`, existing locked `sha2`/`wasm-bindgen`, TypeScript/Preact, IndexedDB, Vite, Playwright, Tauri companion, GitHub REST API/companion `gh`.

**Spec:** `docs/superpowers/specs/2026-11-11-lexlean-mvm-workbench-design.md`

## Global Constraints

- `AgentStore` is the only in-memory task/run/event/cursor authority; `AgentTaskStore` is only its durable adapter. MVM/virtual Git is a bounded per-tab sandbox, never a second task store or model runtime.
- Pin LexLean commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` and the spec's exact file hashes. Extract only the complete 188-line vsock module. Preserve MIT/Apache-2.0/copyright/Lean-source notices and record UOR changes. Encode initramfs as hash-pinned Rust source bytes; commit no binary.
- Do not port LexLean `api.rs`, `lib.rs`, `wasm_ast_eval.rs`, `execute_cargo_json`, generated GLLM or AST-evaluated Cargo behavior. `cc` must execute genuine RV64IMAC C-to-ELF output; Cargo/rustc without a real guest toolchain report unavailable.
- No new default runtime dependency, public listener, Electron, dense matmul, softmax, page-shared MAC key or new external package. Existing locked SHA/WASM crates may be reused.
- Numeric bounds: 512 files, 1 MiB/file, 16 MiB workspace, 64 KiB/command, 2 MiB/protocol message, 1 MiB/output, 256 64-KiB CoW sectors, 256 Git commits and 200 memory-only history entries. CoW snapshots go directly from worker to IndexedDB; JSON carries only snapshot ID/digest, never the 16 MiB image. Unknown/corrupt/quota-exceeded persistence refuses explicitly. A page-lifetime Web Locks lease guards each `sessionStorage` reload key; duplicated active keys refuse or rotate explicitly, never share writes.
- Raw command history and tokens are never persisted. Secret-like terminal input is refused before worker submission. Real GitHub tokens are memory-only, never registered with the MVM and cleared on reset/dispose; engine secret tests use synthetic values only.
- The worker protocol is allowlisted; no page-callable host-secret/proxy/raw-CoW import API. Browser vsock is in-memory only with no host network.
- Every GitHub write pins `RepoTarget`, records base ref/blob SHA, uses single-flight separately approved branch/edit/commit/PR actions and refuses stale bases, cross-repo/default-branch writes and unapproved repositories. Smoke mode allowlists only a dedicated non-production repository.
- Desktop writes use only existing `AgentHost` intent/receipt approval with fixed absolute `gh api` allowlists. Tauri token entry is disabled and `src-tauri/tauri.conf.json`/CSP remains unchanged. Direct static-web token writes are UNVERIFIED unless separately exercised live.
- Static unavailable states are explicit: no fake terminal, build, GitHub success, model response or PR link. ReplayModelPort remains deterministic synthetic output, not R4 generation.
- Preserve protected selectors, mode-correct skip link, exact 1023/1024 rail behavior, keyboard/focus, CJK/RTL/IME, legacy/typed queries and 390/1024/1440 behavior. Desktop must execute MVM status/cc, not only mount the UI.
- Real R4 generation, useful-model evaluation, model-driven autonomous bug fixing, production deployment/mutation and full LexLean/Cursor parity are `NOT_RUN` and are not claimed.
- Protected delivery only: roadmap item/issue before product code, isolated worktree, RED-first tests, independent reviews, protected PR, fresh-main verification, preserved evidence and cautious cleanup. Generated roadmap views use their separate official render path.

## Review Focus

- Exact nontrivial C output, malformed ELF/C/instructions/domains/CIDs/frames and synthetic secrets must fail closed or execute correctly, never return canned status (Tasks 1–2).
- Worker exports, lazy loading, per-tab atomic persistence, corrupt/quota/version handling, command serialization and history limits must not leak secrets or silently replace state (Task 2).
- MVM file-tree source, revision-checked editor/terminal writes, stale repository bases, duplicate paths/branches and concurrent actions must not silently overwrite current state (Tasks 3–5).
- RepoTarget allowlists, exact ref/content/pull payloads, base-SHA CAS, single-flight approvals and default skipped live smoke must prevent unapproved/production writes (Tasks 5–6).
- Real desktop local-WASM execution and responsive/IME/focus behavior must work on the actual workbench, not only fixtures (Tasks 3, 7).

---

### Task 1: Native `uor-r4-mvm` engine and pinned license/provenance

**Files:**
- Create: `crates/uor-r4-mvm/Cargo.toml`, `src/lib.rs`, `src/mvm.rs`, `src/mvm_cc.rs`, `src/spiralcore_vsock.rs`, `src/mvm_initramfs.rs` (source bytes), `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`
- Modify: `Cargo.toml`, `Cargo.lock` to add the workspace member; no new external package
- Test: `crates/uor-r4-mvm/tests/mvm.rs`, `provenance.rs`

**Interfaces:**
- Consumes: pinned upstream `mvm.rs`, `mvm_cc.rs`, `mvm_initramfs.bin`, exact-output tests and only `lexlean_generated.rs:45,751-45,938` (the self-contained 188-line `r4_spiralcore_vsock` tail module); `r4_compositional_gllm` begins at line 45,940 and every other generated-file line is excluded.
- Produces: native `MvmSandbox` boot/status/exec/files/CoW/vsock APIs and `compile_c_to_rv64_elf`; secret/proxy/raw-sector test APIs are not WASM exports.

- [ ] **Step 1: Write failing tests** for all four source hashes plus initramfs bytes; `mvm_boot_reports_rv64imac_and_zero_nic`, `mvm_exec_mutates_virtual_files`, `mvm_cow_restore_is_deterministic`, upstream fib/collatz exact stdout and `EM_RISCV=243`, a second distinct C program, invalid-C no-panic diagnostics, bounded vsock/placeholders, state-changing status and invalid ELF/CID/domain/synthetic-secret refusal.
- [ ] **Step 2: Run** `cargo test -p uor-r4-mvm`; expected FAIL because the crate/behavior is absent.
- [ ] **Step 3: Vendor/adapt only the allowlisted engine** with notices and upstream commit/hashes in README. Reuse existing locked `sha2`; no new external package. Keep `forbid(unsafe_code)` and deterministic fuel. Generate `mvm_initramfs.rs` from the pinned bytes and verify its SHA-256.
- [ ] **Step 4: Run** `cargo test -p uor-r4-mvm` and `cargo fmt --check`; expected PASS including exact `FIB12=144 COLLATZ27=111 SUM=255`, `AF_INET_DENIED=-97` and a changed CoW/status digest after writes.
- [ ] **Step 5: Commit** `feat(mvm): import pinned rv64imac sandbox with provenance`.

### Task 2: WASM worker protocol, local packaging and bounded persistence

**Files:**
- Modify: `crates/uor-r4-mvm/Cargo.toml`, `src/lib.rs`, `web/studio/package.json`, `web/studio/src/stage-dist.mjs`
- Create: `crates/uor-r4-mvm/src/wasm.rs`, `web/studio/build_mvm_wasm.sh`, `web/studio/src/mvm/{protocol.ts,worker.ts,client.ts,client.test.ts,persistence.ts,persistence.test.ts}`, `web/studio/tests/e2e/mvm-terminal.spec.ts`

**Interfaces:**
- Consumes: Task 1 sandbox/compiler and existing Vite `assets/*.wasm` desktop runtime allowlist.
- Produces: bounded `{id,kind:'init'|'status'|'exec'|'read'|'write'|'snapshot'|'restore'|'compile'|'vsock'|'dispose',payload}` / `{id,ok,result|error}`; `MvmClient.status/exec/read/write/snapshot/restore/compile/vsock/dispose`; versioned per-tab persistence. `snapshot` returns `{snapshotId,digest}` after worker-to-IndexedDB storage; `restore` accepts only that handle/digest, never a full CoW image in JSON.

- [ ] **Step 1: Write failing tests** `mvm_worker_protocol_is_bounded_and_typed`, `mvm_status_and_exact_c_output_match_native`, `mvm_cow_state_survives_reload`, `mvm_snapshot_restore_respects_message_cap_at_256_sectors`, `mvm_corrupt_quota_partial_and_unknown_version_are_refused`, `mvm_tabs_are_isolated`, `duplicated_tab_gets_distinct_sandbox_or_refuses`, `mvm_dispose_rejects_later_commands`, `mvm_worker_exports_no_secret_apis`, `mvm_history_is_memory_only_and_secret_like_input_is_refused`, `mvm_worker_is_lazy_until_terminal_open`.
- [ ] **Step 2: Run** focused Vitest and the new Playwright spec; expected FAIL: worker/protocol/packaging absent.
- [ ] **Step 3: Implement WASM and one serialized worker owner** with numeric limits, request IDs and no secret/raw-CoW egress exports. Persist full CoW directly from worker to IndexedDB and return only snapshot ID/digest. Use `sessionStorage` reload keys guarded by page-lifetime Web Locks; a duplicated active key refuses with an explicit ownership state or user-chosen fresh key. Use v1 transactions and digest read-back. Raw command history stays in memory. Unknown future versions never overwrite existing state.
- [ ] **Step 4: Generate WASM/JS under ignored `web/studio/node_modules/.cache/uor-r4-mvm/` before Vite** and import the `.wasm` with `new URL(..., import.meta.url)` so Vite emits a hashed local `assets/*.wasm`. Add explicit generated-input/staging tests and zero external requests on initial page load. Do not change `src-tauri/tauri.conf.json`, `scripts/studio/runtime-files.sh` or `prepare-tauri-runtime.sh`.
- [ ] **Step 5: Run** focused Vitest, `wasm32-unknown-unknown` check/build and real browser Playwright; expected PASS for exact native/browser C output, files, vsock, reload and failure states. This is the decisive browser preflight; failure stops UI work for design revision.
- [ ] **Step 6: Commit** `feat(mvm): add bounded wasm worker and durable per-tab sandbox`.

### Task 3: LexLean three-pane workbench and revision-safe editor/MVM terminal

**Files:**
- Create: `web/studio/src/components/{LexWorkbench.tsx,MvmTerminal.tsx,LexWorkbench.test.tsx,MvmTerminal.test.tsx}`, `web/studio/src/mvm/{workspaceAdapter.ts,workspaceAdapter.test.ts}`, `web/studio/tests/e2e/mvm-workbench.spec.ts`
- Modify: `web/studio/src/components/{StudioApp.tsx,WorkspaceEditor.tsx,types.ts}`, `web/studio/src/adapters/{workflow.ts,fileSave.ts,fileSave.test.ts}`, `web/studio/src/styles/lexworkbench.css`, `web/studio/src/studio.css`

**Interfaces:**
- Consumes: `MvmClient`, existing `TaskRail`, revised `WorkspaceEditor`, assistant/task detail and `AgentClient` owner. Existing native `TerminalPanel` stays a separate companion option.
- Produces: `LexWorkbench`, `MvmTerminal`, an explicit MVM `fileSource`, and a `WorkspaceApi`-compatible adapter using serialized `saveActiveFile` path/saveId plus per-file base revisions.

- [ ] **Step 1: Write failing tests** `lexworkbench_has_three_panes_and_real_bottom_terminal`, `mvm_file_tree_lists_mvm_workspace_not_host_tree`, `mvm_editor_save_uses_active_path_save_id_and_revision`, `mvm_stale_editor_save_conflicts_with_terminal_write`, `mvm_virtual_file_open_and_save_updates_worker`, `mvm_mobile_drawers_preserve_editor_and_terminal`, `mvm_static_unavailable_state_is_explicit`.
- [ ] **Step 2: Run** focused Vitest; expected FAIL: components/adapter/file source absent.
- [ ] **Step 3: Implement layout and adapter** without a second task/runtime owner. Left MVM repository/file explorer and tasks, center editor/review, right assistant, bottom real MVM. Desktop mounts the same UI. Preserve protected selectors and mode-correct skip link.
- [ ] **Step 4: Run** focused tests plus Playwright for file open/save/conflicts, terminal state, 390/1024/1440, exact `1023_1024_rail_boundary`, keyboard/focus, `rtl`, `cjk_ime`, and `legacy_and_typed_query` regressions.
- [ ] **Step 5: Commit** `feat(studio): add lexlean workbench and revision-safe mvm terminal`.

### Task 4: Stateful virtual Git and unavailable-toolchain honesty

**Files:**
- Create: `web/studio/src/mvm/{virtualGit.ts,virtualGit.test.ts,commands.ts,commands.test.ts}`
- Modify: `web/studio/src/mvm/{client.ts,client.test.ts,persistence.ts,persistence.test.ts}`, `web/studio/src/components/{MvmTerminal.tsx,MvmTerminal.test.tsx}`

**Interfaces:**
- Produces: `VirtualGit.init/status/branch/add/diff/log/commit/checkout`, deterministic `{hash,parents,message,author,timestamp,tree}` commits and bounded serialized command results.

- [ ] **Step 1: Write failing tests** `git_init_branch_add_commit_log_are_deterministic`, `git_checkout_restores_tree`, `git_duplicate_branch_and_path_are_refused`, `git_save_and_status_agree`, `git_corrupt_history_and_cap_are_refused`, `cargo_rustc_report_unavailable_not_success`.
- [ ] **Step 2: Run** focused Vitest; expected FAIL: stateful virtual Git absent.
- [ ] **Step 3: Implement virtual Git** with canonical path ordering, SHA-256 tree/commit identities, parent links and 256-commit bound. Do not pretend virtual Git is a GitHub remote or port AST Cargo evaluation.
- [ ] **Step 4: Run** focused Vitest plus browser virtual Git/editor consistency and unavailable-toolchain checks.
- [ ] **Step 5: Commit** `feat(mvm): add deterministic virtual git history and commands`.

### Task 5: Pinned, session-approved GitHub client and `AgentHost` companion

**Files:**
- Create: `web/studio/src/agent/{githubClient.ts,githubClient.test.ts}`, `web/studio/src/components/{GitHubPanel.tsx,GitHubPanel.test.tsx}`, `src-tauri/src/agent_github.rs`, `src-tauri/tests/{agent_github.rs,agent_mvm.rs}`
- Modify: `web/studio/src/agent/{transports.ts,client.ts}`, `web/studio/src/components/LexWorkbench.tsx`, `src-tauri/src/{agent_host.rs,lib.rs}`

**Interfaces:**
- Produces: `RepoTarget`, bounded `getRepo/listDirectory/getContent/getRef/createRef/createOrUpdateContent/createPull`, repository-to-MVM seed with blob SHAs, and explicit Branch/Edit/Commit/PR states with base-SHA CAS and single-flight guards.

- [ ] **Step 1: Write failing tests** `github_public_repo_read_needs_no_token`, `github_desktop_read_uses_companion_or_unavailable`, `github_repo_target_is_pinned`, `github_production_and_unapproved_repo_are_refused_in_smoke`, `github_cross_repo_pr_and_default_branch_write_are_refused`, `github_write_requires_explicit_action`, `github_token_is_never_persisted_logged_or_registered_with_mvm`, `github_ref_content_sha_and_pr_payloads_are_exact`, `github_stale_base_is_refused`, `github_double_click_creates_one_pr`, `github_compound_payload_and_error_are_bounded`, `github_companion_uses_agent_host_receipt_and_approved_gh_without_credentials`, `github_companion_binds_selected_existing_task_run_without_synthetic_records`, `github_tauri_token_entry_is_disabled_and_csp_unchanged`.
- [ ] **Step 2: Run** focused Vitest and `cargo test --manifest-path src-tauri/Cargo.toml --test agent_github`; expected FAIL: adapter/actions absent.
- [ ] **Step 3: Implement bounded contents/refs/pulls client and repository seed** with exact owner/repo allowlist, base ref/blob SHA, conflict refusal and idempotent single-flight actions. Static token is optional and memory-only; Tauri disables token entry/direct fetch and uses companion reads or unavailable.
- [ ] **Step 4: Implement companion through existing `AgentHost`** with fixed absolute `gh`, allowed `api` methods/endpoints, host-issued opaque receipt and repo/branch/action approval preview. Bind each write to the selected existing task/run; no selection is unavailable and no synthetic AgentStore record is created. No raw page `Command`, credential output or second approval authority.
- [ ] **Step 5: Run** focused browser/default/GUI host tests and `agent_mvm` desktop packaging/worker checks; expected PASS for exact payloads, approval refusal, unchanged CSP and desktop MVM status/cc execution.
- [ ] **Step 6: Commit** `feat(studio): add pinned session-approved github workflow`.

### Task 6: Opt-in non-production smoke PR and real workbench acceptance

**Files:**
- Create: `web/studio/tests/e2e/github-smoke.spec.ts`, `docs/loop/tasks/studio-lexlean-mvm-workbench/kimi-1/{smoke-target.md,smoke-evidence.md}`
- Modify: `web/studio/src/components/{GitHubPanel.tsx,GitHubPanel.test.tsx}`

**Interfaces:**
- Consumes: Tasks 3–5 workbench, virtual Git, pinned client and `AgentHost` companion.
- Produces: committed non-production repository/ref/commit/PR identities plus real static-web browser/host and desktop-adapter evidence.

- [ ] **Step 1: Write failing smoke assertions** `github_smoke_is_skipped_by_default`, `github_smoke_creates_branch_edit_commit_and_pr`, `github_smoke_does_not_touch_default_or_unapproved_repository`, `github_smoke_has_no_secret_in_storage_logs_or_mvm`, `github_opened_repo_file_base_sha_matches_pr_diff`, `virtual_git_commit_is_labelled_local_staging_not_github_sha`.
- [ ] **Step 2: Run** default suite and opt-in smoke before implementation; expected default SKIP with zero writes and opt-in FAIL because no workflow/evidence exists.
- [ ] **Step 3: Inspect then select a dedicated non-production fork/test repository.** Record exact owner/repo and provisioning authority in committed `docs/loop/tasks/studio-lexlean-mvm-workbench/kimi-1/smoke-target.md` and reproduce identities in REPORT; neither the product nor `githubClient` may create a repository. Enable only `UOR_GITHUB_SMOKE=1` plus its exact owner/repo allowlist. Drive the real static-web browser build in Chromium through the host-approved loopback companion `gh`, separately approving branch, small non-code fixture edit, commit and PR. Do not use a model, read/print credentials, merge the PR or mutate a default/production branch.
- [ ] **Step 4: Verify actual identities through bounded `gh api`** and the real static-web workbench: repository tree seed, base ref/blob SHA, correct diff, one PR after repeated action, no secret persistence and explicit approval states. Exercise desktop companion reads/approval separately; direct Tauri/static-web token writes remain UNVERIFIED unless separately approved and observed.
- [ ] **Step 5: Commit** `test(studio): accept opt-in non-production github smoke workflow`.

### Task 7: Final local/desktop acceptance, independent review and protected delivery

**Files:**
- Create: `web/studio/tests/e2e/lexworkbench-final.spec.ts`, `docs/loop/tasks/studio-lexlean-mvm-workbench/kimi-1/{ORDER.md,REPORT.md}`
- Modify: `docs/roadmap/items/studio-lexlean-mvm-workbench.md`, `docs/start/FILEMAP.md`, `docs/products/{harness-parity.md,web-client.md,desktop-client.md}`, `docs/integration/current-state.md`, `web/studio/README.md`
- Official separate render path only: `docs/roadmap/README.md` and rendered STATE/HANDOFF; never hand-edit them in this implementation PR.
- Test: all new MVM/Studio/host suites and existing Studio suites.

**Interfaces:**
- Produces: final acceptance logs, coverage matrix, exact-head independent approval, protected merge and fresh-main verification.

- [ ] **Step 1: Add final failing acceptance checks** `final_mvm_native_wasm_parity`, `final_desktop_mvm_status_and_cc_execute`, `final_workbench_accessibility_ime_rtl_and_query_modes`, `final_1023_1024_rail_boundary`, `final_virtual_git_and_editor_revision_consistency`, `final_github_repo_pin_cas_approval_and_secret_boundary`, `final_live_smoke_is_skipped_by_default`, `final_worker_is_lazy_and_page_load_has_no_external_requests`, `final_default_build_has_no_studio_test_api`, `final_coverage_rows_are_reconciled`.
- [ ] **Step 2: Run** full Vitest, all default Playwright suites, native MVM/WASM tests, Rust contract tests, default/GUI Tauri variants, typecheck/test/default builds, actual desktop window MVM status/cc, `uor-harness check`, full documented gates, `loop report --check` and `loop pr-check --local`. Expected failures only for real defects or explicit `NOT_RUN` rows; a mount-only desktop check is insufficient.
- [ ] **Step 3: Fix only proven defects and rerun** with independent spec/quality/security/complexity review. Reconcile every coverage row to an actual test or `NOT_RUN`/`UNVERIFIED` with reason. Update the three product documents to supersede the old GitHub-refused boundary only for the verified companion path; keep direct static token writes UNVERIFIED unless exercised.
- [ ] **Step 4: Commit** `test(studio): final lexlean mvm workbench acceptance`, push, obtain exact-head approval and required GitHub checks, use the protected merge queue, then confirm actual merge.
- [ ] **Step 5: Verify fresh main** for merge identity, changed-file bytes/patch identity and required fresh-main checks. Deliver generated roadmap/status views only through their official separate path. Preserve unique logs/notices before deleting only this lab's delivered branch/worktree; run `uor-hygiene --apply` and clean only identified reproducible inactive artifacts.
