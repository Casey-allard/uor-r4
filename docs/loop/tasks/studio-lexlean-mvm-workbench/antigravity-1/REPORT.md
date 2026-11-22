# REPORT for `studio-lexlean-mvm-workbench` (`antigravity-1`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939.

## What did not work

- Initial `cargo test --locked -p uor-r4-mvm` failed until `Cargo.lock` updated `uor-r4-mvm` dependency edges offline (`serde`, `serde_json`, `wasm-bindgen`, `js-sys`) reusing already workspace-locked versions (zero new external packages added).
- Upstream `lexlean-r4` `288e2677` `compile_c_to_rv64_elf` dropped hardcoded `stdio.h` lookup outside the guest filesystem, requiring internal tests to invoke `compile_c_units_to_rv64_elf` with `/usr/include` headers from the booted guest.
- First fresh-context adversarial review (`34004e92`) returned `CHANGES` on five prerequisite MVM defects listed in the item sheet (`mvm_cc.rs` global array addressing, nested block variable shadowing, `printf` return count, UTF-8 mojibake/`tar` byte-boundary slicing, and CoW sector leakage on overwrite/delete), plus system-path protection gaps, partial `UORCOW01` checksum coverage, 32-bit `wasm32` address casts, `.unwrap()` panics in malformed `.git` handling, missing `WasmMvmSandbox` runtime execution test, an inline `scope:` edit on the roadmap item, and 4-level relative links in this report.
- Second fresh-context adversarial review (`d0993046`) returned `CHANGES` on remaining edge cases: `is_protected_canonical_guest_path` used instead of `is_protected_guest_path` on `ln`, `chmod`, `touch`, `gzip`, and `SYS_openat`; unguarded `mkdir`/`rmdir`/`cp -r`/`SYS_mkdirat`; CoW sector leaks on symlink overwrite and non-`-k` `gzip`/`gunzip`; `SYS_brk`/`SYS_lseek`/`SYS_mmap` `wasm32` truncation/overflow bugs; and byte-index `&str` slicing in `complete_git_bridge`.
- Third fresh-context adversarial review (`c5b244ae`) returned `CHANGES` on root-dir `"/"` path joining (`ln`/`mv`/`cp -r` creating `"//bin"`), `rename_workspace_file` allowing `"."` -> `"/workspace"`, UTF-8 byte slicing in `printf`/`tr`/`jq`, and `mount_initramfs`/`import_cow_snapshot_bytes` checked arithmetic.

## What changed

