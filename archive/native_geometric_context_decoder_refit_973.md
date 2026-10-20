# Conditional decoder refit — #973, September 13, 2026

**Actual gate: `FAIL_CONTEXT_DECODER_REFIT_DEVELOPMENT_GATE`. Retain `15baec48`; no promotion.** All four prespecified method contrasts and all four numeric model gates fail. This is a completed negative development result, not general language or coding qualification. The owner's subsequent [attention reassessment](native_geometric_attention_reassessment_973.md) supersedes further automatic state/head experiments.

## Design and actual behavior

The four cells use frozen parent/repeated-learned/varied-learned state snapshots, the exact saved 48-document datasets (2,688 byte/EOS positions each), inherited caps and geometry, and one depth-three/minimum-leaf64 conditional head fit per cell. There are no new state proposals. The first parent/repeated fit exactly reproduces the 8f8 branch payload (`94ed55a8a716ac1f922476992fe83a3a51b14eb9faa97d4c7dde30890336d60f`) before the other fits execute; training metadata appropriately differs.

| State / decoder data | Original Full correct /672 | Original NLL | Opened Full correct /381 | Opened NLL | Recombination Full correct /672 | Recombination NLL |
|---|---:|---:|---:|---:|---:|---:|
| Parent / repeated | 166 | 2.839534 | 52 | 3.653604 | 100 | 3.731054 |
| Learned repeated / repeated | 152 | 2.812370 | 48 | 3.631449 | 99 | 3.694897 |
| Parent / varied | 85 | 3.171070 | 39 | 3.362274 | 81 | 3.418240 |
| Learned varied / varied | 86 | 3.192744 | 36 | 3.429999 | 79 | 3.461138 |

Decoder-only adaptation decreases recombination NLL by 0.312814 but loses 19 correct positions. The additional learned-varied state increases NLL by 0.042898 and loses two correct positions against the parent state with matched varied-data refit. The fixed decoder was not the sole explanation of the failed context-augmentation result. Neither this contrast nor lower own-training loss establishes exhaustion of the geometry or the necessity of a new objective.

All 32 Full continuations (four cells × four original and four recombination prompts) remain incoherent, reach 96 bytes and do not emit EOS. Useful Rust/code execution is not established. Reports retain every actual output. Each cell receives all prior individual/control comparisons; loss counts are not aggregated into a preservation pass. Root/source trajectories remain exactly those of each corresponding frozen state. Hamming fields describe aligned categorical drift, not semantic distance.

The four original source-bound witnesses are `43899490`, `d2f05bf4`, `3abe61c8`, `1c962c53` in table order. These identify test-only conditional heads, not normal promoted model artifacts. The tracked evidence binds their full identifiers, payloads, datasets, source, binary and exact report inventory. Construction fit `before` scores use the inherited cap-only baseline; they are not comparisons against the frozen old angular head.

## Preserved reporting failure and evaluation-only recovery

The first exclusively claimed attempt completed all four fits, saved all heads/witnesses/fit receipts and generation, and replayed the prior behavior plus three cell reports. Writing the fourth pretty JSON report exceeded the 64 MiB report cap. It is sealed **INCOMPLETE** with `report write exceeds storage limit with seal reserve`: 26 files / 56,562,059 bytes. It contains no final model gate. That failure is not relabeled PASS.

A separately source-bound Rust recovery driver loaded and validated the saved witnesses, used compact JSON, and reran evaluation only: **zero fits, four preserved fits**. It checked exact parsed-JSON equality against all previously completed reports, completed the missing comparisons and sealed 18 files / 38,068,437 bytes. Its result contains the actual failed model gate above. The original driver, binary, registration snapshot and sealed attempt remain unchanged. No fit was repeated to repair serialization.

Original executable SHA256: `cad7bcefd01c1b7e06492b1b75a59252c6a81e8d657cf042ecc522b49c7cb85c`. Recovery executable SHA256: `6e89d6d76d5b5e590f12abf25da7b2582ec15c0d553faaef198e7757a957dee0`. Original and recovery source freezes bind 140 and 141 files respectively; the original test registration is preserved separately.

## Executed checks, resources and lineage

Production runtime/training/geometry source is unchanged. Two focused release test invocations pass: the original witness/reproduction test (with two small synthetic fits) and the recovery serialization test (no fits). The actual original experiment exits 101 for report capacity; the recovery experiment passes execution checks and reports the failed model gate. All 28,161 prior behavior rows and 120 prior outputs replay exactly. The four 512-step operation censuses observe at most 23/22/20/20 prediction angular comparisons, within 27; allocation is `NOT_RUN`.

Nineteen prior sealed roots / 689 files, retained artifact hash, the owner's original checkout and 22 parked dirty paths verify unchanged. Memory repair and V3–V7 remain parked. The current worktree is `/Users/casey.allard/uor-r4-worktrees/shared-geometric-core`, branch `codex/geometric-context-decoder-refit`.

Build/test/model charge is **222,406 / 420,000 ms**. Parent cycle **2,897,050 / 3,200,000 ms**; shared ledger **122,647,623 / 132,950,000 ms**. The necessary 200,000 ms local increment was recorded before use; shared ceiling unchanged. Report recovery raised this step's projected new storage from 128 to 192 MiB before use, with no change to the parent's 2 GiB ceiling or 128 MiB stop margin. Execution receipt records 128,118,784 new bytes, parent growth 1,787,117,568 bytes, and peak sampled process-tree RSS 2,532,589,568 bytes. Subsequent research/documentation/delivery growth is recorded in the final local receipt. No paid compute or cleanup occurred.

[Tracked evidence](evidence/native_geometric_context_decoder_refit_973.json) points to the original paths under `/Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/context-decoder-refit-1/`: `attempt-1/failure.json`, `attempt-2/result.json`, `summary.json`, `selected-deltas.json`, both source freezes, reviews, resource projection and execution receipt. The final resume note and research remain inside this established project-local handoff. Do not rerun completed fits or mutate either sealed attempt.
