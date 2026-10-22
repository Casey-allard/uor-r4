# REPORT: m7-parity-library-cli-browser, antigravity-25

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

1. **No accepted M4 `UORLUT01` model artifact exists yet:** Because M1–M6 remain open (`0 / 8` milestones complete) and [`m4-export-uorlut01-cli`](../../../../roadmap/items/m4-export-uorlut01-cli.md) (`#2857`) has not yet exported a qualified model artifact, this parity harness executes against the deterministic `seed = 5` `mk-synthetic-stack` (`UORLUT01`, `469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`). Per the item's done-when, passing on the synthetic stack proves the three-host harness works and is not a model capability or milestone result.
2. **`docs/research/notebook/m4-serving.md` is outside the item's current `scope:` list:** [`docs/roadmap/items/m7-parity-library-cli-browser.md`](../../../../roadmap/items/m7-parity-library-cli-browser.md) declares `scope: [crates/uor-r4-integer/**, web/**, docs/products/**, docs/labs/m7-parity-library-cli-browser/**]` without `docs/research/notebook/m4-serving.md`. To keep `loop pr-check --local` (`scope`) green without touching out-of-scope files, the six-column research notebook row for `m4-serving.md` is staged verbatim in [`docs/labs/m7-parity-library-cli-browser/README.md`](../../../../labs/m7-parity-library-cli-browser/README.md) (`§4`) for backfill once `notebook:` lands on the item file.
3. **Deliberate-break verification (`uor-chat` token perturbation):** Temporarily flipping 1 bit of the last generated token (`*last ^= 1`) in `run_stack_chat` (`crates/uor-r4-integer/src/bin/uor-chat.rs`) and running `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --locked --test m7_parity_library_cli_browser` failed immediately with `assertion left == right failed: P0_bos: CLI 32+16 tokens != uninterrupted 48`; reverting the fault restored `2 passed; 0 failed`.

## What changed

- **Code (`148` lines changed: `+147 / -1`):**
  - [`crates/uor-r4-integer/src/bin/uor-chat.rs`](../../../../../crates/uor-r4-integer/src/bin/uor-chat.rs): Added `--prompt-ids <CSV>`, `--save-session <PATH>`, and `--restore-session <PATH>` to the `--stack` serving path (`run_stack_chat`), stepping integer token IDs through `IntegerStackSession::step` + `stack_argmax`, encoding/decoding `UR4SESS1` binary session snapshots via `uor_r4_integer::session_codec::{encode, decode}`, and emitting `schema: "uor-r4.uor-chat-stack-parity/1"` with zero floating-point operations and zero matrix multiplication.
- **Tests (`1,054` lines added: `+1,054 / -0`):**
  - [`crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`](../../../../../crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs) (`+595 / -0`): Three-leg integration test suite comparing 48-token greedy trajectories and `UR4SESS1` session snapshots (`32`-step, `48`-step uninterrupted, `32 + 16` fresh-process restore, and cross-host CLI $\to$ library restore) across all 8 prompts of `m7-parity-prompts-v1`, plus 3 planted-fault tamper controls (`planted_fault_controls_detect_token_and_snapshot_tampering`).
  - [`web/studio/verify_m7_parity.mjs`](../../../../../web/studio/verify_m7_parity.mjs) (`+459 / -0`): Self-bootstrapping Node.js driver that builds `web/studio/pkg/uor_r4_integer_bg.wasm` and `synthetic-stack.lut` when absent on a clean clone, traps all 6 browser network APIs (`remote_requests == 0`), and executes the 8-prompt uninterrupted `48`-token and `32 + 16` save/restore checks across both direct `wasm-pack` exports (`loadStack`, `step`, `saveSession`, `restoreSession`) and [`web/studio/assets/js/uor_model_worker.js`](../../../../../web/studio/assets/js/uor_model_worker.js).
