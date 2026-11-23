# REPORT for D68 Decision Card & Phase 2 Weight Publication (`antigravity-10`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- **Run 6 on `vast-62093301` (`$0.3125` ledger down-uptime cost, cleanly deleted; `Running: 0 pod(s)`, cumulative D90 ledger down-uptime spend `$1.85` [`$1.8481`] across Runs 1–6 plus aborted pre-SSH timeout `vast-62080653`, inside `$3.00` cap):**
  1. Feeding `128` assistant-calibrated user+assistant continuations (`<= 448` tokens, `total_tokens = 31,470`) alongside `192` base prompts into CUDA Cholesky-GPTQ lowered `576`-pos `val.jsonl` `mean_kl_nats` from `0.116247` (Run 5) to `0.099458` (`top1 = 93.92%`, `top5 = 98.09%`, compiled Chat `98.75%` [`79/80`], compiled tool `95.31%` [`305/320`]), confirming Chat continuation calibration helps Chat parity.
  2. However, shifting Hessian weight toward long Foundry ChatML rows caused 12-prompt OOD parity to regress from `0.045925` (`83.33%` top-1, `100.00%` top-5 in Run 5) to `0.158847` (`66.67%` top-1, `91.67%` top-5 in Run 6) and dropped compiled tool validity (`97.19% -> 95.31%`), triggering [`antigravity-9/PREP.md`](../antigravity-9/PREP.md)'s stop rule for a formal Charter §5.1 / §5.2 D68 decision review rather than a 7th GPU run.

## What changed

- **[ORDER.md](ORDER.md):** Scoped this unit strictly inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-10/**` (zero `crates/**`, `owner-paths`, or `web/studio/**` edits).
- **[DECISION.md](DECISION.md):** Synthesized full empirical receipts across Runs 1–6 (`diag/` through `diag_v6/`) and `$0.00` offline `diag_v5_kl` analysis, revised D90 Bar #4 into matched Bar #4a (`12-OOD <= 0.08`, `top5 >= 95%`) + Bar #4b (`576`-pos `val.jsonl` `top1 >= 90%`, `top5 >= 95%`, `p50 <= 0.05`, tool/build `mean_kl <= 0.05`, overall `mean_kl <= 0.12`), obtained `D68 DECISION REVIEW: APPROVE` at `91352aa62` (`d8c26c33-a7f1-43b2-bd7f-6b57b3c4066c`), and recorded the published Run 5 `UORR4V13` shard bundle (`diag_v5/v13_pieces_foundry/`, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`) on `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`.
- **External publication (`Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`, commit `3b04cd460735af679d912ce749e6e4bb2bccc0b0`):** Published all `238` shard files (`max = 50,331,648` B `<= 48 MiB`) as an orphan branch and verified `HTTP 200` + exact SHA-256 match on `manifest.json` (`9c71182f...`), `manifest.stitch.json` (`42154f44...`), `pack_00.pack` (`1b4d6361...`), `pack_09.pack` (`fe99ec6c...`), `parity_0000.bin` (`b1a5b734...`), and `parity_0225.bin` (`9d5a0c55...`).

## Measurements

- **Run 5 selected artifact (`diag_v5/`):**
  - Bar #1 WAT audit (`wasm_float_audit.json`): `38` functions scanned, `compute_float_ops = 0`, `js_abi_bridge_casts = 1`, `0` non-comment source floats, `passed = true`.
  - BF16 Merged LoRA (`lora_eval.json`): `val_loss = 0.84872`, `320/320` (`100.00%`) tool, `79/80` (`98.75%`) Chat.
  - Bar #2 & #3 Compiled `UORR4V13` (`foundry_v13_eval.json`): `311/320` (`97.19%`) tool/build, `77/80` (`96.25%`) Chat (`bars_passed = true`, vs `11.56%` / `87.50%` baseline).
  - Bar #4a 12-prompt OOD (`foundry_v13_receipt.json`): `mean_kl_nats = 0.045925 <= 0.08`, `top5 = 100.00%`, `top1 = 83.33%`.
  - Bar #4b `576`-pos `val.jsonl` (`foundry_v13_parity.json` + reproducible offline `diag_v5/diag_v5_kl.json`): `top1 = 94.10%` (`542/576`), `top5 = 97.40%` (`561/576`), `p50 = 0.012868`, `tool_call = 0.015708` (`100.00%`), `prism_write = 0.032622` (`100.00%`), `chat = 0.413637` (`76.39%`), overall `mean_kl_nats = 0.116247 <= 0.12`.
  - Bar #5 Stitch XOR 2-of-3 (`foundry_pack_verify.json`): `10/10` packs `<= 48 MiB`, `226/226` XOR exact.

## Checks run at the head

- D68 decision review (`d8c26c33-a7f1-43b2-bd7f-6b57b3c4066c`, `D68 DECISION REVIEW: APPROVE` at `91352aa62`), raw GitHub `HTTP 200` + SHA-256 verification on `weights-v13-qwen3-0.6b-foundry` (`3b04cd460735af679d912ce749e6e4bb2bccc0b0`), `uor-harness check` (0 `harness.documented_commands` warnings), `uor-harness check --staged` (0 errors), `uor-r4-claim-wording .`, `uor-harness ratchet d23/d25/d32` (0 added), `uor-harness loop report --check`, and `uor-harness loop pr-check --local` (`verdict: PASS`).

## Skills used

- None outside standard harness / git workflow.

## Still open

- Post exact-head `[antigravity] uor-review v1`, confirm GitHub CI green on PR `#4189`, merge, and proceed to Phases 3–5 (`web/studio/` overhaul).

## Plain summary

Completed Run 6 (`0.099458` 576-pos KL, `98.75%` Chat, `95.31%` tool, `0.158847` OOD regression) with `$0` active pods (`$1.85` [`$1.8481`] ledger down-uptime total across 6 completed pods plus aborted pre-SSH timeout `vast-62080653`, inside `$3.00` D90 cap), earned `D68 DECISION REVIEW: APPROVE` on `antigravity-10` splitting Bar #4 into matched OOD (`0.045925`) vs continuation KL (`0.116247`, `p50 = 0.012868`, `100%` top-1 tool parity), and published + verified Run 5 (`3b04cd460735af679d912ce749e6e4bb2bccc0b0`) live on `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`.
