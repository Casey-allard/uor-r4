# REPORT: d11-exporter-float-trunk, kimi-46

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md). Prep sheet: [d11-exporter-float-trunk-longer-dual-2026-10-26.md](../../../../harness/prep/d11-exporter-float-trunk-longer-dual-2026-10-26.md).

References #2820. The item is not done and no `Closes` claim is made.

## What did not work, first

- **Attempt 1 was an instrument failure before training.** The launcher's checksum command used two `printf` placeholders for arguments that each already contained a hash and path. The resulting manifest was malformed and the job exited at `pins`. The 40-file archive was downloaded and every `shasum -a 256 -c` check passed. The repair uses one placeholder per line and attempt-unique pod/store paths; it does not overwrite the failure archive.
- **Attempt 2 stopped at the registered I-Prefix instrument gate, before export.** Training, all 2,048 batches, evaluation placement and save/load passed on both seeds. The exact comparison with claude-131 did not. Step 0 matches exactly on both seeds, including losses and top-1 counts; later points differ. Batch construction is the same ordered `.take(num_batches * batch_size)` path, and the post-claude-131 residual/Q16/snap controls are inactive in ordinary dual training. This bounds the stop to training reproducibility across the two pods, not initialization or DEV evaluation. It does not prove the cause. The exact gate was not weakened after the numbers were visible.
- **No export-gap verdict exists.** No D11 artifact, A panel, D counterfactual, gap/residual receipt, artifact tokens/s or zero-multiply serving result was produced by this run. The final trained-model numbers are not an unflagged PASS, flagged PASS, cancellation PASS, exporter-unresolved result or configuration FAIL. #2820 remains open.
- The first `down` refused its own archive verification. The attempt-2 HF listing was confirmed after a successful local hash readback, and `down --confirm-archived` then deleted the pod. No pod remains running.

## What changed

`loop pr-check --local` reports code 2 files +765/−89 and docs 6 files +342/−0; tests are included in the code files, and no generated files changed. The ORDER gave no numeric size estimate, so a ratio is not available.

- `--steps` accepts only 1,024 or 2,048 and is validated before CUDA opens. The default remains 1,024; modes that do not consume the option reject it rather than silently ignoring it.
- Longer training uses the requested batch count and adds the final 2,048 evaluation point without changing the earlier points, learning rates, 1:1 dual loss, seeds or data.
- Gap and residual scorers bind dose, model identity and saved F/D/A prediction outputs; missing seed losses refuse the receipt. Legacy Stage 4b receipts read their nested 1,024 run steps, while modern 1,024 and 2,048 receipts validate their full dose identity.
- The prep sheet registers the dose-plus-coverage intervention, exact outcomes, counterfactual, instruments, cost and attempt-specific archival/readback.
- FIT passed at `5345b554`; D68 passed at `e77a1f91`; D67 passed conditionally on storage at `78a5369a6` and again for attempt 2 at `c719506f`. CI was green at the prior head `c719506f`; fresh checks are required at the delivery head.

## Measurements

**Speed, not a serving result.** On one RTX 5090, 2,048-step dual training ran at 4,470.49 and 4,470.82 training tokens/s, 351.83 and 351.81 seconds per seed (`seeds[].run.train_tokens_per_second` and `train_wall_seconds`). In-run float decode was 158.24 and 158.86 tokens/s. These are not artifact serving rates; no `fbs-toks` measurement was allowed or run.

**I-Prefix at step 1,024, over 14,287 DEV rows.** The command was the sheet's `run_step prefix jq -e -n --slurpfile new "$RUN/train.json" --slurpfile old "$RUN/train-1024.json" ...`; its exit was 1.

| Seed | Step 0 F loss old/new | Step 1,024 F top-1 old → new | Step 1,024 export-mode top-1 old → new | Step 1,024 F loss old → new |
|---|---|---|---|---|
| 1 | 2.793868555551714 / identical | 3,845 → 1,472 | 3,511 → 1,048 | 1.9065087296576082 → 1.82907697925326 |
| 2 | 2.7612276525838535 / identical | 3,128 → 3,258 | 3,023 → 3,209 | 1.877547065864761 → 1.8726267523100388 |

