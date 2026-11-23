# PREP Addendum for D90 Qwen3-0.6B Foundry Webharness LoRA (`d90-foundry-lora-v5`)

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Parent sheets: [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`v1`, `#4175`), [`antigravity-5/PREP.md`](../antigravity-5/PREP.md) (`v2`, `#4178`), [`antigravity-6/PREP.md`](../antigravity-6/PREP.md) (`v3`, `#4180`), [`antigravity-7/PREP.md`](../antigravity-7/PREP.md) (`v4`, `#4183`)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v5`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Prior Run 4 Diagnostics & Offline Root-Cause Synthesis (`vast-62072177`, deleted, `Running: 0 pod(s)`, `~$0.27` spent; combined D90 spend `~$1.13`)

1. **Run 4 negatives (`diag_v4/`, `lora_alpha=16`, `192` base `calibration_prompts()` + `128` Foundry ChatML sequences truncated `<= 320` tokens [`320` total], `--promote-down-int8`):**
   - Bar #1 WAT audit PASSED (`38` funcs scanned, `compute_float_ops = 0`, `js_abi_bridge_casts = 1`, `0` non-comment source floats, `passed = true`) and Bar #5 Stitch XOR pack PASSED (`10/10` <= `48 MiB`, `226/226` XOR exact).
   - BF16 LoRA (`lora_eval.json`): `tool_validity_pct = 99.06%` (`317/320`), `chat_pass_pct = 95.00%` (`76/80`), `selected_epoch = 1`, `val_loss = 1.02184` (weaker than Run 3 `lora_alpha=32`: `100.0%/100.0%`, `val_loss = 1.01718`).
   - Recompiler OOD (`foundry_v13_receipt.json`): `mean_kl_nats = 0.071966` (`<= 0.08`, `top5 = 100.0%`, `top1 = 75.00%`).
   - Compiled `UORR4V13` (`foundry_v13_eval.json` & `foundry_v13_parity.json`): regressed to `86.25%` (`276/320`) tool, `88.75%` (`71/80`) Chat (`bars_passed = false`), and `576`-pos `val.jsonl` parity `top1 = 94.62%` / `top5 = 98.78%` / `mean_kl_nats = 0.143033` (`parity_bar_passed = false`).
2. **Offline synthesis across Runs 3 & 4 (`diag_v3_int4/`, `diag_v3_i8/`, `diag_v4/`, and `HoloV13Model::step_token`):**
   - Inspecting `HoloV13Model::step_token` (`DenseExact` MLP, `max_seq = 2048`, `DEFAULT_FLOCK_WINDOW = 512`, hard `token_causes_cycle` block with zero soft repetition penalty) confirmed no runtime mismatch; the `128` duplicated 180-token Foundry system prefixes in Run 4 (`<= 320` tokens) made CUDA Cholesky-GPTQ's Hessian `H = (2/N) X^T X` ill-conditioned compared with Run 3 (`diag_v3_i8/`: `90.62%` tool / `97.50%` Chat / `94.10%` top-1 / `98.96%` top-5 / `0.075072` OOD KL / `0.110754` val-probe KL).
   - Unsmoothed (`label_smoothing = 0.0`) cross-entropy SFT drives target vs distractor logit margins to extreme sharpness, amplifying quadratic KL sensitivity under 4-bit `q/k/v/o/gate/up` quantization on like-for-like 12-prompt OOD parity (`0.075072` nats in Run 3 `i8` vs `0.025384` nats for untuned `Qwen/Qwen3-0.6B` pure int4 and `0.027825` nats for `weights-v13-qwen3-0.6b` Studio LoRA SFT; 576-position `val.jsonl` probe was `0.110754` nats in Run 3 `i8` and `NOT_RUN` on those two external checkpoints).
   - **Targeted fix:** Restore Run 3's exact clean `96`-prompt `calib_foundry.txt` (`4e0127d0...`, `c76451b1...`) and `lora_alpha=32`, while adding **`label_smoothing=0.05`** exclusively to training causal-LM cross-entropy (unsmoothed `val_loss` selection preserved) plus FP32 teacher parity export and `--promote-down-int8`.

## Bar

Every gate below is evaluated on the exact **unchanged held-out `val.jsonl` split** (`N = 400`, SHA-256 `8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), BOTH on merged BF16 (`lora_eval.json`) AND on the **compiled `UORR4V13` (`--promote-down-int8`) pure-integer runtime** (`foundry_v13_eval.json`, `foundry_v13_parity.json`), alongside `baseline_eval.json`, `foundry_v13_receipt.json`, `manifest.json`, and `manifest.stitch.json` before pod teardown:

