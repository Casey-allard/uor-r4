# REPORT: m1-d11-exporter-discrete, antigravity-14

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Direct Cargo dependency wiring across `uor-r4-discrete`, `uor-r4-lut`, and `uor-r4-integer` requires `Cargo.lock`:** `Cargo.lock` is outside the `scope` of [`m1-d11-exporter-discrete.md`](../../../../roadmap/items/m1-d11-exporter-discrete.md) (`[crates/uor-r4-discrete/**, crates/uor-r4-integer/**, crates/uor-r4-lut/**, docs/mechanisms/lut-serving.md, docs/model/serving-and-export.md]`), and `uor-harness loop gates --full` runs `--locked`. Rather than editing `Cargo.toml` dependencies, `uor-r4-lut`, `uor-r4-discrete`, and `uor-r4-integer` each parse and validate the identical `UORLUT01` (`uor-r4.lut-discrete/1`) container format directly using their existing `serde`, `serde_json`, and `sha2` dependencies.
- **`driver_source_sha256()` and `.apply(` audit constraints in `uor-r4-discrete`:** `crates/uor-r4-discrete/src/training.rs` hashes `snapshot.rs` and `main.rs` in `driver_source_sha256()`, and `crates/uor-r4-discrete/src/lib.rs` scans `src/**/*.{rs,cu}` to enforce that `.apply(` calls appear only in `bounded_fixture.rs` and `admitted_transition.rs`. Placing the exporter in [`crates/uor-r4-discrete/src/d11_export.rs`](../../../../../crates/uor-r4-discrete/src/d11_export.rs) with zero `.apply(` calls left `driver_source_sha256()` and the `.apply(` inventory unchanged.
- **CGU-sensitive `wasm_load_stack` inlining in `audit_zero_matmul_wasm.py`:** Adding `pub mod discrete_d11;` to [`crates/uor-r4-integer/src/serving_root.rs`](../../../../../crates/uor-r4-integer/src/serving_root.rs) shifted `codegen-units=16` partitioning so `wasm_load_stack` (which lacked the `#[inline(never)]` attribute present on `wasm_step`, `wasm_save`, `wasm_restore`, and `wasm_weights_per_token`) was inlined into `wasm_bindgen_exports::load_stack`, causing `audit_zero_matmul_wasm.py --serving --call-graph` to traverse `IntegerStackModel::parse` from the un-allowlisted root `loadStack` (`mul=45, div=6, float=32`). Adding `#[inline(never)]` to `wasm_load_stack` in [`crates/uor-r4-integer/src/wasm_serve.rs`](../../../../../crates/uor-r4-integer/src/wasm_serve.rs) restored deterministic call-graph separation across CGU layouts (`7/7` symbols `ok`, `mul=0, div=0, float=0, unclassified=0`).

## What changed

This task closes #2838 by delivering the table-native `UORLUT01` (`uor-r4.lut-discrete/1`) container schema, the `DiscreteModel` exporter/importer in `uor-r4-discrete`, and the integer loader and `Route`/`Flock`/`Relation` evaluator in `uor-r4-integer`:

