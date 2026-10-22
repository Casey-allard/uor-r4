# M7 Three-Host Bit-Identity & Session-Snapshot Parity Harness (`m7-parity-library-cli-browser`, Issue `#2867`)

- **Date:** 2026-10-22
- **Lab:** `antigravity`
- **Milestone / Issue:** M7 (`#2035`), item [`m7-parity-library-cli-browser`](../../roadmap/items/m7-parity-library-cli-browser.md) (`#2867`)
- **Pre-registration (`DRAFT`):** [`preregistration-DRAFT-2026-10-22.md`](./preregistration-DRAFT-2026-10-22.md)
- **Bar Registration Request:** [`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431) (superseding the initial `vocab = 64` draft in [`#2867` comment `6352106589`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352106589))
- **Status:** Three-host parity harness implemented and verified (`8 / 8` prompts uninterrupted `48` tokens + `8 / 8` fresh-process `32 + 16` save/restore continuations) on the `seed = 5` `mk-synthetic-stack` (`UORLUT01`) stand-in; item `#2867` remains `status: ready` until PM bar registration and re-execution on an accepted M4 `UORLUT01` model artifact (`#2857`).

## 1. Architecture & Reference Alignment

1. **Reference `UORLUT01` Execution Path:**
   - All three hosts execute the preserved D11 integer stack engine (`uor_r4_integer::stack::IntegerStackModel` + `IntegerStackSession::step` + `stack_argmax`) with greedy decoding (`temperature = 0`, first index on ties), read mode fixed by the artifact header (`"lorentz"` on `mk-synthetic-stack`), zero floating-point operations, and zero matrix multiplication.
   - Note on container boundary: `IntegerCapabilityApi::new` (`crates/uor-r4-integer/src/capability_api.rs`) wraps the retained recurrent `Bundle` container (`IntegerModel`), whereas `UORLUT01` (`uor-r4.lut-stack/1`) is served across the Rust library, `uor-chat --stack`, and Browser `serving-wasm` by `IntegerStackModel` and `IntegerStackSession` (`crates/uor-r4-integer/tests/bundle_stack_boundary.rs`).
2. **Canonical `UR4SESS1` Session Snapshot Equivalence:**
   - All three hosts serialize and restore `SerializedStackSession` via `uor_r4_integer::session_codec::{encode, decode}` (`magic = "UR4SESS1"`, little-endian integer varint format carrying `schema`, `version`, `artifact_sha256`, `position`, `cache_at`, `lift_at`, `copy_scale_q16`, `logits`, `tokens`, `snap_trace`, `pointer_keys`, `pointer_lifts`, `mixture`, recurrence `state`/`history`, and read `keys`/`values`/`lifts`).
3. **Three Legs Compared:**
   - **Leg 1 — Rust Library:** `IntegerStackModel::parse` + `IntegerStackSession` + `stack_argmax` + `session_codec::{encode, decode}` in [`crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`](../../../crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs).
   - **Leg 2 — `uor-chat` CLI:** [`crates/uor-r4-integer/src/bin/uor-chat.rs`](../../../crates/uor-r4-integer/src/bin/uor-chat.rs) invoked as a child process with `--stack <artifact.lut> --tokenizer <tokenizer.json> --prompt-ids <csv> --max-tokens <N> --save-session <path> [--restore-session <path>]`, emitting `schema: "uor-r4.uor-chat-stack-parity/1"`.
   - **Leg 3 — Browser WASM & Studio Web Worker:** [`web/studio/verify_m7_parity.mjs`](../../../web/studio/verify_m7_parity.mjs) driving both the raw `wasm-pack` WebAssembly module (`uor_r4_integer_bg.wasm` + `uor_r4_integer.js`) and [`web/studio/assets/js/uor_model_worker.js`](../../../web/studio/assets/js/uor_model_worker.js) under all 6 network traps (`remote_requests == 0`).

## 2. Frozen `m7-parity-prompts-v1` Panel Results (`seed = 5` `mk-synthetic-stack`)

- **Artifact:** `synthetic-stack.lut` (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `weights_per_token = 15,968`)
- **WASM Module:** `uor_r4_integer_bg.wasm` (`381,734 B`, `sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`)

| Prompt ID | `prompt_ids` | First 8 Generated IDs (of 48) | 32-Step `UR4SESS1` Bytes & SHA-256 | 48-Step & `32 + 16` Restored `UR4SESS1` Bytes & SHA-256 | Library = CLI = WASM = Worker |
|---|---|---|---|---|---|
| `P0_bos` | `[0]` | `[38, 9, 33, 8, 25, 38, 30, 25]` | `9,782 B` `76fb6aba0ab23e8ef1e279c281332f0845ac801d51f10b3cf1bfeab838037a8a` | `14,198 B` `6df6bc6cf57c36112d4e7f866df2115f163d1288364ce198324376270fe52185` | **PASS** |
| `P1_quad` | `[0, 7, 19, 31]` | `[0, 17, 8, 25, 17, 25, 17, 8]` | `10,610 B` `ee9d5860e3439e99a22d0df0d24e8ea0d888537d6ff882ec52b3b9163340a8c8` | `15,026 B` `22c4ef314919d8d2b78f5368decc9dd1ad1280c3c4131ca37d02d5095652bdf8` | **PASS** |
| `P2_boundary` | `[0, 1, 38, 39]` | `[18, 8, 25, 17, 8, 8, 25, 17]` | `10,610 B` `59c583463c44f724e0d1fbbc3a94eabcf2b34ecb1a0f310f5b6b08ed840a4dc2` | `15,026 B` `11dcd8a10d5a6ea35b545ba7a0dde0c1533b15b7dd7613cf563b73b998bdc28c` | **PASS** |
| `P3_fibonacci` | `[0, 1, 2, 3, 5, 8, 13, 21]` | `[0, 33, 0, 17, 9, 9, 9, 9]` | `11,714 B` `7fd19c8d614d840497397ccf16ff91584d3b9067fae81754c3ad7e3e381cb3a0` | `16,130 B` `c9b94d3fd0cda2079c916416279b9238826a8e9f329ae950e734e929d3faf19f` | **PASS** |
| `P4_primes` | `[0, 2, 3, 5, 7, 11, 13, 17]` | `[33, 33, 33, 33, 33, 33, 33, 33]` | `11,714 B` `9ca83afdc339a8848e48edabfa4cc5da9326a21b4e925630681c3558101d683d` | `16,130 B` `2198c8eb51108acc743e687a85856b17cc978b8759cd350577773769bbaa3259` | **PASS** |
| `P5_alternating` | `[0, 39, 1, 38, 2, 37, 3, 36]` | `[0, 17, 8, 25, 17, 9, 30, 17]` | `11,714 B` `27b182de4a583d4e709f1782f7fe473f5a97c3208ee532554755aa46e6342b73` | `16,130 B` `8e3c75f5f1dee319cb6ef85b6c419bdc2d52a5c74c29dea19f87458e29161a87` | **PASS** |
| `P6_repeated` | `[0, 17, 17, 17, 25, 25]` | `[17, 25, 17, 25, 17, 25, 17, 25]` | `11,162 B` `bf7f1be1b91542e5799e722d9b14d35484762a1601c657c5f47b4929642d096f` | `15,578 B` `b0069321611cf77ebe160009591bcd29c9c30f0124c707704632dae53a4b382c` | **PASS** |
| `P7_octave` | `[0, 4, 8, 12, 16, 20, 24, 28]` | `[25, 17, 30, 17, 30, 17, 25, 17]` | `11,714 B` `465d0d36faa4e0707b6cef16244a916ee40458b9d22973955065f1a420cdf1fe` | `16,130 B` `f9029c5f490ccfff4950eae0072cc8f4ec78b3254c085267fa40004d105803dd` | **PASS** |

## 3. Planted-Fault Controls (`planted_fault_controls_detect_token_and_snapshot_tampering`)

1. **Single-token perturbation in CLI or Browser leg:** Flipping 1 bit of token index `15` (`tampered_tokens[15] ^= 1`) causes `check_three_way_row_parity` to reject both the CLI and Browser legs immediately.
2. **Corrupted `artifact_sha256` in `UR4SESS1` snapshot:** Flipping 1 hex character of `artifact_sha256` inside a `UR4SESS1` snapshot causes both `IntegerStackSession::restore_state` and `uor-chat --restore-session` to reject the snapshot (`ArtifactMismatch`).
3. **Truncated `UR4SESS1` stream:** Truncating 1 byte off a valid `UR4SESS1` snapshot causes both `session_codec::decode` and `uor-chat --restore-session` to reject the snapshot (`session codec: truncated input`).

## 4. Re-Run Commands & CI Coverage Note

Standard CI jobs run default workspace features and do not execute the feature-gated `--features serving-wasm` parity integration test or the Node.js browser harness (`verify_m7_parity.mjs`), so the local lab verification recorded here is the current evidence. Anyone can reproduce both checks from a clean clone with:

```sh
CARGO_TARGET_DIR=local/antigravity/target node web/studio/verify_m7_parity.mjs
CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --locked --test m7_parity_library_cli_browser
```

## 5. Staged Research Notebook Entry (`docs/research/notebook/m4-serving.md`)

> Note: `docs/research/notebook/m4-serving.md` is staged here for verbatim backfill once the umbrella item scope PR adds `notebook:` to `docs/roadmap/items/m7-parity-library-cli-browser.md`.

| Date | Mechanism / Line | Question / Hypothesis | Artifact & Seed | Result | Next Step |
|---|---|---|---|---|---|
| 2026-10-22 | M7 three-host library / CLI / browser parity harness (`#2867`, `#2035`) | Does the D11 integer stack engine (`IntegerStackModel` + `IntegerStackSession` + `stack_argmax` + `UR4SESS1` `session_codec`) produce bit-identical 48-token trajectories and byte-identical `UR4SESS1` session snapshots (both uninterrupted and `32 + 16` fresh-process save/restore) across the Rust library, the `uor-chat --stack` CLI, and the Browser WASM / Studio Web Worker across the 8-prompt `m7-parity-prompts-v1` panel? | `synthetic-stack.lut` (`469,252 B`, seed `5`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`), `uor_r4_integer_bg.wasm` (`381,734 B`, `sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`), `crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`, `web/studio/verify_m7_parity.mjs` | **Confirmed (harness verified on synthetic stand-in; no model capability claim):** `8 / 8` prompts (`384 / 384` uninterrupted tokens and `384 / 384` `32 + 16` fresh-process save/restore tokens) and all 16 `UR4SESS1` session snapshots (`9,782 B` to `16,130 B`) match bit-for-bit across the Rust library, `uor-chat` CLI, direct WASM, and `uor_model_worker.js` (`remote_requests = 0`, `weights_per_token = 15,968`), and all 3 planted-fault controls fail as expected. | Register the `m7-parity-prompts-v1` bar on `#2867` and re-run `m7_parity_library_cli_browser` on the accepted M4 `UORLUT01` model artifact once `#2857` lands. |
