# PREP Addendum for D90 Qwen3-0.6B Foundry Webharness LoRA (`d90-foundry-lora-v4`)

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Parent sheets: [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`v1`, `#4175`), [`antigravity-5/PREP.md`](../antigravity-5/PREP.md) (`v2`, `#4178`), [`antigravity-6/PREP.md`](../antigravity-6/PREP.md) (`v3`, `#4180`)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v4`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Prior Run 3 Diagnostics (`vast-62062122`, cleanly deleted, `Running: 0 pod(s)`, `~$0.52` spent; combined D90 spend `~$0.86`)

Run 3 under `d90-foundry-lora-v3` proved end-to-end LoRA Chat/Tool mastery while isolating the remaining quantization-KL gap in compiled `UORR4V13` parity (all local diagnostics preserved under `local/antigravity/d90-foundry-lora/diag_v3_int4/` and `diag_v3_i8/`):
1. **Bar #1 WASM float audit PASSED (`wasm_float_audit.json`):** `holo_inference_functions_scanned = 38`, `compute_float_ops = 0`, `js_abi_bridge_casts = 1` (`wbg_cast`/`JsValue`/`js_sys`), `holo_runtime_src_noncomment_float_lines = 0`, `passed = true`.
2. **BF16 Merged LoRA (`lora_eval.json`) PASSED Bars #2 & #3 at 100.0%:** Baseline scored `37/320` (`11.56%`) tool and `70/80` (`87.50%`) Chat; Epoch 1 (`val_loss = 1.01718`) beat Epoch 2 (`val_loss = 1.04017`) and restored Epoch 1 adapter to hit **`320/320` (`100.0%`) tool/build** and **`80/80` (`100.0%`) Chat** (`sample_failures = []`).
3. **Compiled `UORR4V13` (`int4` `down_proj`, `diag_v3_int4/`):** `tool_validity_pct = 93.75%` (`300/320`), `chat_pass_pct = 96.25%` (`77/80`), `bars_passed = true`; Stitch pack `9/9` <= `48 MiB`, `205/205` XOR exact; `foundry_v13_parity.json` (`576` pos): `top1 = 94.62%`, `top5 = 97.92%`, `mean_kl_nats = 0.155951` (recompiler OOD `0.109085`).
4. **Compiled `UORR4V13` (`--promote-down-int8`, `diag_v3_i8/`):** `tool_validity_pct = 90.62%` (`290/320`), `chat_pass_pct = 97.50%` (`78/80`), `bars_passed = true`; Stitch pack `10/10` <= `48 MiB`, `226/226` XOR exact; recompiler OOD `mean_kl_nats` dropped to **`0.075072` (`<= 0.08`)**; `foundry_v13_parity.json` (`576` val-probe pos): `top1 = 94.10%`, `top5 = 98.96%`, `mean_kl_nats = 0.110754` (`> 0.08`).
5. **Root cause of `0.110754` val-probe KL vs `0.075072` OOD KL:** `calib_foundry.txt` passed bare user prompts that `patch_recompile.py` wrapped inside `format_qwen3_prompt` (`"You are a helpful assistant."` with empty assistant reply), so CUDA Cholesky-GPTQ's Hessian `H = (2/N) X^T X` never saw Foundry's 180-token `system_prompt` or assistant `<tool_call>`/`<prism-write>`/`Chat` activations; `lora_alpha=32` also used `2.0x` adapter scaling, and `train_foundry_lora.py` evaluated teacher logits in BF16 rather than FP32.

## Bar

Every gate below is evaluated on the exact **unchanged held-out `val.jsonl` split** (`N = 400`, SHA-256 `8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), BOTH on merged BF16 (`lora_eval.json`) AND on the **compiled `UORR4V13` (`--promote-down-int8`) pure-integer runtime** (`foundry_v13_eval.json`, `foundry_v13_parity.json`), alongside `baseline_eval.json`, `foundry_v13_receipt.json`, `manifest.json`, and `manifest.stitch.json` before pod teardown:

