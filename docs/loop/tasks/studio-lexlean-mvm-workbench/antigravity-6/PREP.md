# PREP Addendum for D90 Qwen3-0.6B Foundry Webharness LoRA (`d90-foundry-lora-v3`)

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Parent sheets: [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`d90-foundry-lora-v1`, merged in `#4175`), [`antigravity-5/PREP.md`](../antigravity-5/PREP.md) (`d90-foundry-lora-v2`, merged in `#4178`)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v3`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Prior Run 2 Diagnostics (`vast-62057972`, cleanly terminated, `~$0.10` spent; combined D90 spend `~$0.34`)

Run 2 under `d90-foundry-lora-v2` established clear Bar #1 and Bar #2 passes while isolating the remaining Bar #3 Chat failure (all local diagnostics preserved under `local/antigravity/d90-foundry-lora/diag_v2/`):
1. **Bar #1 WASM float check PASSED (`wasm_float_audit.json`):** `holo_inference_functions_scanned = 38`, `compute_float_ops = 0`, `float_ops = 0`, `js_abi_bridge_casts = 1` (`generate_chat_turn_with_history` `f64.convert_i32_u` -> `wbg_cast`/`JsValue`/`js_sys`), `holo_runtime_src_noncomment_float_lines = 0`, `passed = true`.
2. **Untuned baseline (`baseline_eval.json`, `val_sha256 = 8ae0ec5c...`):** `Qwen/Qwen3-0.6B` scored `37/320` (`11.56%`) on tool/build and `70/80` (`87.50%`) on Chat.
3. **Run 2 BF16 (`1` epoch, `lr=1.2e-4`, `dropout=0.10`, `wd=0.05`, `val_loss=1.00785`):**
   - **Bar #2 Tool/Build PASSED:** `310/320` (**`96.88%`** >= `90%`). All `10` misses were two unseen phrasing pairs in `VAL_TEMPLATES`: `github_pull` (`4` rows adding an unexpected `message` arg) and `github_push` (`6` rows hallucinating `workspace_etch`/`workspace_upload`).
   - **Bar #3 Chat (`70/80`, `87.50%`, target `>= 95.0%`):** `6` of `10` Chat misses (`val_ch_000, 008, 040, 056, 064, 072`) fell on `CHAT_TOPICS_VAL[0]` (`Walk me through staying in Home Chat mode before pushing a project to the Studio builder.`), emitting `<prism-write` tags or `"No project is saved yet..."`. Root causes: (a) `CHAT_TOPICS_TRAIN[16]` included literal `` `<tool_call>` `` / `` `<tool_response>` `` tags inside a Chat target, and (b) Chat comprised only `1,200 / 6,400` (`18.75%`) of `train.jsonl` (`50` rows per topic × `1` epoch) versus `81.25%` (`5,200/6,400`) tool/build rows (`63.75%` single-turn tool + `<prism-write>`). The remaining `4` misses (`val_ch_002, 012, 049, 058`) came from under-exposure across the `1`-epoch schedule.

## Bar

Every gate below is evaluated on the exact **unchanged held-out `val.jsonl` split** (`N = 400`, SHA-256 `8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), BOTH on merged BF16 (`lora_eval.json`) AND on the **compiled `UORR4V13` pure-integer runtime** (`foundry_v13_eval.json`, `foundry_v13_parity.json`), alongside `baseline_eval.json`, `foundry_v13_receipt.json`, `manifest.json`, and `manifest.stitch.json` before pod teardown:

