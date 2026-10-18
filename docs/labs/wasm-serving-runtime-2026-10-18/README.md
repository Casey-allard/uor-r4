# WebAssembly D11 Integer Serving Runtime and Audit Gate (#1172, PR #2654)

- **Date:** 2026-10-18
- **Author:** Yazan Barakat (`@Cyborg-Hawk-AI`), reviewed and hardened by Anti-Gravity
- **Milestone / Issues:** [#1172](https://github.com/UOR-Foundation/uor-r4/issues/1172), [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) (`[M4]`), [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) (`[M5]`), [#2035](https://github.com/UOR-Foundation/uor-r4/issues/2035) (`[M7]`), [#2655](https://github.com/UOR-Foundation/uor-r4/issues/2655), [#2656](https://github.com/UOR-Foundation/uor-r4/issues/2656), [#2657](https://github.com/UOR-Foundation/uor-r4/issues/2657)
- **PR:** [#2654](https://github.com/UOR-Foundation/uor-r4/pull/2654) (`966c28519574c541c82b3f414a77d5091d1a57f4`)
- **Resource cost:** `$0` new paid resource (host-only Rust, `wasm32-unknown-unknown` bytecode inspection, Node.js round-trip harness, and pre-recorded offline receipts).

## Summary

Delivers the browser/Node.js WebAssembly D11 integer serving runtime (`crates/uor-r4-integer/src/wasm_serve.rs`, `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`, and `crates/uor-r4-integer/src/wasm_host.rs` behind the `serving-wasm` Cargo feature), exact `UR4SESS1` binary session state serialization (`crates/uor-r4-integer/src/session_codec.rs`), 4-byte-plane WASM SIMD128 `i8x16_swizzle` dot-product kernels (`crates/uor-r4-integer/src/stack/simd.rs`), the synthetic `UORLUT01` stack fixture generator (`crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`), browser/Node.js verification harnesses (`docs/wasm-serving/wasm_serve_demo.html`, `docs/wasm-serving/verify.mjs`), and the call-graph WASM zero-matmul / zero-mul / zero-float audit gate (`scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`).

Also includes the offline `xtask qubo-codec` solver (`xtask/src/qubo_codec.rs`), D5-b address-assignment QUBO memo (`ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md`), second-instrument Linux RAPL energy protocol and synthetic-stack receipt (`ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`, `ops/measurement_report_rnd.json`), and labeled exploratory Braket SV1 2-qubit receipt (`ops/braket_qaoa_toy_receipt.json`).

## Hardening Applied Before Merge

1. **Zero-`mul` WASM serving hot path (`7/7` serving symbols pass `--serving --call-graph`):**
   - `crates/uor-r4-integer/src/wasm_serve.rs`: Replaced `Vec<Option<Slot>>` with `Vec<Option<Box<Slot>>>` so indexing into `SLOTS` shifts by pointer width (`handle << 2`) rather than emitting `i32.mul` by `sizeof(Slot) == 456`. Checked slot capacity (`< MAX_SLOTS`) before `Box::leak(Box::new(model))` so a full slot table never leaks the parsed `IntegerStackModel`, and used `top1.saturating_sub(top2)` for `margin_q16`. Exports `wasm_load_stack`, `wasm_step`, `wasm_save`, `wasm_restore`, `wasm_weights_per_token`, `wasm_alloc_events`, and `wasm_leaked_models` (re-exported to JS via `wasm_bindgen_exports.rs` as `init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `allocEvents`, `leakedModels`).
   - `crates/uor-r4-integer/src/stack/session.rs` and `crates/uor-r4-integer/src/stack/flock.rs`: Used `core::hint::black_box` on `d` in `stack_recurrence` (preventing LLVM from coalescing `d * 6` into `i32.mul`) and in small zero-initialization loops in `stack_heads`, `flock_select_integer`, and `restore_state` (preventing LLVM from lowering loops to `memory.fill` or `memset` calls in hot step paths).
   - `scripts/audit_zero_matmul_wasm.py`: Added `--serving` mode checking all 7 `SERVING_WASM_SYMBOLS` (`loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `stack::IntegerStackSession::step`, and `stack::stack_argmax`), matching both raw Rust symbol mangling and `wasm-bindgen` demangled `name` sections, and classifying `0xFC 8..17` bulk-memory/table operations as non-arithmetic memory operations.
2. **Exact 32-bit `swizzle_dot_arm_t` & `dot_dot_arm_m` (`crates/uor-r4-integer/src/stack/simd.rs`):**
   - Replaced single-byte `narrow_i16x8_s` truncation with a 4-byte-plane `i8x16_swizzle` + `u16x8_extadd_pairwise_u8x16`/`i16x8_extadd_pairwise_i8x16` + `u32x4_extadd_pairwise_u16x8`/`i32x4_extadd_pairwise_i16x8` decomposition (and matching scalar `byte_plane_swizzle_dot`) that is bit-identical to the reference loop across the full `i32::MIN..=i32::MAX` range, with `dot_dot_arm_m` delegating to `swizzle_dot_arm_t`.
3. **Untrusted snapshot parser hardening (`crates/uor-r4-integer/src/session_codec.rs`):**
   - Added `checked_add` in `Reader::take` and `checked_count` in `Reader::i32s`, `u32s`, `i64s`, `u64s`, and `decode` (`layer_count`) so malformed varints in a `UR4SESS1` stream return `Err("session codec: truncated input".to_string())` before any allocation (`rejects_oversized_varint_slice_count_without_oom`).
4. **Repository hygiene:**
   - Removed generated `pkg-serving/` bundle, root `wasm_serve_demo.html`, `handoff/work-cards.md`, and `ops/measure_final_rnd.py` from tracked git state; preserved `docs/wasm-serving/wasm_serve_demo.html`, `docs/wasm-serving/verify.mjs`, `docs/evidence/s1-4-snap-parity-2026-09-29.json`, `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md`, `ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`, `ops/measurement_report_rnd.json`, and `ops/braket_qaoa_toy_receipt.json`; updated `README.md` `## Acknowledgements`.

## Verification

- `cargo test -p uor-r4-integer --features serving-wasm`: **165 passed; 0 failed** (164 unit tests + 1 integration test).
- `cargo test -p xtask`: **78 passed; 0 failed**.
- `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`: **0 compiler warnings**.
- `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap`: **7/7 serving symbols PASS (`mul=0, div=0, float=0, unclassified=0`)**.
- Node.js round-trip (`docs/wasm-serving/verify.mjs`: `loadStack` -> 32 token steps -> `saveSession` -> `restoreSession` -> 16 token steps compared against a fresh 48-token reference run) on both `-C target-feature=-simd128` and `+simd128` builds: **`reloadIdentical: true, run1MatchesRef: true`**.
- `wasm32-unknown-unknown` `+simd128` execution of `swizzle_dot_arm_t` and `dot_dot_arm_m` in Node.js V8 across full `i32::MIN..=i32::MAX` and nibbles `0..16`: **PASS (`rc = 0`, `mul=0, div=0, float=0`)**.

## Maintainer annotation, 2026-10-18 (post-merge; the record above is unchanged)

Recorded by the Kimi PM from the outside-collaborator audit. The merged record is left as written; these are the corrections a reader needs.

1. **The SIMD kernels are unintegrated fixtures, not the serving hot path.** `swizzle_dot_arm_t`, `dot_dot_arm_m` and `byte_plane_swizzle_dot` have no callers outside `crates/uor-r4-integer/src/stack/simd.rs` and its own tests; `stack_gemv_pairs` (`crates/uor-r4-integer/src/stack/kernels.rs`) still runs the scalar pair-table path. The `7/7` zero-`mul` audit never exercises them. Treat them as fixtures pending integration.
2. **`docs/wasm-serving/*` is not runnable from a clean clone.** `wasm_serve_demo.html:33` imports `./pkg-serving/uor_r4_integer.js`, and `verify.mjs:2,4` reads `./uor_r4_integer_bg.wasm` and imports `./uor_r4_integer.js` from the same directory. `pkg-serving/` is gitignored (`.gitignore:67-68`) and no bundle is committed, so both files are preserved as source only. Build with the `wasm-pack` command in "Verification" above and place its output beside them.
3. **The `serving-wasm` D29 interpretation.** `crates/uor-r4-integer/src/serving_root.rs:47,51,54` declares `model`, `sampling` and `tables` unconditionally, so the `serving-wasm` build compiles them. Compliance rests on the feature defaulting off plus link-time removal, not on their absence from the module. This accepts the reviewer's "presence counts, not only execution" standard as an interpretation for this artifact's scope; it is not recorded in `DECISIONS.md` and does not amend D29.
4. **Provenance note.** The squash merge `966c28519574c541c82b3f414a77d5091d1a57f4` is authored by Yazan Barakat but contains maintainer-written hardening (`aa5c21d91`, `9d6a31be9`, `c921e0b56`); `git blame` will misattribute those lines.
