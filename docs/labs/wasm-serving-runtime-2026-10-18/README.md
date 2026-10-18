# WebAssembly D11 Integer Serving Runtime and Audit Gate (#1172, PR #2654)

- **Date:** 2026-10-18
- **Author:** Yazan Barakat (`@Cyborg-Hawk-AI`), reviewed and hardened by Anti-Gravity
- **Milestone / Issues:** [#1172](https://github.com/UOR-Foundation/uor-r4/issues/1172), [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) (`[M4]`), [#2655](https://github.com/UOR-Foundation/uor-r4/issues/2655), [#2656](https://github.com/UOR-Foundation/uor-r4/issues/2656), [#2657](https://github.com/UOR-Foundation/uor-r4/issues/2657)
- **PR:** [#2654](https://github.com/UOR-Foundation/uor-r4/pull/2654)
- **Resource cost:** `$0` new paid resource (host-only Rust, `wasm32-unknown-unknown` bytecode inspection, Node.js round-trip harness, and pre-recorded offline receipts).

## Summary

Delivers the browser/Node.js WebAssembly D11 integer serving runtime (`crates/uor-r4-integer/src/wasm_serve.rs` behind the `serving-wasm` Cargo feature), exact session state serialization (`crates/uor-r4-integer/src/session_codec.rs`), 4-byte-plane WASM SIMD128 `i8x16_swizzle` dot-product kernels (`crates/uor-r4-integer/src/stack/simd.rs`), and the call-graph WASM zero-matmul / zero-mul / zero-float audit gate (`scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`).

Also includes the offline `xtask qubo-codec` solver (`xtask/src/qubo_codec.rs`), D5-b architectural memo (`docs/architecture/d5b-qubo-codec-2026-04-16.md`), and pre-recorded toy receipt (`docs/measurements/1172-braket-qubo-receipt.json`).

## Hardening Applied Before Merge

1. **Zero-`mul` WASM serving hot path (`7/7` serving symbols pass `--serving --call-graph`):**
   - `crates/uor-r4-integer/src/wasm_serve.rs`: Replaced `Vec<Option<Slot>>` with `Vec<Option<Box<Slot>>>` so indexing into `SLOTS` shifts by pointer width (`handle << 2`) rather than emitting `i32.mul` by `sizeof(Slot) == 456`. Checked slot capacity (`< MAX_SLOTS`) before `Box::leak(Box::new(model))` so a full slot table never leaks the parsed `IntegerStackModel`, and used `top1.saturating_sub(top2)` for `margin_q16`.
   - `crates/uor-r4-integer/src/stack/session.rs` and `crates/uor-r4-integer/src/stack/flock.rs`: Used `core::hint::black_box` on `d` in `stack_recurrence` (preventing LLVM from coalescing `d * 6` into `i32.mul`) and in small zero-initialization loops in `stack_heads`, `flock_select_integer`, and `restore_state` (preventing LLVM from lowering loops to `memory.fill` or `memset` calls in hot step paths).
   - `scripts/audit_zero_matmul_wasm.py`: Added `--serving` mode checking all 7 serving entrypoints (`IntegerStackSession::step`, `stack_recurrence`, `stack_heads`, `flock_select_integer`, `wasm_step`, `with_slot_mut`, `wasm_snapshot`), matching both raw Rust symbol mangling and `wasm-bindgen` demangled `name` sections.
2. **Exact 32-bit `swizzle_dot_arm_t` & `dot_dot_arm_m` (`crates/uor-r4-integer/src/stack/simd.rs`):**
   - Replaced single-byte `narrow_i16x8_s` truncation with a 4-byte-plane `u8x16_swizzle` + `i16x8_extadd_pairwise_i8x16_s`/`u` + `i32x4_extadd_pairwise_i16x8_s`/`u` decomposition that is bit-identical to `scalar_dot_arm_t` across the full `i32::MIN..=i32::MAX` range.
3. **Untrusted snapshot parser hardening (`crates/uor-r4-integer/src/session_codec.rs`):**
   - Added `checked_add` in `Reader::take` and `checked_count` in `Reader::i32s`, `u32s`, `i64s`, `u64s`, and `decode` (`layer_count`) so malformed varints return `Err(CodecError::UnexpectedEof)` before any allocation (`rejects_oversized_varint_slice_count_without_oom`).
4. **Repository hygiene:**
   - Removed generated `pkg-serving/` bundle, root `wasm_serve_demo.html`, `handoff/work-cards.md`, and `ops/measure_final_rnd.py` from tracked git state; updated `README.md` `## Acknowledgements`.

## Verification

- `cargo test -p uor-r4-integer --features serving-wasm`: **165 passed; 0 failed** (164 unit tests + 1 integration test).
- `cargo test -p xtask`: **78 passed; 0 failed**.
- `wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm` (with `-C target-feature=-simd128`): **0 compiler warnings**.
- `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap`: **7/7 serving symbols PASS (`mul=0, div=0, float=0, unclassified=0`)**.
- Node.js round-trip (`wasm_load_model` -> 16 token steps -> `wasm_snapshot` -> `wasm_restore` -> 16 token steps): **`reloadIdentical: true, run1MatchesRef: true`**.
