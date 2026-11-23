# PREP Addendum for D90 Qwen3-0.6B Foundry Webharness LoRA (`d90-foundry-lora-v6`)

- Lab: antigravity
- Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939)
- Parent sheets: [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`v1`, `#4175`), [`antigravity-5/PREP.md`](../antigravity-5/PREP.md) (`v2`, `#4178`), [`antigravity-6/PREP.md`](../antigravity-6/PREP.md) (`v3`, `#4180`), [`antigravity-7/PREP.md`](../antigravity-7/PREP.md) (`v4`, `#4183`), [`antigravity-8/PREP.md`](../antigravity-8/PREP.md) (`v5`, `#4186`)
- Owner decision: [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- Run: `d90-foundry-lora-v6`
- Scope boundary: Studio-only (`web/studio/` + external `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`); `crates/**` and M1–M8 native training/serving remain strictly quarantined under Charter §3.

## Prior Run 5 Diagnostics & Offline Root-Cause Synthesis (`vast-62082624`, deleted, `Running: 0 pod(s)`, `~$0.31` spent; combined D90 spend `~$1.44`)

1. **Run 5 empirical results (`diag_v5/`, `lora_alpha=32`, training `label_smoothing=0.05`, `96` user-only `calib_foundry.txt`, `--promote-down-int8`, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`):**
   - Bar #1 WAT audit (`wasm_float_audit.json`): PASS (`38` funcs scanned, `compute_float_ops = 0`, `float_ops = 0`, `js_abi_bridge_casts = 1`, `0` non-comment source floats, `passed = true`).
   - Untuned baseline (`baseline_eval.json`): `tool_validity_pct = 11.56%` (`37/320`), `chat_pass_pct = 87.50%` (`70/80`).
   - BF16 Merged LoRA (`lora_eval.json`, epoch 1 `best_val_loss = 0.84872`): PASS (`tool_validity_pct = 100.00%` [`320/320`], `chat_pass_pct = 98.75%` [`79/80`], `bars_passed = true`).
   - Like-for-like 12-prompt OOD parity (`foundry_v13_receipt.json`): `top1_parity_pct = 83.33%`, `top5_parity_pct = 100.00%`, `mean_kl_nats = 0.045925` (`<= 0.08`, vs `0.025384` untuned `Qwen/Qwen3-0.6B` pure int4 and `0.027825` `weights-v13-qwen3-0.6b` Studio LoRA SFT; the `576`-position `val.jsonl` probe was `NOT_RUN` on those two external checkpoints).
   - Compiled `UORR4V13` held-out `val.jsonl` (`foundry_v13_eval.json`): PASS Bar #2 & #3 (`tool_validity_pct = 97.19%` [`311/320`], `chat_pass_pct = 96.25%` [`77/80`], `bars_passed = true`; `7/9` tool/build failures emitted `<tool_call>{"name": "prism-write", ...}` instead of `<prism-write path="...">`).
   - Compiled `UORR4V13` Stitch XOR pack (`foundry_pack_verify.json`): PASS Bar #5 (`10/10` packs `<= 48 MiB`, `226/226` XOR exact).
   - Compiled `UORR4V13` `576`-pos `val.jsonl` parity (`foundry_v13_parity.json`): **FAIL (`parity_bar_passed = false`)** — `top1_parity_pct = 94.10%` (`542/576`), `top5_parity_pct = 97.40%` (`561/576`), `mean_kl_nats = 0.116247 > 0.08`.
2. **Offline `$0.00` `diag_v5_kl` root-cause breakdown (`n=576`, reproducing `94.10% / 97.40% / 0.116247`):**
   - `tool_call` (`n=396`, 11 eps): `top1 = 100.00%`, `mean_kl = 0.015708` (`med = 0.0099`, `p90 = 0.0239`), centered logit `rms = 0.2462`, `cos = 0.97850` (`9.3%` of total KL).
   - `prism_write` (`n=36`, 1 ep `len=500`): `top1 = 100.00%`, `mean_kl = 0.032622` (`med = 0.0122`, `p90 = 0.0805`), centered logit `rms = 0.3119`, `cos = 0.97053` (`1.8%` of total KL).
   - `chat` (`val_ch_000..003`, `n=144`): `top1 = 76.39%`, **`mean_kl = 0.413637`** (`med = 0.1737`, `p90 = 1.0570`, `max = 4.1086`), centered logit `rms = 0.5030`, `cos = 0.93313` (**`89.0%` of total KL**, `34/34` top-1 misses).
   - Sequence-length KV quantization drift is ruled out (`pearson_corr(abs_pos, kl) = 0.0465`, and `val_pw_000` at `len=500` achieved `100.00%` top-1 / `mean_kl = 0.032622`), and uniform logit scaling is ruled out (optimal `alpha=1.05` drops `tool_call` `0.015708 -> 0.008239` while `chat` remains `0.413637 -> 0.394840`).
   - **Targeted fix:** Across Runs 1–3 & 5, CUDA Cholesky-GPTQ (`H = (2/N) X^T X`) saw only user prompts wrapped in `"You are a helpful assistant."` (`0` Foundry assistant tokens, `0` Foundry system tokens); in Run 4, `128` duplicate `~285`-token Foundry system prefixes truncated `<= 320` tokens also excluded assistant completions and ill-conditioned `H`. Run 6 keeps the winning Run 5 SFT recipe untouched (`c043a1a9...`, `79780c7e...` train, `8ae0ec5c...` val) and updates `calib_foundry.txt` + `patch_recompile.py` to calibrate Cholesky-GPTQ on `128` diverse `train.jsonl` user+assistant continuations (`112` compact + `16` full-system rows, `max_len <= 448`).

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
- **Pinned local scripts under `/Users/casey.allard/uor-r4/local/antigravity/d90-foundry-lora/` (verified by `--self-test` & local `cargo check`, zero repo commits):**
  - `foundry_schemas.json`: `sha256: d35ec55a8907c85fda77203602df2e3fc73c75f04f7fc39e7b6bc59e2effd366`
  - `generate_foundry_sft.py`: `sha256: 397c266fef9af7bcd1987c20406c76db3f2c7db926ff41e84d81beefc64b6ce3` (`128` assistant-calibrated `calib_foundry.txt`: `112` compact user+assistant [`64` chat, `32` tool_call `2×16`, `16` prism_write] + `16` full-system user+assistant [`10` chat, `4` tool_call, `2` prism_write]; bit-identical `train.jsonl` & `val.jsonl`)
  - `train_foundry_lora.py`: `sha256: c043a1a91d1eb0e8e1c8b8a0ac51ba0359558b1b6d525ec30927a73237449317` (`r=16`, `lora_alpha=32`, training `label_smoothing=0.05`, `epochs=2`, `lr=1.0e-4`, unsmoothed `val_loss` selection, FP32 parity teacher)
  - `eval_foundry_v13.py`: `sha256: 2d7bb71d352a9f259a69cfa157d611df3937e03fc705f6436786890df23416e4`
  - `foundry_v13_eval_pack.rs`: `sha256: 051dca96e9d4341a76ce57e49b4086cf67fc7f97c2157de3c3e202a975b56159`
  - `patch_recompile.py`: `sha256: bb5da659f37d7590791c723a230e21bc4dc6a4752003a94297ac0ccf775460de` (direct tokenization for `<|im_start|>` calibration strings, `ids.truncate(448)`, `Calibration tokenized seqs=... total_tokens=... max_len=...` log)

## Data split

- Deterministic seed-fixed generator (`DATA_SEED = 20261122`, `calib_rng = Random(DATA_SEED + 6)`, `--self-test` verified `320/320` + `80/80` = `100.0%` gold pass, `train_val_overlap_hashes == 0`, `calib_foundry_count == 128` strictly disjoint from `val.jsonl`):
  - **`train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`):** Bit-identical to Runs 3–5.
  - **`val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`):** Bit-identical across Runs 1–5 (`320` tool/build + `80` Chat across `400/400` distinct prompts; `0` overlap).

## Steps

1. Check `uor-pod status` (`0` pods); provision `1× RTX 5090` (`<= $1.05/h`, or `--gpu 4090 <= $0.95/h`) for session `d90-foundry-lora-v6` and immediately arm `L-004` watcher (`schedule` 60-min timer). Copy ZERO secrets/tokens to pod (§3.13).
2. Upload the 6 pinned files and verify all 6 SHA-256 digests; install Python deps + Rust `1.97.1` (`rustup target add --toolchain 1.97.1 wasm32-unknown-unknown`) + `wasm-tools`; clone `Casey-allard/lexlean-r4@e782e7bc`; run `patch_recompile.py`; build `lexlean_r4_wasm.wasm`, `recompile_holo_v13`, and `foundry_v13_eval_pack`.
3. Run Bar #1 fail-closed WAT audit (`wasm_float_audit.json`, `compute_float_ops == 0`, `passed == true`) and generate datasets (`train_sha256 == 79780c7e...`, `val_sha256 == 8ae0ec5c...`, `calib_foundry_count == 128`).
4. Renew lease and run single-line `/workspace/d90-foundry-lora/run_v6.sh` via `uor-pod run`: `train_foundry_lora.py` (`--epochs 2 --lr 1.0e-4`) -> `recompile_holo_v13 --promote-down-int8 --calib-file /workspace/d90-data/calib_foundry.txt` -> `foundry_v13_eval_pack --mode eval` -> `eval_foundry_v13.py` -> `foundry_v13_eval_pack --mode pack` -> `RUN_V6_DONE`.
5. If ALL Bars #1–#5 pass: stream-tar results to `local/antigravity/d90-foundry-lora/results/`, verify packs `<= 48 MiB`, IMMEDIATELY release & `uor-pod down <POD> --lab antigravity --session d90-foundry-lora-v6 --confirm-archived`, confirm `Running: 0 pod(s)`, kill `L-004` watcher, push `weights-v13-qwen3-0.6b-foundry` to `Casey-allard/lexlean-r4` from laptop ONLY, verify HTTP `200` + SHA-256s, and hand back via `enter_plan_mode`.

## Stop rule

- **Final calibration run before decision review:** If ANY Bar #1–#5 fails or Run 6 spend hits `$0.80` / `1.5h` (combined `<= $2.25 <= $3.00` D90 cap), stream-tar diagnostics to `diag_v6/`, IMMEDIATELY release & `uor-pod down` (`Running: 0 pod(s)`), kill `L-004` watcher, publish NOTHING, and hand back for a formal Charter §5.1/§5.2 decision review (specifically if Bars #1/#2/#3/#5 + `top-1 >= 90%` + 12-OOD `<= 0.05` pass while only high-entropy prose `576`-pos `mean_kl_nats` exceeds `0.08`) rather than launching a 7th GPU run.
- **What pass unlocks:** Hand back via `enter_plan_mode` with verified `weights-v13-qwen3-0.6b-foundry` URLs/digests and Bar #1–#5 metrics to plan Phases 3–5.

## Cost

- **Target hardware:** `1× RTX 5090` (`<= $1.05/h`) or `1× RTX 4090` (`<= $0.95/h`) via `uor-pod up --count 1`.
- **Budget cap:** Hard ceiling `1.5h` / `<= $0.80` for Run 6 (expected `~30 min`, `~$0.30`; combined D90 spend `~$1.74` well inside `$3.00`). Tear down immediately after archiving.

## Minimum test power

- Held-out `val.jsonl` keeps exact `400` distinct episodes (`320` tool/build, CI half-width `<= 3.4 pp` at `90%`; `80` Chat, CI half-width `<= 5.1 pp` around `95%`), bit-identical across Runs 1–6 (`8ae0ec5c...`).
- Parity probe covers `576` (`>= 512`) token positions across `16` stratified `val.jsonl` sequences + full `400`-episode `HoloV13Model::generate_chat_turn_with_history`.

## Fitness review

TEST FITNESS: FIT — reviewed at 8c5af319d4faba000b5fc752bda5b9209def6088 by fresh-context research subagent ca2233d8-1982-408d-b9a8-4cef81c451a6
