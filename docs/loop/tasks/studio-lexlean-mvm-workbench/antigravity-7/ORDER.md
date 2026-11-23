# ORDER for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v4`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `14da23c1e`. Updates the pre-registration sheet (`PREP.md`, Run `d90-foundry-lora-v4`) following Run 3 diagnostics on `vast-62062122` (`~$0.52` spent, cleanly deleted with `Running: 0 pod(s)`; combined D90 spend `~$0.86` of `$3.00` cap).

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero binaries (§3.13), and non-author `FIT_VERDICT: FIT` + `VERDICT: APPROVE` on `antigravity-7/PREP.md` before renting Pod #4.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-7/**` only (zero `owner-paths` or `crates/**` modifications):
- `PREP.md`: records Run 3 empirical diagnostics verbatim (BF16 `100.0%` tool / `100.0%` Chat at Epoch 1; compiled `int4` `93.75%` tool / `96.25%` Chat / `205/205` XOR / `94.62%` top-1 / `0.155951` mean KL; compiled `--promote-down-int8` `90.62%` tool / `97.50%` Chat / `226/226` XOR / `94.10%` top-1 / `98.96%` top-5 / `0.075072` OOD mean KL / `0.110754` val-probe mean KL), pins updated Stage A SHA-256 digests (`generate_foundry_sft.py`, `patch_recompile.py`, `train_foundry_lora.py`) preserving bit-identical `train.jsonl` (`79780c7e...`) and `val.jsonl` (`8ae0ec5c...`) while feeding `128` full ChatML calibration episodes (`max_ct <= 320`), `lora_alpha=16` (`r=16`), FP32 parity teacher export, and `--promote-down-int8`, capping Run 4 at `<= $1.50` / `2.0h` (combined `< $2.36 <= $3.00` D90 cap).
- `REPORT.md`: summarizes Run 3 empirical findings and Run 4 pre-registration checks.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local`, `uor-harness check --staged`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if non-author reviewer finds `PREP.md` `NOT_FIT` or any file outside `antigravity-7/**` is modified.