1. **Zero neural-compute float ops in compiled WASM (`compute_float_ops == 0`, fail-closed):** `eval_foundry_v13.py --wasm target/wasm32-unknown-unknown/release/lexlean_r4_wasm.wasm --holo-src crates/wasm/src/holo_geometric_runtime.rs --out wasm_float_audit.json` confirms `compute_float_ops == 0`, `holo_inference_functions_scanned > 0`, `holo_runtime_src_noncomment_float_lines == 0`, `passed == true`.
2. **Held-out `<tool_call>` / `<prism-write>` schema validity (`>= 90%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_tool = 320` (`18` × `16` tools + `32` `<prism-write path="...">`) validated deterministically against `foundry_schemas.json`.
3. **Held-out deterministic Chat mode pass rate (`>= 95%` on both `lora_eval.json` and compiled `UORR4V13` `foundry_v13_eval.json`):** `N_chat = 80` validated by all four deterministic D28 / §3.10 checks (`20..=512` generated tokens & `>= 40` non-ws chars & clean terminator, zero spurious tool tags, non-looping 4-gram count `<= 2` & unique ratio `>= 0.55`, `>= 1` `expected_keywords` hit).
4. **Compiled `UORR4V13` teacher parity (`top-1 >= 90%`, `mean KL <= 0.08` over `576 >= 512` `val.jsonl` positions):** `foundry_v13_eval_pack --mode eval` feeds `parity_probe_tokens.json` (`16` stratified `val.jsonl` episodes × `36` positions = `576`) through `HoloV13Model::step_token(..., use_shortlist: false)` vs merged FP32 `teacher_parity_logits.bin` -> `foundry_v13_parity.json`.
5. **Stitch XOR 2-of-3 pack integrity (`foundry_pack_verify.json`):** `foundry_v13_eval_pack --mode pack` outputs `<= 50,331,648` B (`48 MiB`) `pack_XX.pack` bundles, `manifest.json` (`uor-r4.holo-pieces/1`), `manifest.stitch.json` (`stitch/xor-2-of-3/1`), 100% byte-exact XOR drop-and-recover across every group `0..parity_groups-1`, and matching `container_sha256`.

## Base artifact

- **Pretrained BF16 model:** `Qwen/Qwen3-0.6B` pinned at HF SHA `c1899de289a04d12100db370d81485cdf75e47ca`.
- **Compiler & runtime source repo:** `Casey-allard/lexlean-r4@main` `e782e7bc83bc45ab1881b13c045d39592f4cca06` (`weights-v13-qwen3-0.6b` reference `081cb4f9cdf0e4e9dd1a160fa014615cb81db10e`), Rust toolchain `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`).
- **Updated pinned local scripts under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test`, zero repo commits):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: 28907f23fc85a77ff0dcbafe7f4798dcdc88704378103197dcde5903edac0bff` (`calib_foundry.txt` upgraded to `128` stratified full ChatML `train.jsonl` episodes [`80` tool_call + `16` prism_write + `16` tool_followup + `16` Chat, zero val overlap] while preserving `train.jsonl` `79780c7e...` & `val.jsonl` `8ae0ec5c...` bit-identically)
  - `train_foundry_lora.py`: `sha256: 79e377433a670e01f2c406e82e968bf4103329e7814bc9429fb3a6c4b9d6563c` (`r=16`, `lora_alpha=16` [1.0x scaling], `epochs=2`, `lr=1.0e-4`, `lora_dropout=0.08`, `weight_decay=0.03`, best-`val_loss` adapter restore, FP32 teacher logit export matching `recompile_holo_v13`)
  - `eval_foundry_v13.py`: `sha256: 2d7bb71d352a9f259a69cfa157d611df3937e03fc705f6436786890df23416e4`
  - `foundry_v13_eval_pack.rs`: `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py`: `sha256: b91006fab20e163732df64f3d1d180a8c6fcaf60805277b1b43c47f9d48372cb` (tokenizes `<|im_start|>` ChatML calibration strings directly without wrapping in `format_qwen3_prompt`, truncated to `<= 320` tokens)

## Data split

- Deterministic seed-fixed generator (`DATA_SEED = 20261122`, `--self-test` verified `320/320` + `80/80` = `100.0%` gold pass, `train_val_overlap_hashes == 0`, `calib_foundry_count == 128`):
  - **`train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`):** Bit-identical to Run 3 (`2,400` Chat / `2,400` tool_call / `800` `<prism-write>` / `800` multi-turn).
  - **`val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`):** Bit-identical across Runs 1–4 (`320` tool/build + `80` Chat across `400/400` distinct prompts; `0` overlap).

