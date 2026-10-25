---
id: infra-desktop-client-tauri
title: Build the Tauri desktop client against the synthetic stand-in
milestone: infra
area: serving
status: done
open_to: anyone
needs_gpu: false
depends_on: [m7-studio-connect-wasm-engine]
scope: [src-tauri/**, web/**, crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/products/**, Cargo.toml, docs/labs/infra-desktop-client-tauri/**]
---

## Why it matters

It is the third product the owner defined on 19 October: the same interface as the web client, running the model natively on your own laptop, with durable memory and offline use. It sits at the serving end of the project, after the model file is built. No trained model exists, so this work runs against the synthetic stand-in. See [the desktop client specification](../../products/desktop-client.md).

## What is missing

The desktop client exists only as a written specification: a Tauri app sharing the web interface, running the model natively with durable memory and offline use on M1-class laptops. It is a defined product but is not part of any M1 to M8 bar in these sources. Per the specification:

- No `src-tauri/` project exists. The Studio page `web/studio/index.html` already calls `native_read_file`, `native_write_file`, `native_git_status`, `native_run_terminal_command` and `native_list_local_models` when `window.__TAURI__` is present, but nothing in the repository defines them.
- The engine, `crates/uor-r4-integer`, is a library and a CLI verified only on test and synthetic models.
- `crates/uor-r4-object-store` is a tested library that no crate uses yet. Storing session snapshots in it is a design note, not done.
- That native is faster than WebAssembly is a hypothesis. No decode speed exists for any D11 artifact, so no speed bar is registered.

## Done when

A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a trained model exists, "runs the engine" means the synthetic stand-in.

## How to start

1. Read `docs/products/desktop-client.md` (goals 1–5, the component table and the data flow), `docs/products/README.md`, and the Tauri hooks in `web/studio/index.html` (`grep -n "__TAURI__\|native_" web/studio/index.html`). Check where `m7-studio-connect-wasm-engine` stands, so the web and desktop clients keep one shared interface.
2. Post a design note on the issue first. It must give: where `src-tauri/` lives and whether it joins the root `Cargo.toml` workspace; the command list and its argument types; how file access is limited to folders the user grants; how the engine is linked as a library (the same code as the `uor-chat` program); how the model manager checks the `UORLUT01` file's SHA-256 against its pin before it passes the bytes to the structural reader in `crates/uor-r4-integer/src/stack/format.rs` (that reader checks offsets and lengths, not the hash); what `native_write_file` and `native_run_terminal_command` do, since the specification wants the terminal command off by default, shown to the user before it runs and limited to a granted folder (its open question 3); and why the app adds no softmax or dense matrix multiplication (the engine is integer adds, shifts and look-ups; `scripts/audit_zero_matmul_serving.py` is the existing serving audit).
3. Write a failing test first. Keep each command a plain Rust function, so it is tested on the host without a window. A first test: `native_list_local_models` returns the synthetic stand-in's file and refuses a file whose hash differs from its pin. Find the stand-in with `grep -rln -i synthetic crates/uor-r4-integer/src`. Run `~/.cargo/bin/cargo test -p <tauri-crate>`.
4. Implement in order: model manager and engine call; token streaming to the page; scoped file commands; durable memory through `crates/uor-r4-object-store` with a `UR4SESS1` snapshot. Test close and resume token for token on the synthetic stand-in. That is only a check of the restore path, not the M3 bar.
5. Check offline use with the network off. Only the optional update check may use the network. The shared Studio page currently loads CDN assets and sends web-search queries to Wikipedia and DuckDuckGo (`docs/products/web-client.md`), so the desktop build must self-host every asset and drop or gate web search before this check can pass. Do not time native
 against WebAssembly in a way that claims a speed; the specification leaves that open.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m4-serving.md`, and the restore row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:67-77`](../../../docs/start/where-were-going.md)
