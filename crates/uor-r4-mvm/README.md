# uor-r4-mvm

This native Rust crate runs RV64IMAC Linux-userland binaries on a software CPU with an in-memory filesystem and vsock transport. Kimi adapted the pinned LexLean compiler and emulator in [#3941](https://github.com/UOR-Foundation/uor-r4/pull/3941/commits/138f473eaa0628322ce598da8cd447dababbc814). Codex adopted the native component and added a bounded local command for [#3939](https://github.com/UOR-Foundation/uor-r4/issues/3939).

## Run locally

```sh
cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- status
cargo run --locked -q -p uor-r4-mvm --bin uor-mvm -- collatz 27
```

`status` boots the pinned initramfs and prints the guest architecture and CID. `collatz N` finds the smallest starting value in 1 through N with the longest stopping time under the Collatz iteration. For 27 the guest prints `limit=27 start=27 steps=111`. The command compiles the checked-in C program with the supplied integer, executes its RISC-V ELF, and forwards its output. It requires an ELF machine ID of 243 and at least one retired guest instruction.

Each process gets 8 MiB of guest RAM and up to 25 million retired instructions. The CLI accepts two commands and bounds N to 1–512. The guest starts fresh on each invocation and receives no host filesystem, command, credential or network connection. The public Rust API provides `guest_identity()` and `run_collatz(limit: u16)`. Both create a fresh guest, and the library validates the numeric range before boot. Only crate-private code accepts C, guest paths, bytecode or command text. Its pinned compiler and filesystem have outstanding correctness defects tracked in the workbench item. The guest RAM and fuel limits apply to execution. The browser worker, persistence and GitHub companion are separate work under #3939.

## Upstream provenance

Vendored from `Casey-allard/lexlean-r4` commit `01956d5c35f268fc7d1b646d97d3dff21b9e945f` under MIT OR Apache-2.0. The upstream license texts are preserved in `LICENSE-MIT` and `LICENSE-APACHE`; generated-code comments identify `R4.SpiralCoreVsock` (`src/R4/SpiralCoreVsock.lex.tex`).

| Input | Upstream identity |
|---|---|
| `crates/wasm/src/mvm.rs` | 98,525 bytes; SHA-256 `2789ec57daa7f92dbd094e5d2c38a8ae343b4f38bd09cd2ed6bc377fa7d67206` |
| `crates/wasm/src/mvm_cc.rs` | 75,765 bytes; SHA-256 `19390475442bbede4332bf38776706d42aa09e2daa9df4e014d140c6af5ce12f` |
| `crates/wasm/src/lexlean_generated.rs` | 2,206,963 bytes; SHA-256 `acc9a89d65bb43bdb61a720fd90570dacf46a43adb9340fcbc427a9245e247e8` |
| `lexlean_generated.rs:45,751-45,938` only | 188-line `r4_spiralcore_vsock` module; SHA-256 `102c1c859413c4d4a1470d8fecb4dcc3dfa8fa0ea50c402468f22718ad0b478f` |
| `crates/wasm/src/mvm_initramfs.bin` | 12,164 bytes; SHA-256 `071e9c8d7749c3b86947529db532db0b82ac3707ad1b738d3ba681dc8161c65c` |
| `crates/wasm/tests/mvm_spiralcore_sandbox.rs` | 9,573 bytes; SHA-256 `9b99e2d64f32475decc5c10db658677a4b9b211f8d6963e3668fbb471a653c7b` |

The module ends at line 45,938. `r4_compositional_gllm` begins at line 45,940 and is excluded, as are every other generated-file line, LexLean `api.rs`, `lib.rs`, `wasm_ast_eval.rs`, `execute_cargo_json`, the generated application/model and AST-evaluated Cargo behavior.

## UOR modifications

- `mvm_cc.rs` is byte-identical to upstream. `spiralcore_vsock.rs` contains only the exact 188-line module above.
- [The emulator patch](provenance/mvm-uor.patch) records both import substitutions and the local ELF, snapshot and frame-length changes. The provenance test reverses that unified diff in memory and checks the upstream length and SHA-256.
- The local ELF parser validates every program header and supplies checked file and guest-memory ranges to the loader. Snapshots save the next sector number. The vsock sender checks the full payload length and the receiver requires one complete frame.
- `mvm_initramfs.rs` is generated from the pinned binary into a Rust byte slice, so no compiled initramfs binary is committed. Its bytes and SHA-256 are tested.
- Vendored modules are `#[rustfmt::skip]` so package formatting cannot change upstream hashes; new tests and adapter code remain rustfmt-checked.
- `lib.rs` exposes the guest identity and the fixed workload. Internal host tests exercise the original source and use synthetic placeholders for the secret policy. An external API test compares guest output to an independent stopping-time calculation for all 512 supported inputs.
- Cargo/rustc are not emulated by an AST evaluator. Without a genuine guest Rust toolchain they must report unavailable. C compilation and execution must emit genuine `EM_RISCV` output and exact nontrivial results.

The native package tests check exact output, ELF identity, filesystem changes, in-memory CoW restore, vsock admission and pinned source identities. The browser and desktop acceptance tests belong to the remaining workbench item.
