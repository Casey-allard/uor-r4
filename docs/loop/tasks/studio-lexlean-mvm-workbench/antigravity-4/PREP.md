# PREP for D90 Qwen3-0.6B Foundry Webharness LoRA and UORR4V13 Compilation

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v1`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Bar

Every gate below is evaluated on the held-out `val.jsonl` split (`N = 400`, SHA-256 `8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), BOTH on merged BF16 (`lora_eval.json`) AND on the **compiled `UORR4V13` pure-integer runtime** (`foundry_v13_eval.json`, `foundry_v13_parity.json`, executing `HoloV13Model` via `foundry_v13_eval_pack`), alongside `baseline_eval.json`, `foundry_v13_receipt.json`, `manifest.json`, and `manifest.stitch.json` before pod teardown or shard publishing:

1. **Zero floating-point ops verified in compiled WASM serving (`float_ops == 0`):** `eval_foundry_v13.py --wasm target/wasm32-unknown-unknown/release/lexlean_r4_wasm.wasm --holo-src crates/wasm/src/holo_geometric_runtime.rs --out wasm_float_audit.json` scans the compiled `wasm32-unknown-unknown` WAT disassembly (`wasm-tools print`) across all `HoloV13Model` inference functions (`HoloV13Model`, `step_token`, `prefill_tokens_tiled`, `score_final_hidden_logits`, `attend_token_layer`, `pick_next_holo_token`, `select_mlp_shortlist_into`, `gemv_i4_rht*`, `gemv_i8_*`, `int_rms_norm_gain`, `silu_i32_with_table`, `exp_neg_with_table`, `dequant_embed_row_into`) and fails closed unless `holo_inference_functions_scanned > 0`, `float_ops == 0`, and `holo_runtime_src_noncomment_float_lines == 0` (independent of `recompile_holo_v13.rs`'s receipt field).
2. **Held-out `<tool_call>` / `<prism-write>` schema validity (`>= 90%` on both BF16 `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** Scored over all `320` held-out tool/build episodes (`N_tool = 320`: `288` single-turn tool calls across `16` tools + `32` `<prism-write>` builds) using `eval_foundry_v13.score_predictions` against `foundry_schemas.json` (`sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`). Valid tool calls carry well-formed `<tool_call>{"name": ..., "arguments": ...}</tool_call>` matching the tool registry (`workspace_read`, `workspace_write`, `workspace_list`, `linux_exec`, `mvm_exec`, `github_status`, `github_list_repos`, `github_clone`, `github_pull`, `github_push`, `prismpm_verify`, `lexlean_check`, `lexlean_verify`, `standards_lookup`, `stitch_fetch`, `stitch_peer_status`) and exact per-tool required keys/JSON types; valid Build edits emit non-empty `<prism-write path="...">...</prism-write>` (`>= 24` chars body).
3. **Held-out deterministic Chat mode pass rate (`>= 95%` on both BF16 `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`, zero neural graders per D28 / §3.10):** Scored over all `80` held-out Home Chat episodes (`N_chat = 80`), passing iff ALL four deterministic checks hold:
   - Valid UTF-8 text (`20..=512` generated tokens, `>= 40` non-whitespace chars, zero `\uFFFD`) ending with EOS or terminal punctuation (`.`, `!`, `?`, `` ` ``, `"`, `'`, `)`, ```` ``` ````).
   - Zero spurious `<tool_call>`, `</tool_call>`, or `<prism-write` tags.
   - Non-looping bound: maximum identical whitespace 4-gram count `<= 2` AND distinct 4-gram ratio `>= 0.55`.
   - Deterministic topic grounding: case-folded response matches `>= 1` anchor from `episode.expected_keywords`.
4. **Compiled `UORR4V13` teacher parity (`top-1 >= 90%`, `mean KL <= 0.08` over `>= 512` `val.jsonl` positions):** `foundry_v13_eval_pack --mode eval` feeds `parity_probe_tokens.json` (`576` assistant+context token positions = `16` stratified `val.jsonl` episodes [`12` tool/build + `4` Chat] × `36` trailing positions each) through `HoloV13Model::step_token(..., use_shortlist: false)` and checks `top1_parity_pct >= 90.0` and `mean_kl_nats <= 0.08` against BF16 merged `teacher_parity_logits.bin` in `foundry_v13_parity.json` (in addition to `recompile_holo_v13.rs`'s 12-prompt GPU receipt `foundry_v13_receipt.json`).
5. **Stitch XOR 2-of-3 pack integrity (`foundry_pack_verify.json`):** `foundry_v13_eval_pack --mode pack` bundles `48` pieces (`<= 50,331,648` B / `48 MiB`) per `pack_XX.pack`, emits canonical `manifest.json` (`uor-r4.holo-pieces/1` with `"packs"` + `"pieces"`) and `manifest.stitch.json` (`stitch/xor-2-of-3/1`), verifies byte-exact single-piece drop-and-recover across EVERY parity group `0..parity_groups-1` directly from `pack_XX.pack` slices + `parity_XXXX.bin`, and verifies `container_sha256` matches across `manifest.json`, `manifest.stitch.json`, `foundry_v13_receipt.json`, `foundry_v13_parity.json`, and `foundry_pack_verify.json`.

## Base artifact

- **Pretrained BF16 model:** `Qwen/Qwen3-0.6B` pinned at HuggingFace commit SHA `c1899de289a04d12100db370d81485cdf75e47ca`.
- **Compiler & runtime source repo:** `Casey-allard/lexlean-r4` pinned at `main` commit SHA `e782e7bc83bc45ab1881b13c045d39592f4cca06` (`gpu/recompile_holo_v13.rs` + `crates/wasm/src/holo_geometric_runtime.rs`), plus reference shard/pack layout at `weights-v13-qwen3-0.6b` (`081cb4f9cdf0e4e9dd1a160fa014615cb81db10e`).
- **Pinned local scripts & Rust driver under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test`, zero repository commits to `UOR-Foundation/uor-r4`):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: c5f993156ca157527eba81da6050130f7a6b5c1a5a9a4640578b619f9542e897` (`DATA_SEED = 20261122`)
  - `train_foundry_lora.py`: `sha256: a6c78fdaea4f8fd7a9a417912c46f7d053dddb9748f7b2497307a1f16ca97224` (`TRAIN_SEED = 42`, `PYTORCH_DETERMINISTIC = 1`)
  - `eval_foundry_v13.py`: `sha256: 991f0b26a181a5d2c98b87d147975b56d2c59cd0c322da9e364ce0ed04ad4846`
  - `foundry_v13_eval_pack.rs` (built on pod inside cloned `lexlean-r4/crates/wasm/src/bin/`): `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py` (adds `UOR_CUDA_ARCH` dynamic `sm_89`/`sm_120` flag + `--calib-file` to pod clone of `gpu/recompile_holo_v13.rs`, then copied to `crates/wasm/src/bin/recompile_holo_v13.rs`): `sha256: c76451b13f5e933c5ca0bcade9fa3c989576c43767b17bac6355668d760f0b7d`
- **Workspace baseline:** `UOR-Foundation/uor-r4` PR #4175 (`antigravity/d90-studio-overhaul` branched from `main` `3c01e23fd`), zero changes under `crates/**`.

## Data split

- `generate_foundry_sft.py` (`DATA_SEED = 20261122`) generates strictly disjoint pools (zero shared paths, repos, branches, modules, queries, URLs, peers, commands, or prompt templates) and passes `--self-test` (`100.0%` gold tool & Chat pass, `0` overlap):
  - **`train.jsonl`:** `6,400` ChatML episodes (`sha256: 4fa37181f8542feac84371ce124ade0fde6bd872fafddd374cc5317e5ffc73cb`): `2,880` single-turn `<tool_call>` (`180` × `16` tools), `1,200` `<prism-write path="...">`, `1,120` multi-turn `<tool_call>` -> `<tool_response>` trajectories, and `1,200` Home Chat turns.
  - **`val.jsonl`:** `400` unique held-out ChatML episodes (`sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`): `320` tool/build (`18` × `16` tools = `288` + `32` `<prism-write>`) + `80` Home Chat episodes with deterministic `expected_keywords`; verified `train_val_overlap_hashes == 0` and `400/400` distinct validation prompts.
  - **`calib_foundry.txt`:** `96` Foundry prompts drawn strictly from `train.jsonl` (`0` overlap with `val.jsonl`) appended to `calibration_prompts()` via `--calib-file`.
  - **`parity_probe_prompts.json`:** Fixed `16`-episode stratified slice of `val.jsonl` (`11` `<tool_call>` + `1` `<prism-write>` + `4` Chat) yielding `576` (`>= 512`) token parity positions.

## Steps

1. Run `bash scripts/pod/uor-pod status`; provision one single-GPU pod via `uor-pod up --lab antigravity --session d90-foundry-lora --count 1 --hours 2 --purpose "D90 Qwen3-0.6B Foundry webharness LoRA + UORR4V13 recompile"` (auto-creates exclusive 1-GPU lease; prefer RTX 4090 or RTX 5090 with `UOR_CUDA_ARCH=sm_<compute_cap>`). Copy ZERO tokens, PATs, or API keys onto the pod (§3.13). Arm an `L-004` watcher (`schedule` timer) so no idle pod persists past completion or `2.0h`.
2. On the pod, clone `https://github.com/Casey-allard/lexlean-r4.git` at pinned `e782e7bc83bc45ab1881b13c045d39592f4cca06`, apply `patch_recompile.py` to `gpu/recompile_holo_v13.rs`, copy patched `gpu/recompile_holo_v13.rs` -> `crates/wasm/src/bin/recompile_holo_v13.rs` and `foundry_v13_eval_pack.rs` -> `crates/wasm/src/bin/foundry_v13_eval_pack.rs`, upload the local scripts (`generate_foundry_sft.py`, `foundry_schemas.json`, `train_foundry_lora.py`, `eval_foundry_v13.py`), install `wasm-tools` if absent (`cargo install --locked wasm-tools`), build WASM (`cargo build --locked --release -p lexlean-r4-wasm --target wasm32-unknown-unknown`) + offline native bins (`cargo build --locked --release -p lexlean-r4-wasm --bin recompile_holo_v13 --bin foundry_v13_eval_pack`), and run Bar #1 fail-closed audit: `python3 eval_foundry_v13.py --wasm target/wasm32-unknown-unknown/release/lexlean_r4_wasm.wasm --holo-src crates/wasm/src/holo_geometric_runtime.rs --out wasm_float_audit.json`.
3. Generate `train.jsonl`, `val.jsonl`, `calib_foundry.txt`, and `parity_probe_prompts.json` (`DATA_SEED = 20261122`, asserting exact expected SHA-256s `4fa37181...` / `8ae0ec5c...` and `0` overlap).
4. Run `train_foundry_lora.py`: measure untuned `Qwen/Qwen3-0.6B` (`@ c1899de2`) baseline -> `baseline_eval.json`, train BF16 LoRA (`TRAIN_SEED = 42`, `r=16`, `alpha=32`, exact `<think>\n\n</think>\n\n` ChatML alignment), merge via `merge_and_unload()` into `merged_qwen3_06b/`, evaluate BF16 on `val.jsonl` (`max_new_tokens=192`) -> `lora_eval.json`, and dump BF16 teacher logits on `576` `parity_probe_tokens.json` positions -> `teacher_parity_logits.bin`.
5. Run built `target/release/recompile_holo_v13` (`--cache-dir merged_qwen3_06b --calib-file calib_foundry.txt --out-receipt foundry_v13_receipt.json --pieces-dir raw_pieces`, with `UOR_CUDA_ARCH=sm_<compute_cap>`).
6. Run `foundry_v13_eval_pack --mode eval` (checking `top-1 >= 90%`, `mean KL <= 0.08` over `576 >= 512` `val.jsonl` positions -> `foundry_v13_parity.json`, and running parallel `HoloV13Model::generate_chat_turn_with_history` across all `400` `val.jsonl` episodes -> `foundry_v13_predictions.jsonl`) followed by `eval_foundry_v13.py` (`foundry_v13_eval.json`).
7. Run `foundry_v13_eval_pack --mode pack` producing `v13_pieces_foundry/` (`pack_XX.pack <= 48 MiB`, `parity_XXXX.bin`, `manifest.json`, `manifest.stitch.json`) and verifying byte-exact XOR recovery on 100% of parity groups -> `foundry_pack_verify.json`. Assert ALL 5 Bar gates pass before teardown.
8. Copy `v13_pieces_foundry/`, `baseline_eval.json`, `lora_eval.json`, `foundry_v13_receipt.json`, `foundry_v13_parity.json`, `foundry_v13_eval.json`, `foundry_pack_verify.json`, and WASM float-audit JSON to `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/`; immediately release lease & run `uor-pod down <POD>`, confirming `Running: 0 pod(s)` in `uor-pod status`.
9. From the laptop only, push orphan branch `weights-v13-qwen3-0.6b-foundry` to `https://github.com/Casey-allard/lexlean-r4.git` (asserting every tracked file `< 50 MiB`) and verify HTTP `200` + matching SHA-256 on `manifest.json`, `manifest.stitch.json`, and `pack_00.pack`.

## Stop rule

- **Stop on regression or budget:** Pause and inspect (never leave an idle pod billing) if `val.jsonl` loss rises over two consecutive evaluations, if held-out `<tool_call>` / `<prism-write>` validity or Chat pass rate falls below Bar #2/`#3` on either `lora_eval.json` or compiled `foundry_v13_eval.json`, if `eval_foundry_v13.py` finds `float_ops != 0` in WASM WAT, if `foundry_v13_parity.json` fails `top-1 >= 90%` or `mean KL <= 0.08`, or if spend reaches `$3.00` / `2.0h`. Allow at most one bounded SFT/LR adjustment inside the `$3.00` ceiling; on a second failure, tear down the pod (`uor-pod down`) and hand back via `enter_plan_mode`.
- **What pass unlocks:** Handing back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and eval metrics to plan Phases 3–5 (`web/studio/` `lexlean-r4` WASM port, `afflom/stitch` Linux 6.18.7 `v86` + Stitch WebRTC/`afflom/me`, ChaCha20-Poly1305 GitHub vault, Chrome E2E, `gh-pages` `/studio/` publish).
- **What fail unlocks:** Preserves clean `main` (`web/studio/` unchanged) and triggers investigation/re-plan via `enter_plan_mode` before any UI integration or further GPU spend.

## Cost

- **Target hardware:** `1× RTX 5090` (`~$0.56/h`) or `1× RTX 4090` (`~$0.87/h`) through `scripts/pod/uor-pod --count 1` (strictly inside the `4` pod / `$8.00/h` fleet cap).
- **Budget cap:** Hard ceiling `2.0 hours` wall clock and `<= $3.00` total spend; expected run time `30–50 minutes` (`~$0.30–$0.75`). Tear down immediately after SCP copy.

## Minimum test power

- `val.jsonl` contains `400` distinct held-out episodes (`320` tool/build + `80` Chat), giving `<= 2.5 percentage point` binomial SE (worst case at `p = 0.5`) and `95%` Wilson CI half-width `<= 3.4 pp` at `90%` (`N = 320`) and `<= 5.1 pp` around `95%` (`N = 80`), compared directly against measured untuned `baseline_eval.json`.
- `foundry_v13_eval_pack --mode eval` tests `576` (`>= 512`) token parity positions across `16` stratified `val.jsonl` sequences against BF16 teacher logits AND executes full autoregressive `HoloV13Model::generate_chat_turn_with_history` across all `400` held-out `val.jsonl` rows.

## Fitness review

TEST FITNESS: FIT — reviewed at 4e1f9e118c0e1fdcedd2ff75edddf76a576fb908 by fresh-context research subagent bf39fabc-e1e5-4ce8-b071-e40c1c6e0ea9
