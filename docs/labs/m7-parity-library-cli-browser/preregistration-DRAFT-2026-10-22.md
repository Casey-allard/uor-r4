# Pre-Registration (`DRAFT`): M7 Three-Host Bit-Identity & Session-Snapshot Parity (`m7-parity-library-cli-browser`, Issue `#2867`)

- **Date:** 2026-10-22
- **Lab:** `antigravity`
- **Milestone / Issue:** M7 (`#2035`), item [`m7-parity-library-cli-browser`](../../roadmap/items/m7-parity-library-cli-browser.md) (`#2867`)
- **Bar Registration Request:** [`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431) (superseding [`#2867` comment `6352106589`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352106589))
- **Status:** `DRAFT` (`PROPOSED, NOT REGISTERED` — awaiting PM / owner bar registration on `#2867` and an accepted M4 `UORLUT01` model artifact from `#2857`)

## 1. Bar (Proposed for Registration)

- **Target:** M7 (`#2035`) closing parity bar (`PROPOSED, NOT REGISTERED`):
  1. **Uninterrupted 48-token bit-identity (`8 / 8` prompts):** Across all 8 prompts in the frozen `m7-parity-prompts-v1` panel, 48 greedy continuation tokens (`stack_argmax`, temperature `0`, first index on ties) and the resulting 48-step `UR4SESS1` binary session snapshot (`uor_r4_integer::session_codec::encode`) are 100% bit-identical (`0` token discrepancies out of `384` generated tokens; `0` byte discrepancies across all 8 `UR4SESS1` snapshots) across:
     - **Leg 1 (Rust library):** `uor_r4_integer::stack::IntegerStackModel::parse` + `IntegerStackSession::step` + `stack_argmax` + `session_codec::{encode, decode}`.
     - **Leg 2 (`uor-chat` CLI):** `uor-chat --stack <artifact.lut> --tokenizer <tokenizer.json> --prompt-ids <csv> --max-tokens <N> --save-session <path> [--restore-session <path>]` (`crates/uor-r4-integer/src/bin/uor-chat.rs`).
     - **Leg 3 (Browser WASM & Studio Web Worker):** `crates/uor-r4-integer` (`--features serving-wasm`, `wasm32-unknown-unknown`) via both direct `wasm-pack` exports (`loadStack`, `step`, `saveSession`, `restoreSession`) and `web/studio/assets/js/uor_model_worker.js` (`load_model`, `generate`, `export_session`, `import_session`).
  2. **Fresh-process `32 + 16` save/restore continuation (`8 / 8` prompts):** Saving a `UR4SESS1` snapshot after 32 generated tokens in process/instance A and restoring it into a fresh process/instance B (plus cross-host CLI $\to$ library restore) to generate 16 additional tokens yields the exact 48-token trajectory and byte-identical final 48-step `UR4SESS1` snapshot of the uninterrupted 48-token run in all three legs.
  3. **Zero remote model calls & static WASM gate:** `remote_requests == 0` under all 6 network traps (`fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `navigator.sendBeacon`, `importScripts`) and `7 / 7 PASS` (`mul=0, div=0, float=0, unclassified=0`) on `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`.

## 2. Base Artifact & Frozen Inputs

| Input | Path / Generator | Size (B) | SHA-256 |
|---|---|---|---|
| Synthetic `UORLUT01` stack fixture (`seed = 5`) | `cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- 5 synthetic-stack.lut` | `469,252` | `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df` |
| WebAssembly serving binary (`-C target-feature=-simd128`) | `wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm` | `381,734` | `656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356` |
| Deterministic 40-token byte-BPE tokenizer (`vocab = 40`) | `build_synthetic_40_tokenizer_json()` in [`m7_parity_library_cli_browser.rs`](../../../crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs) | `1,303` | `d764db72c83530cf8cb29e1268b863e79b1df39c2557396a2a01fc5e87809142` |
| Browser parity driver | [`web/studio/verify_m7_parity.mjs`](../../../web/studio/verify_m7_parity.mjs) | — | Tracked in git |
| Three-host Rust integration suite | [`crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`](../../../crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs) | — | Tracked in git |

## 3. Data Split (`m7-parity-prompts-v1`)

8 deterministic prompt sequences spanning length `1..=8` across vocabulary `0..40` (held fixed for harness verification on the synthetic stand-in; when re-run on the accepted M4 `UORLUT01` artifact, the same 8 token-ID sequences plus 8 literal-role text turns under `uor-r4.literal-role-dialogue/2` will execute):

| Prompt ID | Token IDs (`prompt_ids`) | Prompt Length | Uninterrupted `UR4SESS1` Bytes (32 / 48 steps) |
|---|---|---|---|
| `P0_bos` | `[0]` | 1 | `9,782` / `14,198` |
| `P1_quad` | `[0, 7, 19, 31]` | 4 | `10,610` / `15,026` |
| `P2_boundary` | `[0, 1, 38, 39]` | 4 | `10,610` / `15,026` |
| `P3_fibonacci` | `[0, 1, 2, 3, 5, 8, 13, 21]` | 8 | `11,714` / `16,130` |
| `P4_primes` | `[0, 2, 3, 5, 7, 11, 13, 17]` | 8 | `11,714` / `16,130` |
| `P5_alternating` | `[0, 39, 1, 38, 2, 37, 3, 36]` | 8 | `11,714` / `16,130` |
| `P6_repeated` | `[0, 17, 17, 17, 25, 25]` | 6 | `11,162` / `15,578` |
| `P7_octave` | `[0, 4, 8, 12, 16, 20, 24, 28]` | 8 | `11,714` / `16,130` |

## 4. Steps

1. Build or verify the `wasm-pack` WebAssembly module (`web/studio/pkg/uor_r4_integer_bg.wasm`) and `seed = 5` `mk-synthetic-stack` artifact (`synthetic-stack.lut`).
2. Run `node web/studio/verify_m7_parity.mjs` to execute all 8 prompts in both direct WASM and the Studio Web Worker (`uor_model_worker.js`).
3. Run `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --locked --test m7_parity_library_cli_browser` to execute all 8 prompts across the Rust library, `uor-chat` CLI subprocesses, and Browser WASM / Studio Web Worker, plus the planted-fault tamper controls.

## 5. Stop Conditions

- Any single token discrepancy (`1 / 384`) or `UR4SESS1` byte discrepancy across the three legs halts the test immediately.
- Any planted-fault control that fails to reject a tampered token trajectory, corrupted `artifact_sha256`, or truncated `UR4SESS1` stream halts the test immediately.

## 6. Cost

- `$0` GPU / cloud spend (100% deterministic local CPU/WASM host verification of client serving parity per CHARTER §3.6).

## 7. Minimum Test Power

- `8` prompts $\times$ `48` generated tokens = `384` greedy argmax steps per leg (`1,152` steps across the 3 legs for uninterrupted runs + `1,152` steps across the `32 + 16` fresh-process save/restore runs), comparing full `UR4SESS1` binary session state (`9,782` to `16,130` bytes per snapshot, covering `position`, `cache_at`, `lift_at`, `logits`, `tokens`, recurrence `state`/`history`, and Lorentz read `keys`/`values`/`lifts`).

## 8. Verdict

- `TEST FITNESS: PENDING` (awaiting non-author review and PM bar registration on `#2867`).