## Steps

1. Check `uor-pod status` (`0` pods); provision `1× RTX 5090` (`uor-pod up --lab antigravity --session d90-foundry-lora-v4 --count 1 --hours 2 --no-bootstrap --purpose "D90 v4 Qwen3-0.6B Foundry LoRA + ChatML-calibrated int8-down UORR4V13"`) and arm `L-004` watcher (`schedule` 60-min timer). Copy ZERO secrets/tokens to pod (§3.13).
2. Upload the 6 pinned local files and verify all 6 SHA-256 digests; install Python deps + Rust `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`) + `wasm-tools`; clone `Casey-allard/lexlean-r4@e782e7bc`; run `patch_recompile.py`; build `lexlean_r4_wasm.wasm`, `recompile_holo_v13`, and `foundry_v13_eval_pack`.
3. Run Bar #1 fail-closed WAT audit (`wasm_float_audit.json`, `compute_float_ops == 0`, `passed == true`) and generate datasets (`train_sha256 == 79780c7e...`, `val_sha256 == 8ae0ec5c...`, `calib_foundry_count == 128`).
4. Run chained single-line script `/workspace/d90-foundry-lora/run_v4.sh` via `uor-pod run`: `train_foundry_lora.py` (`--epochs 2 --lr 1.0e-4`) -> `UOR_CUDA_ARCH=sm_120 recompile_holo_v13 --promote-down-int8 --calib-file /workspace/d90-data/calib_foundry.txt` -> `foundry_v13_eval_pack --mode eval` (`576`-pos parity + `400`-row val) -> `eval_foundry_v13.py` -> `foundry_v13_eval_pack --mode pack` -> `RUN_V4_DONE`.
5. Verify ALL Bars #1–#5 pass; archive results via `ssh -p` tar stream to `local/antigravity/d90-foundry-lora/results/`; verify local bundle integrity; IMMEDIATELY run `uor-pod release` & `uor-pod down <POD> --lab antigravity --session d90-foundry-lora-v4 --confirm-archived`, confirm `Running: 0 pod(s)`, and cancel `L-004` watcher.
6. Push orphan branch `weights-v13-qwen3-0.6b-foundry` to `https://github.com/Casey-allard/lexlean-r4.git` from the laptop only and verify HTTP `200` + matching SHA-256 on `manifest.json`, `manifest.stitch.json`, and `pack_00.pack`.

## Stop rule

- **Stop on regression or budget:** Pause, download diagnostics to `diag_v4/`, run `uor-pod down <POD> --lab antigravity --session d90-foundry-lora-v4 --confirm-archived`, confirm `Running: 0 pod(s)`, cancel `L-004` watcher, and hand back via `enter_plan_mode` if any Bar #1–#5 fails or Run 4 cost hits `$1.50` / `2.0h` (combined `< $2.36 <= $3.00` D90 cap).
- **What pass unlocks:** Hand back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and Bar #1–#5 metrics to plan Phases 3–5.
- **What fail unlocks:** Clean `main` preserved with full Run 1–4 empirical calibration/quantization diagnostics for owner review or strategy change.

## Cost

- **Target hardware:** `1× RTX 5090` (`~$0.521/h`) via `uor-pod up --count 1`.
- **Budget cap:** Hard ceiling `2.0h` / `<= $1.50` for Run 4 (expected `~35 min`, `~$0.30`; prior Runs 1–3 total `~$0.86`, keeping combined D90 spend well under `$1.30` of the `$3.00` cap). Tear down immediately after SCP/tar copy.

## Minimum test power

- Held-out `val.jsonl` keeps exact `400` distinct episodes (`320` tool/build, CI half-width `<= 3.4 pp` at `90%`; `80` Chat, CI half-width `<= 5.1 pp` around `95%`), directly matching Runs 1–3 (`8ae0ec5c...`).
- Parity probe covers `576` (`>= 512`) token positions across `16` stratified `val.jsonl` sequences + full `400`-episode `HoloV13Model::generate_chat_turn_with_history`.

## Fitness review

TEST FITNESS: FIT — reviewed at 232649b8d2b71d1b8ba095ba4c033681106817d2 by fresh-context research subagent 72cf6a39-899a-4859-9b22-524008d3aba8
