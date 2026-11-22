# uor-r4-mvm

Deterministic zero-`unsafe` RV64IMAC Linux-userland MicroVM, C99-to-RV64 ELF64 compiler, POSIX/`jq`/heredoc shell, 64 KB Copy-on-Write filesystem, and `SpiralCore` `virtio-vsock` zero-NIC switch for CLI and browser Studio WASM execution ([#3939](https://github.com/UOR-Foundation/uor-r4/issues/3939)).

## Run locally

```sh
cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- status
cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- collatz 27
```

- `status` boots the pinned initramfs and prints `architecture=riscv64-rv64imac guest_cid=3`.
- `collatz N` compiles a freestanding C program via `cc` inside the MicroVM, executes the emitted `EM_RISCV` (243) ELF64 binary (`limit=27 start=27 steps=111`), and checks nonzero retired instructions.
- Public Rust callers use [`uor_r4_mvm::MvmSandbox`](src/sandbox.rs), [`guest_identity`](src/lib.rs), and [`run_collatz`](src/lib.rs); browser Web Workers use [`WasmMvmSandbox`](src/wasm.rs) behind `--features wasm` on `wasm32-unknown-unknown`.

Each process receives 8 MiB of guest RAM and up to 25 million retired instructions with zero host filesystem access and zero direct internet sockets (`AF_VSOCK = 40` only).

## Upstream provenance

Vendored and upgraded from `Casey-allard/lexlean-r4` commit `288e2677` under `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`). Pure LexLean admission modules were extracted into [`src/lexlean_admission.rs`](src/lexlean_admission.rs); model/transformer, tokenizer, and `wasm_ast_eval` shortcut modules are excluded.

| Input | Upstream identity (`288e2677`) |
|---|---|
| `crates/wasm/src/mvm.rs` | 331,436 bytes; SHA-256 `85aa69b442c70fc8de646c62bba4c37842b5fbc502627970b438873aecbe12ee` |
| `crates/wasm/src/mvm_shell.rs` | 392,781 bytes; SHA-256 `db9a015b21ab77e8ad6e2901a7ff2a8ce68fa2646212f975b2f3b25266220db2` |
| `crates/wasm/src/mvm_cc.rs` | 127,667 bytes; SHA-256 `66d497f62868dce78ba119972a41dd21e1c572328ab1317e466e2e147e70491d` |
| `crates/wasm/src/vsock.rs` | 1,183 bytes; SHA-256 `1314bc8f02e4ed542f116b57769d2d6b23a5433732252fed811f6a2d2992df09` |
| `lexlean_generated.rs` (`r4_secops_audit`, `45,029..=45,194`) | 6,514 bytes; SHA-256 `a262c7082608879ba3feb4dfe977ed0fca9b6a1fed7bc64fa4437cec091838e6` |
| `lexlean_generated.rs` (`r4_spiralcore_vsock`, `45,839..=46,026`) | 6,391 bytes; SHA-256 `102c1c859413c4d4a1470d8fecb4dcc3dfa8fa0ea50c402468f22718ad0b478f` |
| `lexlean_generated.rs` (`r4_connected_studio`, `46,282..=46,431`) | 5,054 bytes; SHA-256 `26706cf61cdabccba78fe89320cf706e1d23e9e8901d7916f293c15d863528be` |
| `lexlean_generated.rs` (`r4_mvm_linux_container`, `48,288..=48,531`) | 9,838 bytes; SHA-256 `a7a9a9b78ff47edcbf5bf6486ee6e6994552f33ef30602051a59ebeb5dd07955` |
| `lexlean_generated.rs` (`r4_mvm_polyglot_toolchain`, `48,840..=48,909`) | 3,029 bytes; SHA-256 `5efb59ef69c16747a437630a195533f26b524bf1707e664ee61e34aa4d6f0bf9` |
| `crates/wasm/src/mvm_initramfs.bin` | 12,164 bytes; SHA-256 `071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c` |

## UOR modifications

- [mvm-uor.patch](provenance/mvm-uor.patch), [mvm_cc-uor.patch](provenance/mvm_cc-uor.patch), and [mvm_shell-uor.patch](provenance/mvm_shell-uor.patch) record every local patch against `288e2677` and are reversed in provenance tests to recover exact upstream byte lengths and SHA-256 digests.
- `mvm.rs` fixes and security hardening: `MVM_INITRAMFS` source-slice substitution, hardened ELF loading (`crate::elf::parse`), `next_sector` preservation and full-payload SHA-256 verification for `UORCOW01` CoW snapshots, reclamation of superseded/deleted 64 KB CoW sectors, uniform read-only protection over system (`/bin`, `/sbin`, `/usr/bin`, `/usr/include`, `/usr/src`, `/etc`) and canonical paths across host/shell/syscall entry points, `wasm32`-safe checked guest memory range helpers + `MAX_GUEST_FILE_BYTES` cap on `SYS_write`, fallible malformed `.git` handling (no `.unwrap()` panics), and replacement of `wasm_ast_eval::execute_workspace_cargo` with honest exit-127 unavailability messages.
- `mvm_cc.rs` prerequisite fixes: global array address/ptr decay (`GlobalVar.is_array`), lexical scope save/restore for `Stmt::Block` and `Stmt::For`, exact `printf`/`puts` byte count return values, and UTF-8 preservation across comment/preprocessor stripping.
- `mvm_shell.rs` prerequisite & safety fixes: byte-preserving UTF-8 shell/pipe/heredoc parsing, safe raw-byte `tar_build_header` slicing without UTF-8 panic at byte 100, and uniform protected-path enforcement in `mv` and `tar -x`.
- `vsock.rs` enforces exact wire frame length (`wire.len() != end`) matching `deliver_vsock_packet`.
- `mvm_initramfs.rs` stores the pinned `UORMVM01` initramfs as a Rust slice with zero committed binaries (§3.13). `Cargo.toml` reuses only existing workspace-locked `serde`, `serde_json`, `sha2`, and target-gated `wasm-bindgen`/`js-sys` (zero new external dependencies).