1. **Zero neural-compute float ops in compiled WASM (`compute_float_ops == 0`, fail-closed):** `eval_foundry_v13.py --wasm target/wasm32-unknown-unknown/release/lexlean_r4_wasm.wasm --holo-src crates/wasm/src/holo_geometric_runtime.rs --out wasm_float_audit.json` confirms `compute_float_ops == 0`, `holo_inference_functions_scanned > 0`, `holo_runtime_src_noncomment_float_lines == 0`, `passed == true`.
2. **Held-out `<tool_call>` / `<prism-write>` schema validity (`>= 90%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_tool = 320` (`18` × `16` tools + `32` `<prism-write path="...">`) validated deterministically against `foundry_schemas.json`.
3. **Held-out deterministic Chat mode pass rate (`>= 95%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_chat = 80` validated by all four deterministic D28 / §3.10 checks (`20..=512` generated tokens & `>= 40` non-ws chars & clean terminator, zero spurious tool tags, non-looping 4-gram count `<= 2` & unique ratio `>= 0.55`, `>= 1` `expected_keywords` hit).
4. **Compiled `UORR4V13` teacher parity (`top-1 >= 90%`, `mean KL <= 0.08` over `576 >= 512` `val.jsonl` positions):** `foundry_v13_eval_pack --mode eval` feeds `parity_probe_tokens.json` (`16` stratified `val.jsonl` episodes × `36` positions = `576`) through `HoloV13Model::step_token(..., use_shortlist: false)` vs merged BF16 `teacher_parity_logits.bin` -> `foundry_v13_parity.json`.
5. **Stitch XOR 2-of-3 pack integrity (`foundry_pack_verify.json`):** `foundry_v13_eval_pack --mode pack` outputs `<= 50,331,648` B (`48 MiB`) `pack_XX.pack` bundles, `manifest.json` (`uor-r4.holo-pieces/1`), `manifest.stitch.json` (`stitch/xor-2-of-3/1`), 100% byte-exact XOR drop-and-recover across every group `0..parity_groups-1`, and matching `container_sha256`.

## Base artifact

- **Pretrained BF16 model:** `Qwen/Qwen3-0.6B` pinned at HF SHA `c1899de289a04d12100db370d81485cdf75e47ca`.
- **Compiler & runtime source repo:** `Casey-allard/lexlean-r4@main` `e782e7bc83bc45ab1881b13c045d39592f4cca06` (`weights-v13-qwen3-0.6b` reference `081cb4f9cdf0e4e9dd1a160fa014615cb81db10e`), Rust toolchain `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`).
- **Updated pinned local scripts under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test`, zero repo commits):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: 4e0127d009163092340f9e03d33f8696cfee2b0846421ed8d5e5ec86179ed4d1` (`github_pull`/`github_push` expanded `2 -> 4` templates; literal XML tool tags removed from `CHAT_TOPICS_TRAIN[16]`; `5` disjoint Foundry Chat topics appended (`24 -> 29`) covering Home Chat before building, Linux vs MVM, 48 MiB packs, and `9p` `/workspace` sync; `10` Chat paraphrases; post-val `balance_rng` curriculum `2,400` Chat / `2,400` tool_call / `800` `<prism-write>` / `800` multi-turn while keeping `val.jsonl` untouched)
  - `train_foundry_lora.py`: `sha256: 544b4cb774b687f4f1ee2d0108cd94d97e32a2367e7f223015ee4e9e402bb7a6` (`epochs=2`, `lr=1.0e-4`, `lora_dropout=0.08`, `weight_decay=0.03`, best-`val_loss` adapter snapshot & restore before `merge_and_unload()`, `TRAIN_SEED = 42`, `PYTORCH_DETERMINISTIC = 1`)
  - `eval_foundry_v13.py`: `sha256: 2d7bb71d352a9f259a69cfa157d611df3937e03fc705f6436786890df23416e4`
  - `foundry_v13_eval_pack.rs`: `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py`: `sha256: c76451b13f5e933c5ca0bcade9fa3c989576c43767b17bac6355668d760f0b7d`

## Data split

- Deterministic seed-fixed generator (`DATA_SEED = 20261122`, `--self-test` verified `320/320` + `80/80` = `100.0%` gold pass, `train_val_overlap_hashes == 0`):
  - **`train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`):** `2,400` Chat (`37.5%`, `29` topics, zero literal `<tool_call>`/`<tool_response>`/`<prism-write` tags) + `2,400` single-turn tool (`150` × `16`) + `800` `<prism-write>` + `800` multi-turn (`50` × `16`).
  - **`val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`):** **Identical** held-out split to Runs 1 and 2 (`320` tool/build + `80` Chat across `400/400` distinct prompts; `0` overlap with `train.jsonl`), preserving exact baseline comparability (`11.56%` tool / `87.50%` Chat).

## Steps