- **Container schema ([`crates/uor-r4-lut/src/format.rs`](../../../../../crates/uor-r4-lut/src/format.rs)):** Added `DISCRETE_SCHEMA = "uor-r4.lut-discrete/1"`, `DiscreteShape`, `DiscreteSource`, `DiscreteHeader`, `DiscreteArtifactBuilder`, and `DiscreteArtifact` over `UORLUT01` (`discrete.table` `I32`, `discrete.cells` `U32`, `discrete.routes` `U32`, `discrete.gates` `I32`, 32-byte aligned, zero matrix sections).
- **Discrete exporter/importer ([`crates/uor-r4-discrete/src/d11_export.rs`](../../../../../crates/uor-r4-discrete/src/d11_export.rs), [`crates/uor-r4-discrete/src/lib.rs`](../../../../../crates/uor-r4-discrete/src/lib.rs)):** Implemented `export_d11`, `import_d11`, `validate_and_extract_cells`, and `orbit_root_id`, verifying exact E8 240-root orbit membership, endpoint 24-cell orbit root extraction, ternary binder route bounds (`0..64`), gate trit bounds (`{-1, 0, +1}`), and byte-identical snapshot reconstruction (`import_d11(export_d11(&snapshot)) == snapshot`).
- **Integer D11 loader and evaluator ([`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../../crates/uor-r4-integer/src/discrete_d11.rs), [`crates/uor-r4-integer/src/native_geometry.rs`](../../../../../crates/uor-r4-integer/src/native_geometry.rs), [`crates/uor-r4-integer/src/serving_root.rs`](../../../../../crates/uor-r4-integer/src/serving_root.rs), [`crates/uor-r4-integer/src/wasm_serve.rs`](../../../../../crates/uor-r4-integer/src/wasm_serve.rs)):** Implemented `IntegerDiscreteModel::from_bytes`, `evaluate_prefix`, and `observe_target` across `Route`, `Flock`, and `Relation` (with exact 7-channel relation scoring and ternary binder transport) in pure integer arithmetic, plus `#[inline(never)]` on `wasm_load_stack`.
- **Documentation and item status ([`docs/mechanisms/lut-serving.md`](../../../../mechanisms/lut-serving.md), [`docs/model/serving-and-export.md`](../../../../model/serving-and-export.md), [`docs/roadmap/items/m1-d11-exporter-discrete.md`](../../../../roadmap/items/m1-d11-exporter-discrete.md)):** Documented `uor-r4.lut-discrete/1`, `d11_export`, and `IntegerDiscreteModel`, and marked `m1-d11-exporter-discrete` `status: done`.
- **Size by class:** ~1,970 lines of code, ~550 lines of unit tests, and ~60 lines of documentation/task records (+2,510 / -11 lines across 11 files, vs. the ORDER's 650-line estimate, ratio ~3.86x due to embedding the complete self-contained 240-root E8 / 64-element icosian action tables and full `Route`/`Flock`/`Relation` integer evaluator in `crates/uor-r4-integer/src/discrete_d11.rs` without adding cross-crate dependencies to `Cargo.lock`).

## Measurements

- **Discrete D11 round-trip and CUDA-witness integer parity (host CPU):**
  - Command: `~/.cargo/bin/cargo test --locked -p uor-r4-lut && ~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib -- d11_export && ~/.cargo/bin/cargo test --locked -p uor-r4-integer --lib -- discrete_d11 && ~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm --lib`
  - Result:
    - `export_d11` -> `import_d11` reconstructs `Route`, `Flock`, and `Relation` `DiscreteModel` snapshots byte-for-byte (`serde_json::to_value` equality and `snapshot_sha256` match).
    - `Route` 2-token `exact_writer` witness: prefix `[0]` predicts `0` (`admitted_cells = [0, 17]`, `selected_loss = 0`); prefix `[1]` predicts `1` (`admitted_cells = [17, 0]`, `selected_loss = 640`); edited-table negative control flips `admitted_cells` to `[1, 16]`.
    - `Route` 3-token `context_transport` witness: prefix `[0, 1, 2]` yields `[NEG_B, NEG_B]`; prefix `[0, 0, 2]` yields `[B, B]`.
    - `Flock` and `Relation` `C2` copy-successor witness (`vocab=5, context=6, kept_cells=2, kept=1, sweeps=1`, prefix `[0, 1, 2, 3, 4, 1]`, target `2`): baseline (`gates[2]=1, gates[6]=0`) yields read vector `[A, D_ROOT]` and `selected_loss = 640`; active `SequentialSuccessor` at level 0 (`gates[2]=1, gates[6]=1`) yields relation scores `[0, 0, 8, 0, 4]` on eligible level-0 nodes `[0, 1, 2, 4, 5]`, read vector `[A, B]`, and `selected_loss = 448` (`0` BPB export rounding gap).
- **Native and WebAssembly zero-multiply / zero-float audits:**
  - Native command: `python3 scripts/audit_zero_matmul_serving.py "$CARGO_TARGET_DIR/release/uor-r4-stack" --stack`
  - Native result: `44/44` mandatory symbol ranges `PASS` (`0` Class I mul, `0` Class II div, `0` Class III float).
  - WebAssembly command: `RUSTFLAGS="-C target-feature=-simd128" ~/.cargo/bin/cargo build --locked --release --target wasm32-unknown-unknown -p uor-r4-integer --features serving-wasm --lib && python3 scripts/audit_zero_matmul_wasm.py "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/uor_r4_integer.wasm" --serving --call-graph --tap`
  - WebAssembly result: `7/7` serving symbols `ok` (`loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `stack::IntegerStackSession::step`, `stack::stack_argmax`; all `mul=0, div=0, float=0, unclassified=0`).

## Checks run at the head

- `~/.cargo/bin/cargo test --locked -p uor-r4-lut`: `ok. 6 passed; 0 failed`.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib`: `ok. 212 passed; 0 failed`.
- `~/.cargo/bin/cargo test --locked -p uor-r4-integer --lib`: `ok. 45 passed; 0 failed`.
- `~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm --lib`: `ok. 296 passed; 0 failed`.
- `python3 scripts/audit_zero_matmul_serving.py "$CARGO_TARGET_DIR/release/uor-r4-stack" --stack`: `exit 0` (`44/44 PASS`).
- `python3 scripts/audit_zero_matmul_wasm.py "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/uor_r4_integer.wasm" --serving --call-graph --tap`: `exit 0` (`1..7 ok`).
- `uor-harness loop gates --full`: `all 17 gate(s) green`.
- `uor-harness loop report --check`: `REPORT ok`.
- `uor-harness loop pr-check --local`: `pr-check: clean`.

## Skills used

- `uor-research-architect`: followed for native discrete geometry invariance, zero-softmax / zero-matmul compliance, and artifact-bound integer serving verification.

## Still open

- Exporting a trained discrete checkpoint after M1 trunk training and measuring its served evaluation score and tokens per second on the held-out stream are tracked under `m1-export-gap-and-toks` (`#2839`).

## Plain summary

A discrete language model in UOR-R4 stores all of its parameters as whole numbers: 8-dimensional coordinates on the E8 lattice (a 240-point crystal in eight dimensions), 24-cell address labels (which of the 24 regions of that crystal each token's final block belongs to), ternary binder routes (tree-structured address-transport choices), and ternary gates (`-1`, `0`, or `+1` switches that weight tree levels and seven token-to-token relation channels). Previously, `DiscreteModel` snapshots could only be saved as JSON for the GPU training driver (`crates/uor-r4-discrete`), and the integer serving container (`UORLUT01` in `crates/uor-r4-lut` and `crates/uor-r4-integer`) only had schemas for older 4-bit quantized float stacks. This task adds a table-native container schema (`uor-r4.lut-discrete/1`) that packs a discrete model's four integer tables directly into a 32-byte-aligned `UORLUT01` file with zero matrix sections, an exporter and importer in `crates/uor-r4-discrete::d11_export` that round-trip snapshots byte-for-byte, and an integer evaluator in `crates/uor-r4-integer::discrete_d11` (`IntegerDiscreteModel`) that loads the container and reproduces the CUDA-qualified `Route`, `Flock`, and `Relation` predictions and losses bit-for-bit while keeping both the 44-symbol native and 7-symbol WebAssembly zero-multiply audits green.
