# REPORT: m4-lawful-likelihood-bpb, antigravity-113

Lab: antigravity. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md). Order: [`ORDER.md`](ORDER.md). Issue: `#2861` (Milestone M4 `#2032`). Suborders: `local/antigravity/antigravity-113/orders/1.md` (fresh-context FIT review subagent `7a8d747b-c4e7-4036-b756-bfcc0c0a034a`).

Closes #2861

## What did not work

- **`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` remained at `TEST FITNESS: NOT_FIT` after Prerequisite 2 and Prerequisite 1 were satisfied (`classification: instrument`):**
  - In `deepseek-59` (`#3099`), Prerequisite 3 (per-window `se_n = 0.014525 * sqrt(512/n)` scaling and the 2048-window `INCONCLUSIVE` configuration outcome) was folded into the sheet, while Prerequisite 2 (Casey's two Section 5 registration answers) and Prerequisite 1 (the exported `UORLUT01` base artifact SHA-256 and training seeds) remained open.
  - Prerequisite 2 was subsequently answered by the owner (`#2028#issuecomment-6379430093` / `#2861#issuecomment-6379432987`, `"Yes to both (Recommended)"`), and Prerequisite 1 became available when `m1-close-saved-model` (`#2837` / `#3312`) and `m1-fresh-panel-transfer` (`#2841` / `#3318`) closed M1 with the two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-discrete-trunk/1`) artifacts (`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` and `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`), but the sheet had not yet been updated or re-reviewed for `TEST FITNESS: FIT`.
- **No held-out model BPB score is claimed in this unit:** `m4-lawful-likelihood-bpb` (`#2861`, `needs_gpu: false`) builds and registers the lawful likelihood (`PRLL`) and its pre-registration sheet; the GPU held-out stream scoring run on `heldout.u16` belongs to `m4-export-uorlut01-cli` (`#2859`, `needs_gpu: true`).

## What changed

- **Size:** `0` code lines, `0` test lines, `126` documentation lines, `6` generated lines (`docs/roadmap/README.md`) across `8` files (`1.0x` estimate).
1. **`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`:**
   - Recorded Casey's verbatim owner answer `"Yes to both (Recommended)"` (`#2028#issuecomment-6379430093` / `#2861#issuecomment-6379432987`) in Section 5, confirming (i) reading the `0.90` BPB bar through the uncalibrated lawful likelihood (`PRLL`) and (ii) that the `chat-v0-p2` `heldout.u16` stream (`e5f400b0e676791fd0458ce810163c5d2542adda80dccc8282f37d9ca8904cef`) is the chat held-out stream the bar means (clearing Prerequisite 2).
   - Pinned the two-seed M1-accepted 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-discrete-trunk/1`) base artifacts (`t16-frozen-zero-seed1-trunk.d11` SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, seed `0x5eed_0001`, and `t16-frozen-zero-seed2-trunk.d11` SHA-256 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`, seed `0x5eed_0002`, archived at `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/m1-close-saved-model-20261026.tar` SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`) and the deterministic `s[0..4096]` score mapping in Section 6 (clearing Prerequisite 1).
   - Preserved Claude's historical 2026-10-23 `NOT_FIT` block verbatim in Section 10 and appended the fresh-context non-author `TEST FITNESS: FIT` review (subagent `7a8d747b-c4e7-4036-b756-bfcc0c0a034a`, posted at `#2861#issuecomment-6433712065`).
2. **`docs/labs/m4-lawful-likelihood-bpb/README.md`, `docs/research/open-questions.md`, `docs/model/serving-and-export.md`, `docs/research/notebook/m4-serving.md`:**
   - Updated status and cross-references to reflect that `PRLL` (`crates/uor-r4-integer/src/lawful_likelihood.rs`, `#2945`) and its pre-registration sheet (`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`) are complete and `TEST FITNESS: FIT`, and added the notebook row in `docs/research/notebook/m4-serving.md`.
3. **`docs/roadmap/items/m4-lawful-likelihood-bpb.md` and `docs/roadmap/README.md`:**
   - Set `status: done` on `m4-lawful-likelihood-bpb` (`#2861`) and re-rendered `docs/roadmap/README.md` (`73 done`, `11 ready`, `10 held`, `1 decision`, `95 total`), unblocking `m4-export-uorlut01-cli` (`#2859`) on the M4 path.

## Measurements

- **Finite host controls (`crates/uor-r4-integer/src/lawful_likelihood.rs`):** `9 / 9` unit tests passed (`0.00s`, `$0` GPU spend):
  1. `tiny_alphabet_is_hand_computable`: `V = 4` exact sum `2^48` and monotonic masses.
  2. `worst_case_minimum_keeps_one_unit_and_forty_eight_bits`: `V = 4096` minimum score gets exact mass `1` and `48 * 4096` `bits_q12` (`48` bits).
  3. `uniform_power_of_two_vocabulary_is_exactly_uniform`: `V = 4096` equal scores give `2^48 / 4096` each (`12` bits).
  4. `non_power_of_two_remainders_differ_by_at_most_one`: `V = 3` equal scores differ by `<= 1` unit.
  5. `ties_are_keyed_by_the_lowest_index`: equal remainders break ties by lowest token id.
  6. `order_and_argmax_are_preserved`: `64` deterministic pseudo-random score vectors preserve order and argmax.
  7. `distinct_scores_permute_masses_exactly`: candidate permutation equivariance.
  8. `uniform_count_baseline_scores_exactly_log2_vocabulary`: `V = 256` uniform MLE counts score exact `8` bits/byte.
  9. `a_zero_count_is_unscorable_not_infinite`: zero-count MLE entry gets mass `0` and `bits_q12(0)` returns `Err(LikelihoodError::MassZero)`.
- **Pre-registration parameters (`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`):**
  - Stream: `chat-v0-p2` `heldout.u16` (`12,389,242` B, sha256 `e5f400b0e676791fd0458ce810163c5d2542adda80dccc8282f37d9ca8904cef`, `6,194,589` tokens, `17,576,697` stream bytes, `2.837427` bytes/token).
  - Windows & noise: `context = 384`, `stride = 12,098`, primary `512` windows (`196,608` targets, `se_512 = 0.014525` BPB), extension `1024` windows (`se_1024 = 0.010270` BPB) and `2048` windows (`se_2048 = 0.007262` BPB, 95% half-width `0.014234` BPB); uniform floor `4.2292` BPB.

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/.worktrees/antigravity-m3-durable-memory-close/local/target ~/.cargo/bin/cargo test --offline -p uor-r4-integer --lib lawful_likelihood --no-fail-fast`: `9 passed; 0 failed`.
- `uor-harness prep-check`: `FIT  FIT  yes  m4-lawful-likelihood-bpb-2026-10-23.md`.
- `uor-harness check`: `0 error`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/.worktrees/antigravity-m3-durable-memory-close/local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: `95 items OK (ready 11, held 10, decision 1, done 73)`.
- `uor-harness loop report --check --order docs/loop/tasks/m4-lawful-likelihood-bpb/antigravity-113/ORDER.md`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `11 / 11 GREEN`.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`.

## Still open

- None for `m4-lawful-likelihood-bpb` (`#2861`). Execution of the 512-window GPU held-out stream scoring run on `heldout.u16` and the `uor-chat` CLI serving audit belong to `m4-export-uorlut01-cli` (`#2859`).

## Plain summary

The potential-ratio lawful likelihood (`PRLL`, `crates/uor-r4-integer/src/lawful_likelihood.rs`) converts integer candidate scores into exact Q48 probability masses (`1 << 48` total, minimum mass `1`) and fixed-point bits per byte (`bits_q12`, `LOG2_SCALE = 4096`) without softmax, squashing, float, or matrix multiplication. Folding in the owner's recorded `"Yes to both (Recommended)"` answer (`#2861#issuecomment-6379432987`) and the two-seed M1-accepted 19-table `UORLUT01` base artifact pins (`04652a3d…` and `1b49e8bd…`, `#2837`/`#3312`) clears all three prerequisites of the 2026-10-23 review on `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`, which now carries a fresh-context non-author `TEST FITNESS: FIT` review and closes `#2861`.
