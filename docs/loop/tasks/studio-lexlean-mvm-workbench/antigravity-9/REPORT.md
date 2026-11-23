# REPORT for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v6`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 5 on `vast-62082624` (`~$0.31` spent, cleanly deleted; `Running: 0 pod(s)`):**
  1. Bar #1 (`compute_float_ops = 0`), BF16 LoRA (`100.00%` [`320/320`] tool / `98.75%` [`79/80`] Chat, `val_loss = 0.84872`), like-for-like 12-OOD parity (`mean_kl_nats = 0.045925 <= 0.08`), compiled `val.jsonl` Bars #2–#3 (`97.19%` [`311/320`] tool / `96.25%` [`77/80`] Chat), and Bar #5 Stitch XOR (`10/10` packs `<= 48 MiB`, `226/226` XOR exact) all passed.
  2. Bar #4 `576`-pos `val.jsonl` parity missed sole KL threshold (`top1 = 94.10%` [`542/576`], `top5 = 97.40%` [`561/576`], `mean_kl_nats = 0.116247 > 0.08`, `parity_bar_passed = false`).
  3. Offline `$0.00` `diag_v5_kl` analysis proved `tool_call` (`n=396`: `100.00%` top-1, `mean_kl = 0.015708`, `rms = 0.2462`, `cos = 0.97850`) and `prism_write` (`n=36`, `len=500`: `100.00%` top-1, `mean_kl = 0.032622`, `rms = 0.3119`, `cos = 0.97053`) are already near-lossless, while `chat` (`val_ch_000..003`, `n=144`: `76.39%` top-1, `mean_kl = 0.413637`, `rms = 0.5030`, `cos = 0.93313`) drives `89.0%` of total KL and `34/34` top-1 misses because CUDA Cholesky-GPTQ never calibrated on Foundry assistant Chat prose continuations (`0` assistant tokens in `calib_foundry.txt`).

## What changed

- **[ORDER.md](ORDER.md):** Scoped addendum PR inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-9/**` only.
- **[PREP.md](PREP.md):** Recorded Run 5 `diag_v5/` metrics and offline `diag_v5_kl` breakdown verbatim, preserved Run 5's `train_foundry_lora.py` (`c043a1a9...`) and bit-identical `train.jsonl` (`79780c7e...`) / `val.jsonl` (`8ae0ec5c...`), and updated `generate_foundry_sft.py` (`397c266f...`) + `patch_recompile.py` (`bb5da659...`) to feed `128` assistant-calibrated user+assistant continuations (`112` compact + `16` full-system, `<= 448` tokens) into CUDA Cholesky-GPTQ.

## Measurements

- Local Stage A checks: `py_compile` passed; `python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test` passed (`train_sha256 == 79780c7e...`, `val_sha256 == 8ae0ec5c...`, `train_val_overlap_hashes == 0`, `calib_foundry_count == 128`, `320/320` + `80/80` = `100.0%`); temporary patched `recompile_holo_v13.rs` `cargo check --locked -p lexlean-r4-wasm --bin recompile_holo_v13` passed cleanly before cleanup.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`ca2233d8-1982-408d-b9a8-4cef81c451a6` at `8c5af319d4faba000b5fc752bda5b9209def6088`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- Local harness gates (`uor-harness check --staged`, `uor-r4-claim-wording .`, `uor-harness ratchet d23/d25/d32`, `uor-harness loop pr-check --base origin/main --head HEAD --local`): run on the final HEAD commit before posting `[antigravity] uor-review v1` and enqueueing.

## Skills used

- None outside standard harness / git workflow.

## Still open

- Complete FIT review, exact-head review, PR merge on `antigravity-9`, and execute `d90-foundry-lora-v6` on Pod #6.

## Plain summary

Diagnosed Run 5's `576`-pos parity miss (`0.116247` nats) offline to `4` uncalibrated Foundry Chat episodes (`89.0%` of KL; `tool_call` `0.0157` / `prism_write` `0.0326`), updated only GPTQ calibration (`128` user+assistant continuations `<= 448` tokens, preserving bit-identical train/val splits and `label_smoothing=0.05`), and opened Run 6 addendum `antigravity-9`.
