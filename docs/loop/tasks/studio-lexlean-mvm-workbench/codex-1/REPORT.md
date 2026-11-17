# REPORT for the local native MVM component

Lab: codex. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md). [ORDER](ORDER.md). References #3939 and #3941.

## What did not work

The previous item named lines 45,751–46,087 of LexLean's generated file. The vsock module ends at 45,938. Line 45,940 begins a generated model module outside the workbench scope. The complete draft also trusts page-supplied GitHub repository and default-branch metadata at the native boundary. Its static companion path cannot approve writes and the GitHub page client has no production caller. The [exact-head maintainer review](https://github.com/UOR-Foundation/uor-r4/pull/3941#issuecomment-6957713047) records the target and branch defects.

## What changed

Codex adopted Kimi's native import `138f473eaa0628322ce598da8cd447dababbc814` and provenance correction `fd049b00d91c5219541128ac41d0d11b59cb8312`. The [full draft head](https://github.com/UOR-Foundation/uor-r4/commit/f842ad133debeaf1a6641b2f248154ae9e1754a0) preserves all of Kimi's subsequent work. The existing #3939 scope covers the new Rust package, FILEMAP row, CLI and corrected source references. The item continues to be ready and open.

The [native entry point](../../../../../crates/uor-r4-mvm/README.md) boots an ephemeral guest and runs a bounded Collatz workload. Given N from 1 to 512, it compiles the checked-in C program to a RISC-V ELF and interprets it. At 27 the first longest sequence starts at 27 and takes 111 steps. At 512 the result is 327 and 143. The CLI checks the ELF architecture and the retired-instruction count before printing the guest's output. The public API exposes guest identity and the bounded numeric computation. The private compiler's recursion and declared-size allocations occur before the guest's 8 MiB RAM and 25-million-instruction limits. The fixed guest workload is the sole source input on the product path.

## Measurements

Compared the adopted bytes directly with the original Git objects at [LexLean 01956d5c](https://github.com/Casey-allard/lexlean-r4/tree/01956d5c35f268fc7d1b646d97d3dff21b9e945f). The initial import changed two module paths. Reversing the checked-in emulator patch now reconstructs all 98,525 upstream bytes and their pinned SHA-256. The 75,765-byte compiler, 188-line vsock slice, decoded 12,164-byte initramfs, and both original license texts matched byte for byte. The crate README records SHA-256 for each source input. Cargo.lock adds one workspace member using the workspace's existing locked SHA-256 crate.

On Darwin arm64, Rust 1.97.1, the final package suite contains 16 private parser, emulator, compiler and provenance cases, two public API cases, three executable-level cases and two compile-fail examples that enforce private access. Two guest programs emitted exact stdout, empty stderr and a successful exit after executing RV64 instructions. The CLI tests independently computed the expected stopping-time winner for six bounded arguments, including both endpoints, and rejected invalid or unsupported commands. The remaining native cases cover a guest-file mutation, deterministic in-memory CoW restore, bounded in-memory vsock, invalid C, ELF, CID and IP-domain refusal. The secret-policy fixture used a synthetic placeholder.

The first independent review of `4c2e645` found that the ELF loader consumed unchecked table entries, the snapshot omitted its allocator cursor and vsock accepted padding. The repaired loader shares the parser's checked file and RAM ranges. A parser-only test covers incomplete and out-of-range tables. Replaying ordinary file writes after restoring on the same guest and a fresh guest reproduced both file contents and the exported sector sequence. The receiver delivered exactly 65,536 bytes and rejected a padded frame without queuing it. The sender checks the original length before conversion.

A second independent review of `d48a647` verified those three fixes and reproduced five defects in the original general-purpose interfaces. The top session adopted a narrower public contract. Both the Rust API and CLI accept only identity and a bounded numeric argument. The compiler, command/guest-file APIs and snapshots are crate-private and their original tests run inside the crate. An external API test checks every supported numeric input against an independent stopping-time calculation and rejects values outside the range. [The workbench item](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) records the required fixes for global arrays, block scope, formatted-output counts, UTF-8 paths and sector reclamation before any interactive or general-purpose API opens.

## Checks run at the head

- `cargo test --locked --offline -p uor-r4-mvm`
- `cargo clippy --locked --offline -p uor-r4-mvm --all-targets -- -D warnings`
- `cargo fmt -p uor-r4-mvm --check`
- `cargo check --locked --offline -p uor-r4-mvm --lib --target wasm32-unknown-unknown` (compile only)
- Ran `uor-mvm status`, `uor-mvm collatz 27` and `uor-mvm collatz 512` from the normal cargo entry point. The guest reported RV64IMAC and CID 3. The numeric results above are the observed stdout.
- Compared the original dependency and build manifests before compiling, and checked the patch for whitespace errors. The implementation adds no build script, network transport or host command execution.

## Skills used

Inspected the native sources and normal CLI path, verified immutable upstream bytes, and used bounded local execution and an independent arithmetic oracle.

## Still open

#3939 still requires actual browser-WASM and desktop C execution, responsive workbench and editor integration, revision-safe updates, virtual Git, bounded durable CoW, Web Locks and reload/corruption/quota tests. The next offline unit can verify the same pinned programs through a local WASM worker with networking disabled and compare bytes, IDs and refusal cases against the native reader. Persistence should keep the in-memory state and the durable record consistent after a failed save.

The approved repository workflow still needs host-owned repository identity and default-branch discovery, endpoint-specific authenticated reads, an independently confirmed branch/edit/commit/PR sequence, revision checks tied to the actual existing task, and the explicitly opted-in non-production smoke. The full draft's browser, desktop, companion and external actions were not executed during this adoption.

## Plain summary

A local operator can boot the emulator and measure a bounded, input-dependent computation through a genuine guest C binary. The browser IDE and repository-write workflow remain open under #3939.

## Adoption decision

The maintainer review adopts Kimi’s native work with its upstream licenses and source pins. One numeric argument selects a bounded workload that exercises the compiler and the emulator together. The original browser companion crosses a separate trust boundary when a web page selects a repository or a write destination. Issue #3939 retains that integration work and its permission checks.