1. **Zero neural-compute float ops in compiled WASM (`compute_float_ops == 0`, fail-closed):** `eval_foundry_v13.py --wasm ... --holo-src ... --out wasm_float_audit.json` confirms `compute_float_ops == 0`, `holo_inference_functions_scanned > 0`, `holo_runtime_src_noncomment_float_lines == 0`, `passed == true`.
2. **Held-out `<tool_call>` / `<prism-write>` schema validity (`>= 90%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_tool = 320` (`18` × `16` tools + `32` `<prism-write path="...">`) validated deterministically against `foundry_schemas.json`.
3. **Held-out deterministic Chat mode pass rate (`>= 95%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_chat = 80` validated by all four deterministic D28 / §3.10 checks (`20..=512` generated tokens & `>= 40` non-ws chars & clean terminator, zero spurious tool tags, non-looping 4-gram count `<= 2` & unique ratio `>= 0.55`, `>= 1` `expected_keywords` hit).
4. **Compiled `UORR4V13` teacher parity (`top-1 >= 90%`, `mean KL <= 0.08` over `576 >= 512` `val.jsonl` positions):** `foundry_v13_eval_pack --mode eval` feeds `parity_probe_tokens.json` (`16` stratified `val.jsonl` episodes × `36` positions = `576`) through `HoloV13Model::step_token(..., use_shortlist: false)` vs merged FP32 `teacher_parity_logits.bin` -> `foundry_v13_parity.json`.
5. **Stitch XOR 2-of-3 pack integrity (`foundry_pack_verify.json`):** `foundry_v13_eval_pack --mode pack` outputs `<= 50,331,648` B (`48 MiB`) `pack_XX.pack` bundles, `manifest.json` (`uor-r4.holo-pieces/1`), `manifest.stitch.json` (`stitch/xor-2-of-3/1`), 100% byte-exact XOR drop-and-recover across every group `0..parity_groups-1`, and matching `container_sha256`.

## Base artifact

- **Pretrained BF16 model:** `Qwen/Qwen3-0.6B` pinned at HF SHA `c1899de289a04d12100db370d81485cdf75e47ca`.
- **Compiler & runtime source repo:** `Casey-allard/lexlean-r4@main` `e782e7bc83bc45ab1881b13c045d39592f4cca06` (`weights-v13-qwen3-0.6b` reference `081cb4f9cdf0e4e9dd1a160fa014615cb81db10e`), Rust toolchain `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`).
- **Pinned local scripts under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test`, zero repo commits):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: 4e0127d009163092340f9e03d33f8696cfee2b0846421ed8d5e5ec86179ed4d1` (restored Run 3 `96`-prompt `calib_foundry.txt`; bit-identical `train.jsonl` `79780c7e...` & `val.jsonl` `8ae0ec5c...`)
  - `train_foundry_lora.py`: `sha256: c043a1a91d1eb0e8e1c8b8a0ac51ba0359558b1b6d525ec30927a73237449317` (`r=16`, `lora_alpha=32`, training `label_smoothing=0.05`, `epochs=2`, `lr=1.0e-4`, `lora_dropout=0.08`, `weight_decay=0.03`, best unsmoothed `val_loss` adapter restore, FP32 parity teacher logit export)
  - `eval_foundry_v13.py`: `sha256: 2d7bb71d352a9f259a69cfa157d611df3937e03fc705f6436786890df23416e4`
  - `foundry_v13_eval_pack.rs`: `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py`: `sha256: c76451b13f5e933c5ca0bcade9fa3c989576c43767b17bac6355668d760f0b7d` (restored Run 3 `format_qwen3_prompt` `--calib-file` tokenization)

## Data split

- Deterministic seed-fixed generator (`DATA_SEED = 20261122`, `--self-test` verified `320/320` + `80/80` = `100.0%` gold pass, `train_val_overlap_hashes == 0`, `calib_foundry_count == 96`):
  - **`train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`):** Bit-identical to Runs 3–4 (`2,400` Chat / `2,400` tool_call / `800` `<prism-write>` / `800` multi-turn).
  - **`val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`):** Bit-identical across Runs 1–4 (`320` tool/build + `80` Chat across `400/400` distinct prompts; `0` overlap).

