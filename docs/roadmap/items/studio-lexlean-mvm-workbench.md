---
id: studio-lexlean-mvm-workbench
title: Import the LexLean MVM workbench and approved GitHub workflow
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [studio-cursor-hosted]
done_class: host
scope: [Cargo.toml, Cargo.lock, crates/uor-r4-mvm/**, web/studio/package.json, web/studio/package-lock.json, web/studio/playwright.config.ts, web/studio/index.html, web/studio/build_mvm_wasm.sh, web/studio/src/bootstrap.ts, web/studio/src/stage-dist.mjs, web/studio/src/studio.css, web/studio/src/agent/**, web/studio/src/state/agent.ts, web/studio/src/state/agent.test.ts, web/studio/src/styles/**, web/studio/src/hooks/**, web/studio/src/components/**, web/studio/src/adapters/composerInput.ts, web/studio/src/adapters/fileSave.ts, web/studio/src/adapters/fileSave.test.ts, web/studio/src/adapters/workflow.ts, web/studio/src/mvm/**, web/studio/tests/e2e/**, web/studio/README.md, src-tauri/Cargo.toml, src-tauri/Cargo.lock, src-tauri/src/lib.rs, src-tauri/src/main.rs, src-tauri/src/agent_data.rs, src-tauri/src/agent_host.rs, src-tauri/src/agent_server.rs, src-tauri/src/agent_github.rs, src-tauri/tests/agent_github.rs, src-tauri/tests/agent_mvm.rs, src-tauri/tests/agent_data.rs, src-tauri/tests/agent_authority.rs, src-tauri/tests/agent_loopback.rs, src-tauri/capabilities/default.json, docs/loop/tasks/studio-lexlean-mvm-workbench/**, docs/roadmap/items/studio-lexlean-mvm-workbench.md, docs/roadmap/issues.json, docs/start/FILEMAP.md, docs/superpowers/specs/2026-11-11-lexlean-mvm-workbench-design.md, docs/superpowers/plans/2026-11-11-lexlean-mvm-workbench.md, docs/products/harness-parity.md, docs/products/web-client.md, docs/products/desktop-client.md, docs/integration/current-state.md]
---

## Why it matters

The owner wants the useful LexLean IDE experience in UOR Studio: repository/file explorer, editor, right-side assistant and a working bottom MVM terminal on web and desktop. This must be a real execution environment and an approved path from a project change to a GitHub PR, not another decorative terminal or model-capability claim.

## What is missing

The current Studio has a durable browser task runtime and optional companion, but its static terminal is unavailable and the presentation is not LexLean's three-pane IDE. The existing editor's tree is host-owned and cannot list MVM files; the MVM WASM also needs explicit local packaging and desktop execution evidence. Missing are the real browser MVM sandbox, virtual files/Git, RV64 C compilation, revision-safe editor/terminal writes, bounded CoW persistence, repository seeding and an approved GitHub branch/edit/commit/PR flow that pins its target and protects stale bases and credentials.

The accepted M1 artifact is prediction-only and the ReplayModelPort fixture is synthetic. A future generation-capable model must enter through the existing ModelPort boundary. No real generation, useful-model evaluation, production service or full LexLean/Cursor parity is authorized by this item's local implementation.

## Done when

- A `uor-r4-mvm` crate compiles to native Rust and WASM with pinned LexLean provenance/license notices and no committed binary. Exact-output tests boot RV64IMAC, execute commands, mutate virtual files, restore CoW state, compile at least two C programs to genuine RV64 ELF and run them, deliver bounded in-memory vsock frames and reject invalid C/ELF/CIDs/domains/secrets without panic.
- A responsive three-pane workbench uses the existing `AgentStore`, `AgentClient`, serialized editor save adapter and task actions with no second runtime. `WorkspaceEditor` lists MVM rather than host files, and revision checks reject stale editor/terminal writes. Web and desktop share the UI/worker, with real desktop `mvm status`/`cc`, accessible 390/1024/1440 layouts, exact 1023/1024 behavior, protected selectors/skip link and keyboard/CJK/RTL/IME coverage.
- The lazy browser MVM worker supports virtual files, stateful virtual Git, real C build/execution and bounded CoW persistence under numeric file/byte/message/history limits. CoW images move directly between worker and IndexedDB; JSON carries only snapshot ID/digest within its cap. Web Locks prevents duplicated-tab shared writes. Cargo/rustc without a genuine guest toolchain report unavailable. Transactional reload, corruption/quota/version refusal and duplicated-tab ownership pass; raw command history and GitHub tokens are not persisted, secret-like terminal input is refused, and no secret API is page-callable.
- Bounded repository directory/content reads seed the MVM with remote blob identities. GitHub writes pin an approved `RepoTarget`, use base ref/blob SHA compare-and-swap and single-flight separately approved branch/edit/commit/PR actions. Desktop reads/writes use only `AgentHost`-approved companion `gh` under unchanged CSP or report unavailable; writes bind the selected existing task/run and create no synthetic task records. Static token entry alone authorizes nothing. Cross-repository/default-branch/unapproved-target writes refuse.
- The default suite never performs a live GitHub write. An explicitly enabled smoke on a dedicated non-production fork/test repository drives the real static-web browser through the host-approved loopback companion `gh`, separately exercises desktop companion approval, and retains actual repo/ref/commit/PR identities in the committed task evidence/REPORT. It does not touch a default branch and leaves the PR open. Direct static-web token writes remain UNVERIFIED unless separately exercised. Unavailable credentials/network produce honest states.
- Independent task and exact-head reviews, required GitHub checks, protected merge and fresh-main verification complete. Evidence identifies every executed check and every `NOT_RUN`/`UNVERIFIED` row, including real generation, useful-model evaluation, production deployment and full parity.

## Cancel if

The pinned LexLean engine cannot pass its exact-output native/browser preflight without canned behavior, or the base already supplies the complete item. Missing hooks/tests/presentation alone is not cancellation. Any need for a second runtime, model/API change, public unauthenticated service, page-shared MAC key, default-branch write, broadened desktop CSP, committed binary or production capability claim requires a new order/scope decision rather than silent expansion.

## How to start

Read the approved [design](../../superpowers/specs/2026-11-11-lexlean-mvm-workbench-design.md), [implementation plan](../../superpowers/plans/2026-11-11-lexlean-mvm-workbench.md), the completed `studio-cursor-hosted` item and the [parent registration record](../../loop/tasks/studio-ide-harness/kimi-20/ORDER.md). Verify the pinned source hashes before adapting them. Register the item through official roadmap sync, then read the active ORDER under `docs/loop/tasks/studio-lexlean-mvm-workbench/` at the merged item base; do not recreate a self-item registration ORDER. Use a fresh isolated worktree and red-first tests; Tasks 1–2 must prove native/browser MVM behavior before UI work. Import only `lexlean_generated.rs:45,751-46,087` for the self-contained vsock module; do not port any other generated line, AST Cargo evaluator, generated application/model or secret APIs.

## Rules that apply

- Do not edit `crates/uor-r4-api/**`, model/engine/tokenizer implementations, `services/**`, `.github/**`, rendered STATE/HANDOFF, `src-tauri/tauri.conf.json` or another lab's work. Generated roadmap views use only their official separate render path.
- Keep `AgentStore` as sole task/run/event/cursor owner. MVM/virtual Git state is a bounded per-tab sandbox, never a second task store or model runtime. Browser vsock is in-memory only.
- No new default runtime dependency, Electron, public listener, dense matmul/softmax, new external package or page-shared MAC key. `Cargo.toml`/`Cargo.lock` add the workspace member; existing locked SHA/WASM support crates may be reused without adding an external package.
- GitHub tokens are memory-only, never registered with the MVM and never persisted/logged. Secret tests use synthetic values. Companion writes use `AgentHost` authority and a fixed `gh api` allowlist, never raw page commands.
- Preserve LexLean copyright/license/Lean-source notices and all unique evidence. No default-branch/unapproved-repository mutation, production publication, paid compute or capability claim beyond executed checks. No destructive cleanup of unique evidence; standard `uor-hygiene --apply` is limited to identified reproducible inactive artifacts.

## Sources

- Owner request to copy LexLean's design and integrate a fully working MVM terminal and GitHub project workflow, including desktop matching web (Kimi chat, 2026-11-11).
- [Hosted LexLean workbench](https://casey-allard.github.io/lexlean-r4/) with live `mvm status` evidence: RV64IMAC, CID 3, AF_INET refusal, AF_VSOCK and CoW sectors.
- LexLean commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` and exact source hashes recorded in the approved design.
- The completed Cursor-hosted Studio browser runtime and optional Tauri companion.