Step-0 export-mode top-1 was also identical (0 and 11). At step 256 the old/new F top-1 values were 2,280/1,664 and 1,464/1,715; at step 512 they were 3,322/2,808 and 2,308/2,997. Thus the runs agree at initialization and diverge during training. No interpretation of the 2,048-step export gap is licensed.

**Probe and training instruments.** `train_learns=true`, `learns=true`, `under_one_hour_per_seed=true`; 2,048 batches, 8,192 used windows, 68,218 usable windows and 291,414 scored targets. `instruments_pass=true`; evaluation points are exactly `[0,256,512,1024,2048]`; both `i_save_load=true`.

**Evidence and cost.** Attempt 1 is sealed at `results/kimi/d11-fbs-longer-dual-2026-10-26/`; attempt 2 at `results/kimi/d11-fbs-longer-dual-2026-10-26/attempt-2/`. Attempt 2 has 85 total files, of which 83 are hash-listed and verified; `SHA256SUMS` and `job.log` are deliberately not self-covered. Attempt 1 likewise has 40 total files and 38 verified entries. See [READBACK.md](READBACK.md) for the local hash, HF-listing and storage receipts. Attempt 2's `train.json` SHA-256 is `b4ecb87da64916683033cd1098a7c19f1af679549fc5c74dfbd55f438836f342`; the prior receipt remains `5a290fbe47d8f10495b6d89fa8384110061902001ad9ebdf6b365935e220d9bc`. The compute ledger's 2,336-second pod lifetime at an offered $0.975/h is a $0.633 proxy, not a final invoice. Agent review, coordination and CI time are additional; their total is NOT_REPORTED.

## Checks run at the head

- Archived `bridges.out` at run source `e77a1f91`: **82 passed, 5 ignored** for `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` on the RTX 5090. The separate earlier host suite had 60 library tests; it is not the archived CUDA count.
- Host suite after the compatibility repair: **61 library, 4 CLI, 2 dependency and 12 kernel-control tests passed** with `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-float-train --no-fail-fast`. The failing-first compatibility controls failed before the repair and pass after it.
- Attempt 2 `bridges.exit`, `probe-gate.exit`, `train-gate.exit`: all 0. `prefix.exit`: 1, the registered stop.
- Local harness library suite after the FIT-format repair: 677 passed, 1 ignored. The slower local integration command hit its 900-second tool timeout after the library pass; it is not claimed as a complete local suite.
- GitHub at the pre-repair REPORT head `2e3fbaaec049fd865a4fffa34754a78cbad652e3`: no failed or pending checks. All 11 local loop gates and `uor-harness loop pr-check --local` passed there; both are rerun for the repair before delivery.
- `bash -n local/kimi/d11-float-trunk/job.sh`: PASS. Both archived `SHA256SUMS` readbacks: PASS. Final `scripts/pod/uor-pod status`: 0 pods, $0/h.

## Skills used

- **uor-research-architect:** kept the run artifact-bound, matched to its exact outcome and clear about the remaining uncertainty.
- **systematic-debugging:** reproduced the CI and launcher failures, isolated the prefix stop, and avoided treating an instrument fault as a mechanism negative.
- **holding-the-goal / solution-integrity / working-discipline:** preserved the registered gate and failure evidence rather than scoring around a red instrument.

## Still open

- #2820 remains open. This unit does not meet its export-gap, serving or close conditions.
- The next unit must distinguish same-device repeatability from cross-device/source-version drift with a separately FIT'd diagnostic. A new result is required before changing or replacing I-Prefix; it must not be relaxed from this single mismatch.
- If a future valid run reaches the registered outcomes, flagged PASS and cancellation PASS still need D68; exporter-unresolved needs a separate FIT'd localisation; unflagged PASS still needs artifact tokens/s and serving audits.
- The mechanism-page update named in the item remains outside this ORDER.

## Plain summary

The code now permits the planned longer training run and records which dose produced each receipt. The first launch failed because its checksum list was formatted incorrectly. The second launch trained both models successfully, but a safety check found that their earlier training behavior did not exactly reproduce the previous run on another pod. Because that check failed, no export or quality verdict was allowed. Both attempts are preserved and verified, the pod is gone, and the exporter issue stays open pending a reproducibility diagnostic.
