# PREP Addendum for D90 Qwen3-0.6B Foundry Webharness LoRA (`d90-foundry-lora-v2`)

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Parent sheet: [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`d90-foundry-lora-v1`, merged in `#4175`)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v2`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Prior Run 1 Diagnostics (`vast-62052088`, cleanly terminated, `~$0.23` spent)

Run 1 under `d90-foundry-lora-v1` established three concrete facts before triggering stop/re-plan (all local diagnostics preserved under `local/antigravity/d90-foundry-lora/diag/`):
1. **WASM compute float check (`wasm_float_audit.json` + WAT lines `915915–915955`):** `37 / 37` `HoloV13Model` neural compute functions (`step_token`, `prefill_tokens_tiled`, `score_final_hidden_logits`, `attend_token_layer`, `pick_next_holo_token`, `select_mlp_shortlist_into`, `gemv_*`, `int_rms_norm_gain`, `*_with_table`, `dequant_embed_row_into`) contain **`0` float opcodes**, and `holo_geometric_runtime.rs` contains **`0` non-comment `f32`/`f64` code lines**. Only `generate_chat_turn_with_history` (`;1420;`) contained one `f64.convert_i32_u` opcode (`WAT:915934`) immediately feeding `wasm_bindgen` `wbg_cast` -> `JsValue` -> `js_sys::Function::call2` to forward the integer `step_idx` counter to JS's streaming callback.
2. **Untuned baseline (`baseline_eval.json`):** `Qwen/Qwen3-0.6B` scored `37/320` (`11.56%`) on tool/build and `70/80` (`87.50%`) on Chat.
3. **Run 1a (`3` epochs, `lr=2e-4`) & Run 1b (`1` epoch, `lr=1.5e-4`):** Run 1a overfit synthetic train phrasing by Epoch 2 (`train_loss`: `0.1640 -> 0.0068 -> 0.0061`; `val_loss`: `1.1747 -> 1.1886 -> 1.2152`) and tripped the monotonic `val_loss` stop rule. Run 1b (`1` epoch, `lr=1.5e-4`) raised tool/build validity to `306/320` (**`95.62%`** >= `90%` Bar), while Chat scored `66/80` (`82.50%`): `60/60` (`100%`) across 6 topics passed, and all `14/14` Chat misses traced to an SFT generator defect where `TRAIN_CHAT_ANSWERS` fell through to an 18-token boilerplate string on underrepresented topics (causing `token_bounds:18` and missing keywords). All `14` minor tool misses were `prismpm_inspect` (`3`) and `lexlean_check` (`9`) template phrasing ambiguity plus `2` `standards_lookup`.

## Bar

Every gate below is evaluated on the exact **unchanged held-out `val.jsonl` split** (`N = 400`, SHA-256 `8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), BOTH on merged BF16 (`lora_eval.json`) AND on the **compiled `UORR4V13` pure-integer runtime** (`foundry_v13_eval.json`, `foundry_v13_parity.json`), alongside `baseline_eval.json`, `foundry_v13_receipt.json`, `manifest.json`, and `manifest.stitch.json` before pod teardown:

1. **Zero neural-compute float ops in compiled WASM (`compute_float_ops == 0`, fail-closed):** `eval_foundry_v13.py --wasm target/wasm32-unknown-unknown/release/lexlean_r4_wasm.wasm --holo-src crates/wasm/src/holo_geometric_runtime.rs --out wasm_float_audit.json` audits WAT disassembly across all `38` `HoloV13Model` functions and Rust source lines. Only `f64.convert_i32_u` inside `generate_chat_turn_with_history` whose next 2 WAT lines contain `wbg_cast` + `JsValue` + `js_sys` is classified under `js_abi_bridge_casts`; all other float opcodes fail closed (`compute_float_ops == 0`, `holo_inference_functions_scanned > 0`, `holo_runtime_src_noncomment_float_lines == 0`).
2. **Held-out `<tool_call>` / `<prism-write>` schema validity (`>= 90%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_tool = 320` (`18` × `16` tools + `32` `<prism-write path="...">`) validated deterministically against `foundry_schemas.json`.
3. **Held-out deterministic Chat mode pass rate (`>= 95%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_chat = 80` validated by all four deterministic D28 / §3.10 checks (`20..=512` generated tokens & `>= 40` non-ws chars & clean terminator, zero spurious tool tags, non-looping 4-gram count `<= 2` & unique ratio `>= 0.55`, `>= 1` `expected_keywords` hit).
4. **Compiled `UORR4V13` teacher parity (`top-1 >= 90%`, `mean KL <= 0.08` over `576 >= 512` `val.jsonl` positions):** `foundry_v13_eval_pack --mode eval` feeds `parity_probe_tokens.json` (`16` stratified `val.jsonl` episodes × `36` positions = `576`) through `HoloV13Model::step_token(..., use_shortlist: false)` vs merged BF16 `teacher_parity_logits.bin` -> `foundry_v13_parity.json`.
5. **Stitch XOR 2-of-3 pack integrity (`foundry_pack_verify.json`):** `foundry_v13_eval_pack --mode pack` outputs `<= 50,331,648` B (`48 MiB`) `pack_XX.pack` bundles, `manifest.json` (`uor-r4.holo-pieces/1`), `manifest.stitch.json` (`stitch/xor-2-of-3/1`), 100% byte-exact XOR drop-and-recover across every group `0..parity_groups-1`, and matching `container_sha256`.

## Base artifact

- **Pretrained BF16 model:** `Qwen/Qwen3-0.6B` pinned at HF SHA `c1899de289a04d12100db370d81485cdf75e47ca`.
- **Compiler & runtime source repo:** `Casey-allard/lexlean-r4@main` `e782e7bc83bc45ab1881b13c045d39592f4cca06` (`weights-v13-qwen3-0.6b` reference `081cb4f9cdf0e4e9dd1a160fa014615cb81db10e`).
- **Updated pinned local scripts under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test`, zero repo commits):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: 7a108f492f14fa16cb3ce9e2fe07932cd02e94c31059fcfcea690415b26a76b8` (`24` dedicated `45–85`-word `CHAT_TOPICS_TRAIN` answers removing the 18-token boilerplate fallback; `TRAIN_TEMPLATES` disambiguation for `lexlean_check` & `prismpm_verify`; `DATA_SEED = 20261122`)
  - `train_foundry_lora.py`: `sha256: 1cb4f257f8c85dbfbaba935d0105bfcfeccb822ec24800e1d284acf699e22a6d` (`epochs=1`, `lr=1.2e-4`, `lora_dropout=0.10`, `weight_decay=0.05`, `TRAIN_SEED = 42`, `PYTORCH_DETERMINISTIC = 1`)
  - `eval_foundry_v13.py`: `sha256: 2d7bb71d352a9f259a69cfa157d611df3937e03fc705f6436786890df23416e4` (`compute_float_ops == 0` fail-closed audit + explicit `wbg_cast`/`JsValue`/`js_sys` ABI cast separation)
  - `foundry_v13_eval_pack.rs`: `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py`: `sha256: c76451b13f5e933c5ca0bcade9fa3c989576c43767b17bac6355668d760f0b7d`

## Data split

- Deterministic seed-fixed generator (`DATA_SEED = 20261122`, `--self-test` verified `320/320` + `80/80` = `100.0%` gold pass, `train_val_overlap_hashes == 0`):
  - **`train.jsonl` (`6,400` rows, `sha256: 68991b24ab14b6be0aa9ed4f1a3b956238fb23f996fc8ace76b78fde02bda3a6`):** `2,880` single-turn tool + `1,200` `<prism-write>` + `1,120` multi-turn `<tool_response>` + `1,200` Chat turns across `24` dedicated `>= 35`-word topics (`0` boilerplate fallback).
  - **`val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`):** **Identical** held-out split to Run 1 (`320` tool/build + `80` Chat across `400/400` distinct prompts; `0` overlap with `train.jsonl`), preserving exact baseline comparability (`11.56%` tool / `87.50%` Chat).

## Steps

1. Check `uor-pod status` (`0` pods); provision `1× RTX 5090` (`uor-pod up --lab antigravity --session d90-foundry-lora-v2 --count 1 --hours 2 --no-bootstrap`) and arm an `L-004` watcher (`schedule` 60-min timer). Copy ZERO secrets/tokens to the pod (§3.13).
2. Upload the 6 pinned local files and verify all 6 SHA-256 digests on the pod; install `transformers peft accelerate safetensors hf_transfer` + Rust `1.89` (`wasm32-unknown-unknown`) + `wasm-tools`; clone `Casey-allard/lexlean-r4@e782e7bc`; run `patch_recompile.py`; build `lexlean_r4_wasm.wasm`, `recompile_holo_v13`, and `foundry_v13_eval_pack`.
3. Run Bar #1 fail-closed WAT audit (`eval_foundry_v13.py --wasm ... --holo-src ... --out wasm_float_audit.json`) confirming `passed == true`, `compute_float_ops == 0`, `holo_runtime_src_noncomment_float_lines == 0`.
4. Run `generate_foundry_sft.py` (confirming `train_sha256 == 68991b24...` & `val_sha256 == 8ae0ec5c...`), then run `train_foundry_lora.py` (`epochs=1`, `lr=1.2e-4`, outputting `baseline_eval.json`, `lora_eval.json`, `merged_qwen3_06b/`, `parity_probe_tokens.json`, `teacher_parity_logits.bin`).
5. Run `UOR_CUDA_ARCH=sm_120 recompile_holo_v13` (`foundry_v13_receipt.json`, `raw_pieces/`), then `foundry_v13_eval_pack --mode eval` (`foundry_v13_parity.json`, `foundry_v13_predictions.jsonl`) + `eval_foundry_v13.py` (`foundry_v13_eval.json`), then `foundry_v13_eval_pack --mode pack` (`v13_pieces_foundry/`, `foundry_pack_verify.json`). Assert ALL Bars #1–#5 pass.
6. Copy `v13_pieces_foundry/` and all JSON receipts to `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/results/`, verify local SHA-256 digests, IMMEDIATELY release lease & `uor-pod down <POD> --confirm-archived`, confirm `Running: 0 pod(s)`, and cancel `L-004` watcher.
7. Push orphan branch `weights-v13-qwen3-0.6b-foundry` to `https://github.com/Casey-allard/lexlean-r4.git` from the laptop only and verify HTTP `200` + matching SHA-256 on `manifest.json`, `manifest.stitch.json`, and `pack_00.pack`.

## Stop rule

- **Stop on regression or budget:** Pause, download diagnostics, run `uor-pod down`, and hand back via `enter_plan_mode` if `lora_eval.json` or `foundry_v13_eval.json` fails Bar #2 (`>= 90%`) / Bar #3 (`>= 95%`), if `compute_float_ops != 0`, if `foundry_v13_parity.json` fails `top-1 >= 90%` or `mean KL <= 0.08`, or if Run 2 cost reaches `$2.50` / `2.0h` (total combined D90 budget `< $2.80` <= `$3.00`).
- **What pass unlocks:** Hand back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and eval metrics to plan Phases 3–5.
- **What fail unlocks:** Clean `main` preserved with full Run 1 + Run 2 diagnostics for diagnosis before further GPU spend.

## Cost

- **Target hardware:** `1× RTX 5090` (`~$0.521/h`) via `uor-pod up --count 1`.
- **Budget cap:** Hard ceiling `2.0h` / `<= $2.50` for Run 2 (expected `~25–35 min`, `~$0.25–$0.35`; Run 1 spent `~$0.23`, keeping combined D90 GPU spend well under `$1.00`). Tear down immediately after SCP.

## Minimum test power

- Held-out `val.jsonl` keeps exact `400` distinct episodes (`320` tool/build, CI half-width `<= 3.4 pp` at `90%`; `80` Chat, CI half-width `<= 5.1 pp` around `95%`), directly matching Run 1's baseline (`8ae0ec5c...`: `11.56%` tool / `87.50%` Chat).
- Parity probe covers `576` (`>= 512`) token positions across `16` stratified `val.jsonl` sequences + end-to-end `HoloV13Model::generate_chat_turn_with_history` across all `400` rows.

## Fitness review

TEST FITNESS: FIT — reviewed at f139cc2c4478c14255a9a6e8ff26bbaaa6cc013a by fresh-context research subagent f0919869-68ac-4962-b626-8e4cd6d65e90
