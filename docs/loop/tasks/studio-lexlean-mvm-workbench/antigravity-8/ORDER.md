# ORDER for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v5`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `74e638a6f`. Updates the pre-registration sheet (`PREP.md`, Run `d90-foundry-lora-v5`) following Run 4 diagnostics on `vast-62072177` (`~$0.27` spent, cleanly deleted with `Running: 0 pod(s)`; combined D90 spend `~$1.13` of `$3.00` cap).

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero binaries (§3.13), and non-author `FIT_VERDICT: FIT` + `VERDICT: APPROVE` on `antigravity-8/PREP.md` before renting Pod #5.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-8/**` only (zero `owner-paths` or `crates/**` modifications):
- `PREP.md`: records Run 4 empirical negatives verbatim (BF16 `lora_alpha=16` `99.06%` tool / `95.00%` Chat `val_loss=1.02184`; compiled `val.jsonl` `86.25%` tool / `88.75%` Chat; `576`-pos `94.62%` top-1 / `98.78%` top-5 / `0.143033` mean KL; `0.071966` OOD KL; `226/226` XOR exact), restores Run 3's clean `96`-prompt `calib_foundry.txt` (`4e0127d0...` / `c76451b1...`) + `lora_alpha=32` while adding training `label_smoothing=0.05` + FP32 parity teacher (`train_foundry_lora.py` `c043a1a9...`) and `--promote-down-int8`, preserving bit-identical `train.jsonl` (`79780c7e...`) and `val.jsonl` (`8ae0ec5c...`), and capping Run 5 at `<= $1.20` / `2.0h` (combined `< $2.33 <= $3.00` D90 cap).
- `REPORT.md`: summarizes Run 4 empirical findings and Run 5 pre-registration checks.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local`, `uor-harness check --staged`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if non-author reviewer finds `PREP.md` `NOT_FIT` or any file outside `antigravity-8/**` is modified.
