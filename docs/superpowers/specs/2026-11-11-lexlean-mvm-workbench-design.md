# LexLean MVM workbench design — 2026-11-11

## Intent

Import the useful parts of the hosted LexLean IDE into UOR Studio: a responsive three-pane workbench with repository/file explorer, editor, right-side assistant, and a real bottom RV64IMAC Linux-userland emulator (MVM) terminal. The result must let a user inspect and change a project, run commands, and take an explicitly approved project change through GitHub branch, commit and PR on web and desktop.

This is not a visual-only clone, a hardware-hypervisor claim, a new model runtime, or a claim that the unfinished UOR model can fix bugs. The current deterministic replay fixture remains labelled; a future generation-capable R4 model must use the existing `ModelPort` boundary. The required live smoke is a small human-approved fixture edit, not model-driven bug fixing.

## Observed source and constraints

The public [LexLean page](https://casey-allard.github.io/lexlean-r4/) provides the target layout and a working `mvm status` path: RV64IMAC, guest CID 3, `AF_INET` refusal, `AF_VSOCK = 40` transport and copy-on-write sectors. Its shell exposes virtual files, Git state, Rust-check labels, an RV64 C compiler and vsock support. Only the live status path was exercised during scouting; compilation was not claimed as observed there.

Reusable source is pinned to `Casey-allard/lexlean-r4@01956d5c35f268fc7d1b646d97d3dff21b9e945f`:

| Upstream file | Bytes | SHA-256 |
|---|---:|---|
| `crates/wasm/src/mvm.rs` | 98,525 | `2789ec57daa7f92dbd094e5d2c38a8ae343b4f38bd09cd2ed6bc377fa7d67206` |
| `crates/wasm/src/mvm_cc.rs` | 75,765 | `19390475442bbede4332bf38776706d42aa09e2daa9df4e014d140c6af5ce12f` |
| `crates/wasm/src/mvm_initramfs.bin` | 12,164 | `071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c` |
| `crates/wasm/src/lexlean_generated.rs` | 2,206,963 | `acc9a89d65bb43bdb61a720fd90570dacf46a43adb9340fcbc427a9245e247e8` |
| `crates/wasm/tests/mvm_spiralcore_sandbox.rs` | 9,573 | `9b99e2d64f32475decc5c10db658677a4b9b211f8d6963e3668fbb471a653c7b` |

Extract only the self-contained 188-line `r4_spiralcore_vsock` tail module from `crates/wasm/src/lexlean_generated.rs:45,751-45,938`; that generated file is the module's only upstream source, and `r4_compositional_gllm` begins at line 45,940 and is excluded. Do not import any other line of the generated 2.2 MB application/model file. Preserve MIT/Apache-2.0, copyright and Lean-source notices and mark UOR modifications. Do not port LexLean's `api.rs`, `lib.rs`, `wasm_ast_eval.rs`, `execute_cargo_json`, generated GLLM application/model, or its AST-evaluated Cargo success path.

UOR constraints remain binding: `AgentStore` is the only in-memory task/run/event/cursor authority; `AgentTaskStore` is its durable adapter. Do not create a second runtime or model owner. Do not change model/engine/tokenizer code, `crates/uor-r4-api/**`, `services/**`, `.github/**`, rendered STATE/HANDOFF or another lab's work. No page-shared MAC key, public unauthenticated listener, dense matmul, softmax or new default runtime dependency. Existing locked `sha2`/`wasm-bindgen` crates may be reused without adding a new lock entry.

## Approaches considered

### A — Shared UOR runtime with a vendored MVM engine and adapter (chosen)

Vendor the MVM engine into a new `uor-r4-mvm` Rust crate with original license notices and UOR-specific changes. Compile it to native Rust and WASM, then put a thin command adapter behind one lazily started browser worker. Reuse the existing Studio runtime, components and protected selectors while introducing the LexLean three-pane layout.

Advantages: real MVM behavior on static web and desktop, no second model/task authority, no dependency on LexLean's generated application or model, and independently testable CPU/syscall boundaries. Costs: a new crate and command-protocol adapter must be maintained and checked against upstream.

### B — Fork LexLean's whole public application

Rejected: it would duplicate or displace the reviewed AgentStore/runtime, model port, persistence, approval and task UI work, import prohibited generated mechanisms, and make GitHub writes harder to secure.

### C — Copy only the visual layout and use the existing native PTY

Rejected: it does not provide the requested working MVM on static web and leaves the terminal unavailable without a companion. The native PTY remains a separate explicitly approved desktop option.

### D — Companion-only GitHub without a browser token client

Rejected as the sole implementation: it is simpler and is the only desktop write path, but static hosted web also needs an explicit memory-only token option. The live smoke uses the companion; direct static-web token writes remain `UNVERIFIED` unless separately exercised and are not claimed from mocks.

## Architecture

### MVM engine and worker

- New `crates/uor-r4-mvm` crate with `mvm.rs`, `mvm_cc.rs`, `spiralcore_vsock.rs` and hash-pinned `mvm_initramfs.rs` source bytes. No compiled binary is committed; a test reproduces the upstream initramfs SHA-256.
- Public native APIs cover boot/status, command execution, guest file read/write/export, CoW snapshot/restore, RV64 C compilation and vsock packet handling. `forbid(unsafe_code)`, deterministic instruction fuel, and explicit malformed ELF/CID/domain/frame/secret refusal are mandatory.
- Native tests port the upstream exact-output fib/collatz case (`FIB12=144 COLLATZ27=111 SUM=255`, `compiled_elf_machine=243`, `AF_INET_DENIED=-97`), plus a second distinct C program, invalid-C/malformed-input no-panic refusal and state-changing status assertions.
- The same source compiles to WASM. A dedicated lazy worker owns one sandbox per tab and exposes only `init/status/exec/read/write/snapshot/restore/compile/vsock/dispose` in a bounded JSON protocol. Browser vsock is in-memory only and has no host network egress.
- `register_host_secret`, `proxy_vsock_egress`, `guest_ram_contains` and raw CoW-sector import are not page-callable exports. Native tests use synthetic secrets only. Real GitHub credentials are never registered with the MVM.
- IndexedDB `uor-r4.studio.mvm`, version 1, persists bounded virtual files/revisions, virtual Git, cwd and validated CoW snapshots. `snapshot` writes CoW directly from the worker to IndexedDB and returns only `{snapshotId,digest}`; `restore` accepts that handle/digest. CoW images never traverse the JSON protocol. The store does not persist raw terminal command lines, tokens or task state. Terminal history is memory-only and capped at 200 entries; secret-like command input is refused before worker submission.
- Numeric bounds are explicit: 512 files, 1 MiB/file, 16 MiB total workspace, 64 KiB/command, 2 MiB/protocol message, 1 MiB/output, 256 64-KiB CoW sectors and 256 Git commits. IndexedDB writes are transactional with digest read-back; quota, partial-write, corrupt-state and unknown-version failures are explicit. A random non-secret key in `sessionStorage` is a reload hint guarded by a page-lifetime Web Locks lease. A duplicated tab whose copied key is already leased gets an explicit ownership refusal and may choose a fresh sandbox key; tabs never write the same sandbox concurrently and no silent migration occurs.

### Workbench presentation

- `LexWorkbench` composes the existing `TaskRail`, task detail/assistant and new `MvmTerminal` with a revised `WorkspaceEditor`. The existing native `TerminalPanel` remains a separate companion option.
- `WorkspaceEditor` gains an explicit MVM `fileSource`; the left repository/file tree lists MVM workspace files rather than `AgentClient.hostCapabilities()/treePage()` host files. Center: file tabs/editor/review. Right: assistant/model controls/task transcript. Bottom: real MVM terminal.
- Existing `AgentStore`, `AgentClient`, save adapter and task actions remain the only task paths. Existing protected selectors and mode-correct skip link remain in use.
- Every virtual file has a revision. Terminal writes increment it; editor saves include their base revision and the worker refuses stale saves. The existing serialized `saveActiveFile` path/saveId checks still apply, but are not treated as cross-writer revision checks.
- Mobile collapses explorer/assistant into labelled drawers while preserving editor and terminal. Exact 1023/1024 rail behavior, keyboard focus, CJK/IME, RTL, legacy/typed query modes and target-size acceptance remain mandatory.
- Desktop mounts the same Preact UI and lazy MVM worker. Before Vite, the build generates JS/WASM under ignored `node_modules/.cache/uor-r4-mvm/`; `new URL(..., import.meta.url)` makes Vite emit a local hashed `assets/*.wasm`, which the existing typed runtime allowlist already accepts. Acceptance requires real desktop `mvm status` and `cc` execution, not merely mounting the UI.

### Virtual files, repository seeding and Git

- Terminal commands operate on real sandbox state: `help`, `pwd`, `cd`, `ls`, `cat`, `write`, `patch`, `mkdir`, `rm`, `mv`, `cp`, `git`, `cc`, `mvm`, `vsock`, `status`, `history`, `clear` and `reset`.
- A bounded GitHub repository directory/content read seeds the MVM workspace and records each remote blob SHA. This makes the explorer and subsequent PR diff refer to the selected project rather than an unrelated fixture. Limits and explicit truncation refusal apply.
- Virtual Git supports init, status, branch list/create/switch, add, diff, log, commit and checkout with canonical paths, SHA-256 identities, parent links and bounded history. The UI labels every SHA-256 virtual commit `local staging`, distinct from the GitHub commit created later by `gh api`; it does not pretend to be a GitHub remote.
- `cc` executes the real sandbox C compiler and runs its RV64 output. `cargo check/test` and `rustc` return explicit unavailable states unless a genuine guest Rust toolchain is supplied by a separate future item; no AST evaluator or canned success is ported.

### GitHub workflow

Use an explicit state machine: `disconnected → repository_ready → branch_ready → edit_ready → commit_ready → pr_ready`.

- A session `RepoTarget` pins exact owner/repository before any network or CLI call. Smoke configuration allowlists only the dedicated non-production repository. Other user-selected repositories require a new explicit target selection. Cross-repository PRs and every default-branch write are refused; smoke mode refuses production repositories even on non-default branches.
- Public repository metadata/directory/content reads need no token where CORS permits. Static web may use a memory-only token for writes; token entry alone authorizes nothing. Direct static-web token writes are `UNVERIFIED` until a live explicitly approved token smoke occurs. Desktop reads use companion `gh` or show unavailable; the page never fetches GitHub directly under Tauri's CSP.
- Desktop writes use only companion `gh` through the existing `AgentHost` intent/receipt authority, bound to the currently selected existing task/run; with none selected the action is unavailable and no synthetic AgentStore record is created. Tauri token entry is disabled; the unchanged CSP is not broadened. The companion uses a fixed absolute `gh` executable, an `api` subcommand/method/endpoint allowlist and host approval preview containing repo/branch/action. No raw page command or credential output is allowed.
- Branch, edit, commit and PR each require a separate explicit button/action and single-flight guard. Record base ref head and content blob SHA; create the branch from that head and send the existing blob SHA on update. A 409/stale base or changed revision refuses rather than overwriting. Double-click/retry cannot create duplicate refs, commits or PRs.
- Session tokens are memory-only and never enter localStorage, IndexedDB, MVM guest RAM, logs, event history or source. They are never registered with the MVM and are cleared on reset/dispose. A user-typed secret-like terminal line is refused and never persisted; no claim is made about arbitrary unknown secret formats.
- The only live smoke target is a dedicated non-production fork/test repository selected after inspecting available targets. Record its exact owner/repo and provisioning authority in the committed implementation task's `smoke-target.md` and reproduce all identities in its REPORT; neither the product nor `githubClient` may create a repository. The live write smoke drives the real static-web browser build through the host-approved loopback companion `gh`; desktop separately exercises the same host approval path. Direct static-web token writes are not required for this web-companion evidence and remain UNVERIFIED unless separately exercised. Default CI uses mocks and skips live smoke unless both `UOR_GITHUB_SMOKE=1` and an exact repository allowlist are present. Keep the smoke PR open; do not merge it or write `UOR-Foundation/uor-r4:main`.
- Without credentials/companion/network, writes show explicit unavailable states. The product web/desktop/parity documents must describe the new bounded companion capability and the remaining direct-token verification limit.

## Acceptance and negative boundaries

Required positive evidence:

- Native and real browser-WASM parity for exact C output, state-changing status/files, CoW restore and in-memory vsock; malformed input refusal and no page-callable secret APIs.
- Real desktop worker `mvm status` and `cc`, local WASM packaging under the unchanged CSP, and companion-host approval tests.
- MVM file-tree listing, repository seed/blob identity, editor save/revision conflicts, deterministic virtual Git, bounded/corrupt/quota/version persistence and memory-only history.
- Responsive 390/1024/1440 layouts, exact 1023/1024 boundary, protected selectors/skip link, keyboard/CJK/RTL/IME, lazy worker and zero page-load external requests.
- Public GitHub metadata tests and an approved live smoke on a dedicated non-production repository: the real static-web browser uses host-approved loopback `gh` for branch, edit, commit and PR; desktop companion approval is exercised separately. Committed evidence retains all identities, base-SHA protection and single-flight behavior with no default-branch or unapproved-repository writes.

Explicit `NOT_RUN`/`UNVERIFIED`: real R4 generation, useful-model evaluation, model-driven autonomous bug fixing, production deployment/repository mutation, public hosted service, direct static-web token writes unless separately exercised, admin/tenant/secret services, genuine guest Rust toolchain support and full Cursor/LexLean parity. These are not silently converted into claims by green fixtures or a smoke PR.

## Delivery and reviews

Register the roadmap item and issue before product code. Task 1/2 are a decisive native/browser MVM preflight; failure stops UI work for design revision rather than substitution of canned behavior. Use an isolated implementation worktree, red-first tests, independent design/implementation/exact-head reviews, protected PR delivery and fresh-main verification. Preserve unique logs and third-party notices. Clean only reproducible inactive artifacts after preserving evidence.