1. Check `uor-pod status` (`0` pods); provision `1× RTX 5090` (`uor-pod up --lab antigravity --session d90-foundry-lora-v3 --count 1 --hours 2 --no-bootstrap --purpose "D90 v3 Qwen3-0.6B Foundry LoRA + UORR4V13 recompile"`) and arm an `L-004` watcher (`schedule` 60-min timer). Copy ZERO secrets/tokens to the pod (§3.13).
2. Upload the 6 pinned local files and verify all 6 SHA-256 digests on the pod; install `transformers peft accelerate safetensors hf_transfer` + Rust `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`) + `wasm-tools`; clone `Casey-allard/lexlean-r4@e782e7bc`; run `patch_recompile.py`; build `lexlean_r4_wasm.wasm`, `recompile_holo_v13`, and `foundry_v13_eval_pack`.
3. Run Bar #1 fail-closed WAT audit (`eval_foundry_v13.py --wasm ... --holo-src ... --out wasm_float_audit.json`) confirming `passed == true`, `compute_float_ops == 0`, `holo_runtime_src_noncomment_float_lines == 0`.
4. Run `generate_foundry_sft.py` (confirming `train_sha256 == 79780c7e...` & `val_sha256 == 8ae0ec5c...`), then run `train_foundry_lora.py` (`--epochs 2 --lr 1.0e-4`, outputting `baseline_eval.json`, `lora_eval.json`, `merged_qwen3_06b/`, `parity_probe_tokens.json`, `teacher_parity_logits.bin`).
5. Run `UOR_CUDA_ARCH=sm_120` (or `sm_89` if falling back to RTX 4090) `recompile_holo_v13 --cache-dir ... --calib-file ... --out-receipt ... --pieces-dir ...` (`foundry_v13_receipt.json`, `raw_pieces/`), then `foundry_v13_eval_pack --mode eval` (`foundry_v13_parity.json`, `foundry_v13_predictions.jsonl`) + `eval_foundry_v13.py` (`foundry_v13_eval.json`), then `foundry_v13_eval_pack --mode pack` (`v13_pieces_foundry/`, `foundry_pack_verify.json`). Assert ALL Bars #1–#5 pass.
6. Copy `v13_pieces_foundry/` and all JSON receipts to `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/results/`, verify local SHA-256 digests, IMMEDIATELY release lease & `uor-pod down <POD> --confirm-archived`, confirm `Running: 0 pod(s)`, and cancel `L-004` watcher.
7. Push orphan branch `weights-v13-qwen3-0.6b-foundry` to `https://github.com/Casey-allard/lexlean-r4.git` from the laptop only and verify HTTP `200` + matching SHA-256 on `manifest.json`, `manifest.stitch.json`, and `pack_00.pack`.

## Stop rule

- **Stop on regression or budget:** Pause, download diagnostics, release & `uor-pod down`, confirm `Running: 0 pod(s)`, cancel `L-004` watcher, and hand back via `enter_plan_mode` if `lora_eval.json` or `foundry_v13_eval.json` fails Bar #2 (`>= 90%`) / Bar #3 (`>= 95%`), if `compute_float_ops != 0`, if `foundry_v13_parity.json` fails `top-1 >= 90%` or `mean KL <= 0.08`, if `foundry_pack_verify.json` fails XOR recovery, or if Run 3 cost reaches `$2.00` / `2.0h` (combined D90 budget `< $2.50` <= `$3.00`).
- **What pass unlocks:** Hand back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and eval metrics to plan Phases 3–5.
- **What fail unlocks:** Clean `main` preserved with full Run 1 + Run 2 + Run 3 diagnostics for diagnosis before further GPU spend.

## Cost

- **Target hardware:** `1× RTX 5090` (`~$0.521/h`) via `uor-pod up --count 1`.
- **Budget cap:** Hard ceiling `2.0h` / `<= $2.00` for Run 3 (expected `~30–40 min`, `~$0.28–$0.40`; Runs 1+2 spent `~$0.34`, keeping total D90 GPU spend well under `$1.00` against `$3.00` cap). Tear down immediately after SCP.

## Minimum test power

- Held-out `val.jsonl` keeps exact `400` distinct episodes (`320` tool/build, CI half-width `<= 3.4 pp` at `90%`; `80` Chat, CI half-width `<= 5.1 pp` around `95%`), directly matching Runs 1 & 2 (`8ae0ec5c...`: `11.56%` tool / `87.50%` Chat baseline).
- Parity probe covers `576` (`>= 512`) token positions across `16` stratified `val.jsonl` sequences + end-to-end `HoloV13Model::generate_chat_turn_with_history` across all `400` rows.

## Fitness review

TEST FITNESS: FIT — reviewed at 4ed3c93a1fe1ade4f3cafa101021068442cf2678 by fresh-context research subagent d2a98584-6245-4156-aa60-ed3db6f6c710
