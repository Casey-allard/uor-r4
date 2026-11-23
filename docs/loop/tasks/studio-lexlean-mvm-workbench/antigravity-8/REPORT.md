# REPORT for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v5`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 4 on `vast-62072177` (`~$0.27` spent, cleanly deleted; `Running: 0 pod(s)`):**
  1. Scaling down to `lora_alpha=16` reduced BF16 LoRA accuracy from `100.0%`/`100.0%` (`val_loss = 1.01718` in Run 3) to `99.06%` (`317/320`) tool / `95.00%` (`76/80`) Chat (`val_loss = 1.02184`).
  2. Feeding `128` long 320-token ChatML rows sharing the identical 180-token Foundry system prompt degraded compiled `--promote-down-int8` performance across all val metrics (`86.25%` tool [`276/320`], `88.75%` Chat [`71/80`], `576`-pos `val.jsonl` `mean_kl_nats = 0.143033`), even though Bar #1 (`compute_float_ops = 0`), recompiler OOD (`mean_kl_nats = 0.071966`), and Bar #5 Stitch XOR (`226/226`) passed.
  3. Cross-checking Runs 3–4 alongside `HoloV13Model::step_token` (`DenseExact` MLP, zero repetition penalty) confirmed Run 3's `96`-prompt `calib_foundry.txt` + `lora_alpha=32` already passed compiled Bars #2 & #3 cleanly (`90.62%`/`97.50%`), while excessive logit sharpness from unsmoothed (`0.0`) SFT left quantized val-probe KL at `0.110754` (`0.03` nats above `0.08`).

## What changed

- **[ORDER.md](ORDER.md):** Scoped addendum PR inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-8/**` only.
- **[PREP.md](PREP.md):** Recorded Run 4 diagnostics and offline synthesis verbatim, restored Run 3's clean `96`-prompt `generate_foundry_sft.py` (`4e0127d0...`) + `patch_recompile.py` (`c76451b1...`), and pinned `train_foundry_lora.py` (`c043a1a9...`, `lora_alpha=32`, training `label_smoothing=0.05`, unsmoothed `val_loss` selection, FP32 parity teacher) for `--promote-down-int8` compilation.

## Measurements

- Local `--self-test` (`python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test`): unchanged `train.jsonl` (`6,400` rows, `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`) and `val.jsonl` (`400` rows, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), `calib_foundry_count = 96`, `train_val_overlap_hashes = 0`, `320/320` (`100.0%`) + `80/80` (`100.0%`) gold pass.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`ec743abe-b435-403e-9af7-765434d990d9` at `c5ec03c900fda4f1166aa72b2a32dbf4f5798d1e`, prior draft `473929c4-2533-4a57-a3cb-b7cd88875017` at `ae66f59be7925f46dde6b997e1cd23643633a715`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- Pre-review local harness checks (`uor-harness check --staged`, `uor-r4-claim-wording .`, `uor-harness ratchet d23/d25/d32`, and `uor-harness loop pr-check --base origin/main --head HEAD --local`): run on the final HEAD commit before posting `[antigravity] uor-review v1` and running `uor-harness loop pr-check --pr 4186`.

## Skills used

- None outside standard harness / git workflow.

## Still open

- Post `[antigravity] uor-review v1`, verify `uor-harness loop pr-check --pr 4186`, merge PR #4186 via `uor-harness loop queue`, and run `d90-foundry-lora-v5` on Pod #5.

## Plain summary

Recorded Run 4 negatives (`diag_v4/`), restored Run 3's clean `96`-prompt calibration + `lora_alpha=32` baseline that already passed compiled Bars #2–#3, added `label_smoothing=0.05` on training cross-entropy to tame quantized KL divergence (`<= 0.08`), and opened Run 5 addendum `antigravity-8`.
