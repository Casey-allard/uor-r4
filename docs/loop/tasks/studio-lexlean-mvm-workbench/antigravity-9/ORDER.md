# ORDER for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v6`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `991005989`. Updates the pre-registration sheet (`PREP.md`, Run `d90-foundry-lora-v6`) following Run 5 on `vast-62082624` (`~$0.31` spent, cleanly deleted with `Running: 0 pod(s)`; combined D90 spend `~$1.44` of `$3.00` cap) and `$0.00` offline `diag_v5_kl` root-cause diagnosis.

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero binaries (§3.13), and non-author `FIT_VERDICT: FIT` + `VERDICT: APPROVE` on `antigravity-9/PREP.md` before renting Pod #6.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-9/**` only (zero `owner-paths` or `crates/**` modifications):
- `PREP.md`: records Run 5 empirical metrics & `diag_v5_kl` offline root-cause breakdown verbatim, preserves `train_foundry_lora.py` (`c043a1a9...`, `lora_alpha=32`, `label_smoothing=0.05`) + bit-identical `train.jsonl` (`79780c7e...`) & `val.jsonl` (`8ae0ec5c...`) + `--promote-down-int8`, updates GPTQ `calib_foundry.txt` (`generate_foundry_sft.py` `397c266f...`) + `patch_recompile.py` (`bb5da659...`) to `128` diverse user+assistant Chat/tool/prism-write continuations (`<= 448` tokens; only `16` full-system rows), keeps Bars #1–#5 unchanged, and caps Run 6 at `<= $0.80` / `1.5h` (final calibration experiment inside `$3.00` D90 cap before any bar decision review).
- `REPORT.md`: summarizes Run 5 + `diag_v5_kl` empirical findings and Run 6 pre-registration checks.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local`, `uor-harness check --staged`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if non-author reviewer finds `PREP.md` `NOT_FIT` or any file outside `antigravity-9/**` is modified.