- **Docs (`117` lines changed: `+115 / -2`):**
  - [`docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md`](../../../../labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md) (`+64 / -0`): One-page `DRAFT` pre-registration freezing the input SHA-256 digests, the 8-prompt `m7-parity-prompts-v1` panel, and the proposed M7 parity bar for PM registration.
  - [`docs/labs/m7-parity-library-cli-browser/README.md`](../../../../labs/m7-parity-library-cli-browser/README.md) (`+49 / -0`): Lab record, 8-row `UR4SESS1` SHA-256 parity table, planted-fault receipts, and staged `m4-serving.md` research notebook row.
  - [`docs/products/web-client.md`](../../../../products/web-client.md) (`+2 / -2`): Linked `verify_m7_parity.mjs`, `m7_parity_library_cli_browser.rs`, and the M7 parity lab record.
- **Generated (`0` lines) / Other (`0` lines):** Total `1,319` lines changed across 6 scoped files vs `ORDER.md`'s ~765-line estimate (ratio `1.72x`).

## Measurements

- **Three-host 8-prompt parity (`CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --locked --test m7_parity_library_cli_browser` & `node web/studio/verify_m7_parity.mjs`):**
  - Artifact: `seed = 5` `mk-synthetic-stack` (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `weights_per_token = 15,968`).
  - WASM binary: `uor_r4_integer_bg.wasm` (`381,734 B`, `sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`).
  - Uninterrupted 48-token parity across Rust library, `uor-chat` CLI, direct WASM, and Studio Web Worker: `8 / 8` prompts (`384 / 384` tokens, `0` discrepancies).
  - Fresh-process `32 + 16` save/restore parity (including cross-host CLI $\to$ library restore): `8 / 8` prompts (`384 / 384` tokens, `16 / 16` `UR4SESS1` snapshots byte-for-byte identical across all hosts, `9,782 B` to `16,130 B`).
  - Network isolation in Browser WASM / Studio Web Worker: `remote_requests = 0`, steady-state step allocations `allocDelta = 0`.
  - Planted-fault controls (`planted_fault_controls_detect_token_and_snapshot_tampering`): `3 / 3` tamper cases (token bit flip, corrupted `artifact_sha256`, 1-byte truncated `UR4SESS1` stream) rejected.

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo check -p uor-r4-integer --features serving-wasm --locked --all-targets`: `exit 0` (`0` warnings).
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --locked --test m7_parity_library_cli_browser`: `exit 0` (`2 passed; 0 failed`).
- `CARGO_TARGET_DIR=local/antigravity/target node web/studio/verify_m7_parity.mjs`: `exit 0` (`8 / 8` prompts pass across direct WASM and `uor_model_worker.js`, `remote_requests = 0`).
- `uor-harness loop report --check`: `exit 0`.
- `uor-harness loop pr-check --local`: `exit 0`.
- `uor-harness loop gates --full`: `exit 0` (`7 / 7` gates `GREEN`).

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `context-guard`.

## Still open

- PM registration of the proposed `m7-parity-prompts-v1` parity bar in [`preregistration-DRAFT-2026-10-22.md`](../../../../labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md) (requested in [`#2867` comment](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352106589)), backfilling the staged six-column research notebook row from [`docs/labs/m7-parity-library-cli-browser/README.md`](../../../../labs/m7-parity-library-cli-browser/README.md) (`§4`) into `docs/research/notebook/m4-serving.md` once `notebook:` is added to `m7-parity-library-cli-browser.md`, and re-running the three-host parity suite on the accepted M4 `UORLUT01` model artifact once `#2857` (`m4-export-uorlut01-cli`) produces one. References #2867.

## Plain summary

Milestone M7 requires that the same exported integer model file (`UORLUT01`, a lookup-table stack file) produce the exact same token predictions and saved conversation snapshots (`UR4SESS1`, a compact binary format for conversation state) whether it runs inside the Rust library, from the `uor-chat` command-line program, or inside a browser Web Worker using WebAssembly (sandboxed browser machine code). This task adds `--prompt-ids`, `--save-session`, and `--restore-session` flags to `uor-chat --stack`, builds the automated browser parity script (`web/studio/verify_m7_parity.mjs`), and adds a Rust integration test (`m7_parity_library_cli_browser.rs`) that runs 8 fixed prompts across all three hosts — both as uninterrupted 48-token runs and as 32-token runs saved in one process and restored in a fresh process for 16 more tokens — proving 100% bit-for-bit agreement on the synthetic test stack and rejecting tampered tokens or corrupted session files.