- Reverted the inline edit to [studio-lexlean-mvm-workbench.md](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (`origin/main` restored verbatim).
- Upgraded [Cargo.toml](../../../../../crates/uor-r4-mvm/Cargo.toml), [Cargo.lock](../../../../../Cargo.lock), and [README.md](../../../../../crates/uor-r4-mvm/README.md) (`rlib` + `cdylib`, `wasm` feature behind target cfg; `serde`/`serde_json` reuse existing locked crates required by `288e2677` `mvm.rs`).
- Extracted 5 self-contained admission modules from `lexlean-r4` `288e2677` into [lexlean_admission.rs](../../../../../crates/uor-r4-mvm/src/lexlean_admission.rs) and kept [spiralcore_vsock.rs](../../../../../crates/uor-r4-mvm/src/spiralcore_vsock.rs) as a re-export.
- Ported and hardened [mvm.rs](../../../../../crates/uor-r4-mvm/src/mvm.rs), [mvm_shell.rs](../../../../../crates/uor-r4-mvm/src/mvm_shell.rs), [mvm_cc.rs](../../../../../crates/uor-r4-mvm/src/mvm_cc.rs), and [vsock.rs](../../../../../crates/uor-r4-mvm/src/vsock.rs) with reversible upstream patches [mvm-uor.patch](../../../../../crates/uor-r4-mvm/provenance/mvm-uor.patch), [mvm_cc-uor.patch](../../../../../crates/uor-r4-mvm/provenance/mvm_cc-uor.patch), and [mvm_shell-uor.patch](../../../../../crates/uor-r4-mvm/provenance/mvm_shell-uor.patch):
  - Fixed global array decay (`GlobalVar.is_array`), `Stmt::Block`/`Stmt::For` lexical scoping, exact `printf`/`puts` byte count returns, UTF-8 shell/comment/tar/`printf`/`tr`/`jq`/`complete_git_bridge` handling, and CoW sector reclamation via `free_inode_sectors`, `remove_inode_reclaiming`, and guarded `write_symlink_inode`.
  - Enforced unified `is_protected_guest_path` across host (`write_guest_file`, `remove_guest_file`, `rename_workspace_file`, `write_symlink_inode`), normalized shell paths (`rm`, `mv`, `tar -x`, `ln`, `chmod`, `touch`, `mkdir`, `rmdir`, `cp -r`, `gzip`, redirections), and Linux syscalls (`SYS_unlinkat`, `SYS_mkdirat`, `SYS_openat`, `SYS_write`); full-payload SHA-256 + `u32::try_from` + `is_cow_vfs_consistent()` for `UORCOW01` snapshots; checked `mount_initramfs` payload bounds; `wasm32`-safe `guest_range` + checked syscall arithmetic + 8 MiB `MAX_GUEST_FILE_BYTES` cap on `SYS_write`; fallible `.git` error returns (zero `.unwrap()` panics); and unconditional exit-127 `cargo`/`rustc` unavailability (zero AST-eval or model shortcut code).
- Added [sandbox.rs](../../../../../crates/uor-r4-mvm/src/sandbox.rs) (`MvmSandbox`, `ExecResult`) and [wasm.rs](../../../../../crates/uor-r4-mvm/src/wasm.rs) (`WasmMvmSandbox`), updated [lib.rs](../../../../../crates/uor-r4-mvm/src/lib.rs), [provenance.rs](../../../../../crates/uor-r4-mvm/tests/internal/provenance.rs), [mvm.rs](../../../../../crates/uor-r4-mvm/tests/internal/mvm.rs), and [api.rs](../../../../../crates/uor-r4-mvm/tests/api.rs).

## Measurements

- `wasm-pack build crates/uor-r4-mvm --target web --no-opt --out-dir /tmp/mvm-wasm-check -- --locked --features wasm`: compiled `uor_r4_mvm_bg.wasm` (`1.2M`) + JS glue (`13K`).
- One-shot live Node ESM `WasmMvmSandbox` execution against `/tmp/mvm-wasm-check/uor_r4_mvm.js` (non-committed scratch verification): `architecture=riscv64-rv64imac`, `guest_cid=3`, `echo café` preserved UTF-8, in-WASM `cc` compiled `EM_RISCV=243` printing `WASM=42` with `printf` return `8`, and `UORCOW01` snapshot (`21,227` B) restored identically on a fresh `WasmMvmSandbox` instance.
- `cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- status`: `architecture=riscv64-rv64imac guest_cid=3`.
- `cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- collatz 27`: `limit=27 start=27 steps=111`.
- Provenance reversal recovers exact `288e2677` bytes and SHA-256 digests for `mvm.rs` (`331,436` B), `mvm_cc.rs` (`127,667` B), and `mvm_shell.rs` (`392,781` B).
- Diff size breakdown (`uor-harness loop pr-check --local`): `tests 3 file(s) +447/-51, code 10 file(s) +19582/-1995, docs 3 file(s) +94/-21, generated 1 file(s) +4/-0, other 3 file(s) +1718/-22` (overrun above initial native-only estimate explained by porting the full `288e2677` POSIX/`jq`/heredoc shell, C compiler prerequisite fixes, hardening guards, and provenance patch files).

## Checks run at the head

- `cargo fmt --check -p uor-r4-mvm` and `cargo clippy --locked -p uor-r4-mvm --all-targets -- -D warnings`: passed (0 warnings, 0 errors).
- `cargo test --locked -p uor-r4-mvm --all-features`: 18 unit tests passed, 7 `api` integration tests passed (including all 512 Collatz oracle cases and new prerequisite/security regression tests), 3 `cli` tests passed.
- `cargo test --doc --locked -p uor-r4-mvm`: 3 doc-tests passed.
- `wasm-pack build crates/uor-r4-mvm --target web --no-opt --out-dir /tmp/mvm-wasm-check -- --locked --features wasm` + live Node ESM `WasmMvmSandbox` execution test: passed.
- `cargo run -q --locked -p xtask -- links` (0 broken links across changed files; 8320 sources, 29516 checked repo-wide), `uor-harness ratchet d23` (`4451`), `d25` (`93`), `d32` (`31`), `cargo run -q --locked -p uor-r4-claim-wording -- .`, and `uor-harness loop pr-check --base origin/main --head HEAD --local`: all passed (`verdict: PASS`).

## Skills used

- `using-superpowers`, `test-driven-development`, `verification-before-completion`.

## Still open

- Companion PR 2 (`antigravity/foundry-r4-studio`) remains for browser Foundry shell integration, live OPFS CoW persistence, `uor-r4-integer` WASM model wiring, Preact Web Studio archival, Playwright E2E, and `gh-pages` publishing.

## Plain summary

`uor-r4-mvm` now provides a clean, safe, hardened RV64IMAC MicroVM and C99-to-ELF toolchain (`MvmSandbox` and WASM `WasmMvmSandbox`) upgraded to `lexlean-r4` `288e2677`, with all five item prerequisites and adversarial review defects fixed and verified natively and inside WebAssembly with zero committed binaries or shortcut evaluators.
