---
id: infra-desktop-client-tauri
title: Build the Tauri desktop client against the synthetic stand-in
milestone: infra
area: serving
status: in_progress
open_to: anyone
needs_gpu: false
depends_on: [m7-studio-connect-wasm-engine]
scope: [src-tauri/**, web/**, crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/products/**, Cargo.toml, docs/labs/infra-desktop-client-tauri/**]
---

## Why it matters

It is the third product the owner defined on 19 October: the same interface as the web client, running the model natively on your own laptop, with durable memory and offline use. It sits at the serving end of the project, after the model file is built. No qualified trained desktop/product bundle exists, so this work runs against the synthetic stand-in. See [the desktop client specification](../../products/desktop-client.md).

## What is missing

The shared web/Tauri harness now exists and passes host, protocol and packaged-resource checks: one asset-origin UI and backend, a host-mediated `uor-chat` sidecar, trusted folder/model grants, bounded Git/terminal, browser and native MCP, durable object-store sessions, and a launchable macOS `.app`. These are synthetic protocol and host results, not useful-chat or production qualification.

- **Native GUI acceptance has passed at development scope.** The real app showed synthetic `uor-chat` streaming and exact usage, native consent sheets, same-named file isolation across two projects, and both MCP transports returning their exact fixture results. Cancel and normal-exit cleanup passed; a real restart cleared credentials while preserving chat text. The shared toolbar now has working section history, session selection and direct harness actions.
- Exact sidecar hidden-state resume, forced-kill child cleanup, signed distribution, production readiness, M3/M7/M8 completion and native-versus-WASM speed claims remain open. Cmd+R did not reload the page, so it is not used as credential-clearing evidence. GitHub authentication is evaluated in the design record but is not implemented; static Pages has no established direct CORS path.
- Independent review fixes now cover platform-specific asset origins, slow partial streams, async MCP, page-unique cancellation, terminal/Git/filter confinement including includes/worktree configuration and mutation serialization, static canonical-path containment, idempotent process-group termination, atomic request registration/cancellation, readiness/cancellation, UTF-8 errors, MCP failed-list and transactional replacement cleanup, and durable manifest atomicity/collisions. Root-aware recent-file identity, concurrent-open deduplication and initial deep-link history were added after real GUI regressions. Exact-head re-review, protected delivery, fresh-main verification and hosted publication remain open; 25 GiB free is available after targeted cleanup, short of the 30 GiB target.

## Done when

A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a qualified trained desktop/product bundle exists, "runs the engine" means the synthetic stand-in.

## How to start

1. Read `docs/products/desktop-client.md` (goals 1–5, the component table and the data flow), `docs/products/README.md`, and the Tauri hooks in `web/studio/index.html` (`grep -n "__TAURI__\|native_" web/studio/index.html`). Check where `m7-studio-connect-wasm-engine` stands, so the web and desktop clients keep one shared interface.
2. Post a design note on the issue first. It must give: where `src-tauri/` lives and whether it joins the root `Cargo.toml` workspace; the command list and its argument types; how file access is limited to folders the user grants; how the engine is linked as a library (the same code as the `uor-chat` program); how the model manager checks the `UORLUT01` file's SHA-256 against its pin before it passes the bytes to the structural reader in `crates/uor-r4-integer/src/stack/format.rs` (that reader checks offsets and lengths, not the hash); what `native_write_file` and `native_run_terminal_command` do, since the specification wants the terminal command off by default, shown to the user before it runs and limited to a granted folder (its open question 3); and why the app adds no softmax or dense matrix multiplication (the engine is integer adds, shifts and look-ups; `scripts/audit_zero_matmul_serving.py` is the existing serving audit).
3. Write a failing test first. Keep each command a plain Rust function, so it is tested on the host without a window. A first test: `native_list_local_models` returns the synthetic stand-in's file and refuses a file whose hash differs from its pin. Find the stand-in with `grep -rln -i synthetic crates/uor-r4-integer/src`. Run `~/.cargo/bin/cargo test -p <tauri-crate>`.
4. Implement in order: model manager and engine call; token streaming to the page; scoped file commands; durable memory through `crates/uor-r4-object-store` with a `UR4SESS1` snapshot. Test close and resume token for token on the synthetic stand-in. That is only a check of the restore path, not the M3 bar.
5. Check offline use with the network off. All page-load assets are self-hosted. The host starts the loopback `uor-chat` sidecar automatically; external MCP connections require explicit user action and native confirmation, while custom providers and remote model URLs remain browser-only. Do not time native against WebAssembly in a way that claims a speed; the specification leaves that open.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m4-serving.md`, and the restore row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:67-77`](../../../docs/start/where-were-going.md)
