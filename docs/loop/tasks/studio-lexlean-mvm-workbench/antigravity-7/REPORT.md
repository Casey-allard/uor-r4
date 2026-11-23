# REPORT for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v4`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 3 on `vast-62062122` (`~$0.52` spent, cleanly deleted; `Running: 0 pod(s)`):**
  1. Bar #1 WAT float audit passed (`38` funcs scanned, `compute_float_ops = 0`, `js_abi_bridge_casts = 1`, `passed = true`), and BF16 merged LoRA (`lora_eval.json`) passed Bars #2 & #3 at **`100.0%` (`320/320` tool/build, `80/80` Chat, Epoch 1 `val_loss = 1.01718`)**.
  2. Compiled `UORR4V13` (`int4` `diag_v3_int4/`: `93.75%` tool / `96.25%` Chat / `205/205` XOR exact) hit `top1 = 94.62%`, `top5 = 97.92%`, and `mean_kl_nats = 0.155951` (`0.109085` on OOD).
  3. Compiled `UORR4V13` (`--promote-down-int8`, `diag_v3_i8/`: `90.62%` tool / `97.50%` Chat / `226/226` XOR exact) cut recompiler OOD `mean_kl_nats` down to **`0.075072` (`<= 0.08`)** and lifted `576`-pos `val.jsonl` parity to `94.10%` top-1 / `98.96%` top-5 / `0.110754` mean KL (`> 0.08`), leaving `parity_bar_passed = false`.
  4. Root-cause diagnosis isolated the mismatch between OOD KL (`0.075072`) and Foundry `val.jsonl` KL (`0.110754`): `calib_foundry.txt` passed bare user strings wrapped inside `format_qwen3_prompt` (`"You are a helpful assistant."` with empty assistant reply), excluding Foundry's 180-token `system_prompt` and `<tool_call>`/`<prism-write>` assistant continuations from CUDA Cholesky-GPTQ's Hessian `H = (2/N) X^T X`, while `lora_alpha=32` applied `2.0x` adapter scaling and exported BF16 logits instead of FP32.

## What changed

- **[ORDER.md](ORDER.md):** Scoped addendum PR inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-7/**` only.
- **[PREP.md](PREP.md):** Recorded Run 3 diagnostics verbatim and pinned updated Stage A SHA-256 digests (`generate_foundry_sft.py` `28907f23...`: `128` stratified full ChatML `train.jsonl` calibration episodes preserving `train.jsonl` `79780c7e...` and `val.jsonl` `8ae0ec5c...` bit-identically; `patch_recompile.py` `b91006fa...`: direct `<|im_start|>` tokenization capped at `<= 320` tokens; `train_foundry_lora.py` `79e37743...`: `lora_alpha=16` + FP32 parity teacher export) compiled with `--promote-down-int8`.

## Measurements

- Local `--self-test` (`python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test`): unchanged `train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`) and `val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), `calib_foundry_count = 128` (100% `<|im_start|>system\n...` ChatML), `train_val_overlap_hashes = 0`, `320/320` (`100.0%`) + `80/80` (`100.0%`) gold pass.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`72cf6a39-899a-4859-9b22-524008d3aba8` at `232649b8d2b71d1b8ba095ba4c033681106817d2`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- Exact-head verification, `uor-harness check --staged`, `uor-r4-claim-wording`, `uor-harness ratchet d23/d25/d32`, and `uor-harness loop pr-check --pr 4183`: run on the final HEAD commit before enqueueing.

## Skills used

- None outside standard harness / git workflow.

## Still open

- Post `[antigravity] uor-review v1`, verify `uor-harness loop pr-check --pr 4183`, merge PR #4183 via `uor-harness loop queue`, and run `d90-foundry-lora-v4` on Pod #4.

## Plain summary

Recorded Run 3 diagnostics (`100.0%/100.0%` BF16 LoRA; `93.75%/96.25%` `int4`; `90.62%/97.50%` `--promote-down-int8` hitting `0.075072` OOD KL vs `0.110754` val-probe KL), upgraded CUDA GPTQ calibration to `128` full Foundry ChatML training episodes with `lora_alpha=16` and FP32 parity teacher export while keeping `train.jsonl` and `val.jsonl` bit-identical, and opened Run 4 addendum `antigravity-7`.
