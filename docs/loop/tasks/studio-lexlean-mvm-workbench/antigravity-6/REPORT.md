# REPORT for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v3`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 2 on `vast-62057972` (`~$0.10` spent, cleanly deleted; `0` pods running):**
  1. Bar #1 WAT float audit passed (`holo_inference_functions_scanned = 38`, `compute_float_ops = 0`, `js_abi_bridge_casts = 1`, `holo_runtime_src_noncomment_float_lines = 0`, `passed = true`) and Bar #2 tool/build passed (`310/320` = **`96.88%`** vs `11.56%` baseline, `val_loss = 1.00785`), whereas Bar #3 Chat scored `70/80` (`87.50%`, below `>= 95.0%` target).
  2. Inspecting every failure case in `diag_v2/lora_predictions.jsonl` isolated `6/10` Chat misses on `CHAT_TOPICS_VAL[0]` (`Walk me through staying in Home Chat mode before pushing a project to the Studio builder.`) where the model emitted `<prism-write` tags or `"No project is saved yet..."` due to literal `` `<tool_call>` `` / `` `<tool_response>` `` tags inside `CHAT_TOPICS_TRAIN[16]` plus an `18.75%` (`1,200/6,400`) Chat share across `1` epoch; the remaining `4/10` misses reflected under-exposure at `1` epoch (`50` rows per topic).
  3. All `10/320` tool misses fell on two unseen `VAL_TEMPLATES` phrasing families: `github_pull` (`4` rows adding an unexpected `message` arg) and `github_push` (`6` rows emitting `workspace_etch`/`workspace_upload`).

## What changed

- **[ORDER.md](ORDER.md):** Scoped addendum PR inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-6/**` only.
- **[PREP.md](PREP.md):** Recorded Run 2 diagnostics verbatim, pinned updated SHA-256 digests for `generate_foundry_sft.py` (`4e0127d0...`, `2,400` Chat [`37.5%`] / `2,400` single-turn tool / `800` `<prism-write>` / `800` multi-turn rebalanced post-`val` via `balance_rng`, literal XML tool tags stripped from `CHAT_TOPICS_TRAIN[16]`, `5` disjoint Home Chat / VM / `.pack` / `9p` topics appended, `github_pull`/`github_push` templates doubled `2 -> 4`, while keeping `val.jsonl` `8ae0ec5c...` bit-identical) and `train_foundry_lora.py` (`544b4cb7...`, `epochs=2`, `lr=1.0e-4`, `dropout=0.08`, `wd=0.03`, best-`val_loss` adapter snapshot restore).

## Measurements

- Local self-test of updated Stage A scripts (`python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test`): generated `6,400` balanced `train.jsonl` rows (`2400` `chat`, `2400` `tool_call`, `800` `prism_write`, `800` `tool_followup`; `0` literal XML tool tags in Chat; `sha256: 79780c7eaf17f4f354fc331d0a9ea3c315f57a4337fcc737eaec0b6ecd7f076b`) and identical `400`-row held-out `val.jsonl` (`sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`), `train_val_overlap_hashes = 0`, `320/320` (`100.0%`) gold tool/build, and `80/80` (`100.0%`) gold Chat.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`d2a98584-6245-4156-aa60-ed3db6f6c710` at `4ed3c93a1fe1ade4f3cafa101021068442cf2678`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- Exact-head verification, `uor-harness check --staged`, `xtask links`, `uor-r4-claim-wording`, `uor-harness ratchet d23/d25/d32`, and `uor-harness loop pr-check --pr 4180`: run on the final HEAD commit before enqueueing.

## Skills used

- None outside standard harness / git workflow.

## Still open

- Post `[antigravity] uor-review v1`, verify `uor-harness loop pr-check --pr 4180`, merge PR #4180 via `uor-harness loop queue`, and run `d90-foundry-lora-v3` on Pod #3.

## Plain summary

Diagnosed all 10 Chat misses and 10 tool misses in Run 2 (`diag_v2/`), refined Stage A scripts to rebalance training to `37.5%` tag-free Home Chat and disambiguate `github_pull`/`github_push` while keeping held-out `val.jsonl` (`8ae0ec5c...`) bit-identical, and opened pre-registration addendum `antigravity-6` before launching Pod #3.