## Steps

1. Check `uor-pod status` (`0` pods); provision `1× RTX 5090` (`uor-pod up --lab antigravity --session d90-foundry-lora-v5 --count 1 --hours 2 --no-bootstrap --purpose "D90 v5 Qwen3-0.6B Foundry LoRA label_smoothing=0.05 + int8-down UORR4V13"`) and arm `L-004` watcher (`schedule` 60-min timer). Copy ZERO secrets/tokens to pod (§3.13).
2. Upload the 6 pinned local files and verify all 6 SHA-256 digests; install Python deps + Rust `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`) + `wasm-tools`; clone `Casey-allard/lexlean-r4@e782e7bc`; run `patch_recompile.py`; build `lexlean_r4_wasm.wasm`, `recompile_holo_v13`, and `foundry_v13_eval_pack`.
3. Run Bar #1 fail-closed WAT audit (`wasm_float_audit.json`, `compute_float_ops == 0`, `passed == true`) and generate datasets (`train_sha256 == 79780c7e...`, `val_sha256 == 8ae0ec5c...`, `calib_foundry_count == 96`).
4. Run chained single-line script `/workspace/d90-foundry-lora/run_v5.sh` via `uor-pod run`: `train_foundry_lora.py` (`--epochs 2 --lr 1.0e-4`) -> `UOR_CUDA_ARCH=sm_120 recompile_holo_v13 --promote-down-int8 --calib-file /workspace/d90-data/calib_foundry.txt` -> `foundry_v13_eval_pack --mode eval` (`576`-pos parity + `400`-row val) -> `eval_foundry_v13.py` -> `foundry_v13_eval_pack --mode pack` -> `RUN_V5_DONE`.
5. Verify ALL Bars #1–#5 pass; archive results via `ssh -p` tar stream to `local/antigravity/d90-foundry-lora/results/`; verify local bundle integrity; IMMEDIATELY run `uor-pod release` & `uor-pod down <POD> --lab antigravity --session d90-foundry-lora-v5 --confirm-archived`, confirm `Running: 0 pod(s)`, and `manage_task kill` the `L-004` watcher.
6. Push orphan branch `weights-v13-qwen3-0.6b-foundry` to `https://github.com/Casey-allard/lexlean-r4.git` from the laptop only and verify HTTP `200` + matching SHA-256 on `manifest.json`, `manifest.stitch.json`, and `pack_00.pack`.

## Stop rule

- **Stop on regression or budget:** Pause, download diagnostics to `diag_v5/`, release & `uor-pod down <POD> --lab antigravity --session d90-foundry-lora-v5 --confirm-archived`, confirm `Running: 0 pod(s)`, `manage_task kill` `L-004` watcher, and hand back via `enter_plan_mode` if any Bar #1–#5 fails or Run 5 cost hits `$1.20` / `2.0h` (combined `< $2.33 <= $3.00` D90 cap).
- **What pass unlocks:** Hand back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and Bar #1–#5 metrics to plan Phases 3–5.
- **What fail unlocks:** Clean `main` preserved with full Run 1–5 empirical quantization/smoothing diagnostics for owner decision or calibration-bar adjustment.

## Cost

- **Target hardware:** `1× RTX 5090` (`~$0.521/h`) via `uor-pod up --count 1`.
- **Budget cap:** Hard ceiling `2.0h` / `<= $1.20` for Run 5 (expected `~30 min`, `~$0.27`; prior Runs 1–4 total `~$1.13`, keeping combined D90 spend around `~$1.40` well inside the `$3.00` cap). Tear down immediately after `tar` copy.

## Minimum test power

- Held-out `val.jsonl` keeps exact `400` distinct episodes (`320` tool/build, CI half-width `<= 3.4 pp` at `90%`; `80` Chat, CI half-width `<= 5.1 pp` around `95%`), directly matching Runs 1–4 (`8ae0ec5c...`).
- Parity probe covers `576` (`>= 512`) token positions across `16` stratified `val.jsonl` sequences + full `400`-episode `HoloV13Model::generate_chat_turn_with_history`.

## Fitness review

TEST FITNESS: FIT — reviewed at c5ec03c900fda4f1166aa72b2a32dbf4f5798d1e by fresh-context research subagent ec743abe-b435-403e-9af7-765434d990d9
