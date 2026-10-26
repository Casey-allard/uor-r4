# REPORT: d11-exporter-float-trunk, claude-124

Lab: claude. Order: [ORDER.md](ORDER.md). Pre-run review (D67): #2820 comment `6409295624`, PRE-RUN: GO.

References #2820.

## What did not work, first

- **The float sheets' D11 export mode was wrong at a single position, and this unit's parity run found it.**
  - At `06148c7a` the export-mode GPU readout matched its reference on only 1 of 256 single tokens (largest hidden gap 4.975). Full-context sequences in the same mode matched to 4.3e-7.
  - The same token read alone and read at position 0 of a sequence gave different outputs, so the forward was not causal.
  - **Cause:** `try_hamilton_gate_fwd_cuda` (`hamilton_sheet.rs`) rebuilt its gate operand with `Tensor::from_storage(s.try_clone(l), shape)`. candle's CUDA `try_clone` copies the whole buffer and ignores the layout. At one position the recurrence output gate is a narrow of `rec.in`'s output: contiguous, but starting at offset `d`. So the hook snapped the drive half instead of the gate. With a batch or more than one position the narrow is not contiguous and gets copied first, which is why sequences were right.
  - **Repaired here:** `cuda_layout_tensor` narrows the copy back to the operand's own elements. The telemetry copy of `up` had the same pattern and is fixed too.
  - **Pinned:** the GPU test now requires a token read alone to equal that token at position 0 of each sequence, in both modes. With the repair set aside it fails ("export_mode true: token 1 alone differs from position 0 of its sequence, gap 2.33").
  - **Reach, by reading:** every other export-mode caller in the crate scores full windows: `hamilton_sheet.rs` `evaluate_dev_panel` and `spiralcore_fbs_sheet.rs` `panel_loss`, both through `model.hidden(…, SHEET_CONTEXT)` with context 192. The bug cannot occur there, so their published numbers stand. Only a one-position forward in export mode was wrong.
  - **Not pinned:**
    - The same repair to the telemetry copy of `up` has no test; only telemetry reads it.
    - `with_d11_export_gates` restores the hook when its closure returns an error, but not when it panics. The state is thread-local.
- **My first parity design proved nothing about the recurrence's memory.** Single tokens from a zero state never reach the transition rotation, convolution taps 1 to 3 or the carried state. The pre-run reviewer caught it. Before any run, the readouts gained 8 fixed full-context sequences (64 positions), and the ORDER's bars were amended, once and before the run, to cover them.
- **I lost uncommitted edits once, and recovered them.** I set aside a line to build a patch, then ran `git checkout --` on a file that also held my uncommitted sequence edits, which threw them away. The diff I had just written held them, and I restored them from it. Set-asides now run only on a clean, committed tree.
- **My first switch test did not pin each snap site (round-1 review).** It compared whole reference variants, which differ at more than one site, so either gate site could have ignored its mode and the test would still have passed.
  - **Fix:** a hand-computed test now checks the feed-forward gate block by block, and a one-lane recurrence over two steps: the output gate from a zero state, then the transition. It runs under four snap settings. Setting aside any one of the three sites fails it.
  - **Still not pinned by a host test:** that `forward_f32_with` passes `snaps.gates` to the feed-forward call. The GPU parity reading covers it, because the plain readout would miss `FLOAT` otherwise.
- **Still a random-init trunk.** No trained model ran. The export gap of a trained trunk, the Stage 4 sheet and tok/s are still owed.

## What changed

Size against `origin/main`: code +602/−97 in 7 files, including the two in-file test modules; tests +118 in `stack/tests.rs`; docs: the ORDER, this REPORT and the notebook row.

- **`uor-r4-integer`:**
  - `FbsTrunkSnapshot::forward_f32_with(tokens, quantized, FbsReferenceSnaps)` adds snap switches for the transition and the gates (`FbsQuatMode::{Icosian, Q12Icosian, Normalize}`). The constants are `INTEGER`, `FLOAT` and `FLOAT_EXPORT`.
  - `Normalize` copies the float CUDA epsilons: `1e-12` for the Hamilton gates and `1e-6` for the transition.
  - `forward_f32` is `forward_f32_with(INTEGER)`, unchanged. The dense reference passes the snapped default.
  - `FbsFloatReadout` gains `export_mode` and `sequences`. `FbsFloatReadout::check` refuses a readout that is incomplete, repeated, out of order, in the other mode, or holds a bad sequence.
  - `fbs-compare` reports single tokens and sequence positions separately. Each section holds:
    - each readout's parity with its matching reference;
    - the integer control;
    - the attribution of the gap, reference variant by reference variant.
- **`uor-r4-float-train`:**
  - `fbs_float_readout(model, export_mode)` reads every single token plus 8 full-context sequences. Export mode goes through the new `hamilton_sheet::with_d11_export_gates`, which restores the whole gate hook afterwards.
  - `fbs-snapshot` takes `[READOUT_JSON [EXPORT_READOUT_JSON]]`.
  - The hook repair, `cuda_layout_tensor`.

## Measurements

On an RTX 5090, random-init trunk (vocabulary 256, width 32, 2 layers), at the code head `5fdec38a`. That head is the pod's `06148c7a` source plus `git diff 06148c7a..5fdec38a` applied as two patches. Later commits add only this REPORT, the notebook row, and a type alias in `fbs-compare` for clippy, with no change in behaviour.

**Parity: every bar holds.** Bars: largest hidden gap ≤ 1e-3; at least 250 of 256 single tokens agree, and at least 60 of 64 sequence positions.

| Float GPU against its matching reference | Single tokens | Sequence positions |
|---|---|---|
| plain against no snaps | 256/256, gap 9.7e-7 | 64/64, gap 7.1e-7 |
| export mode against `FLOAT_EXPORT` | 256/256, gap 4.6e-7 | 64/64, gap 4.3e-7 |
| integer session against the quantized reference | 256/256 | 64/64 |

So on this random-init trunk shape the converter maps every table right, including the transition, taps 1 to 3 and the state carry. Other shapes and trained weights are not covered by this run.

**Attribution: a reading, no bar.** Each row is the token agreement between adjacent reference variants:

| Step | Single tokens | Sequence positions |
|---|---|---|
| Hamilton-gate icosian snap (`FLOAT` against `FLOAT_EXPORT`) | 67/256, gap 2.825 | 9/64, gap 2.825 |
| transition snap, and plain against Q12-rounded gate snap (`FLOAT_EXPORT` against `INTEGER`) | 256/256, gap 0.536 | 37/64, gap 1.779 |
| table rounding: grid codes, Q16, Q12 (`INTEGER` plain against quantized) | 232/256, gap 2.942 | 56/64, gap 1.198 |

claude-123's 67/256 is therefore the Hamilton gate snap, entirely. At a single position the transition snap cannot act (zero state). Over sequences it changes a further 27 of 64.

**Set-asides and controls:**

| Check | Result |
|---|---|
| Set-aside 1, route decode without its sign bit | the GPU test fails at its bit-equality assert |
| Set-aside 2, conv taps reversed in the converter | parity collapses: plain 0/256 (gap 4.87) and 1/64 (gap 4.67); export 1/256 and 0/64 |
| The causality check without the hook repair | fails, gap 2.33 |
| Host set-asides | each fails its test: the `Normalize` arm; the Q12 rounding; the recurrence ignoring its snaps; the readout's order check; and, one at a time, the transition site, the recurrence output gate site and the feed-forward gate site |
| Artifact | `fbs-export` exits 0; sha256 `0621616ab3d6a9786be8e1e82569d6bf6539a854fbe8775cd5c47d6622c9447a`, the same as claude-123's |
| Suites | float-train 31; integer serving-build library 320; no-feature library 55; `m5_full_path_cost_controls` 3/3; `memory_shape` 3/3 |
| Zero-multiply audit | `FULL PASS`: rlib 56 symbol ranges and 40 call-graph functions; `uor-r4-stack` 44 and 90 |
| GPU cost | one RTX 5090, about 8 minutes for both jobs, at $1.09/h; `vast-57772919` deleted |

Evidence: the HF store `results/claude/d11-fbs-parity-2026-10-26/`, with a `SHA256SUMS`:
- `fbs-parity/` holds the run at `06148c7a`;
- `fbs-fix/` holds the repair run.

## Checks run at the head

See the exact-head review record on the PR:
- the suites above;
- the stub-nvcc `cargo check --features cuda --all-targets`;
- clippy;
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** The parity bar exposed a defect outside the converter. The unit repaired it, because the export gap Stage 4 measures depends on export mode being right.
- **writing-meaningful-tests.** Every new check was set aside to show it can fail, the parity reading included: the reversed conv taps.

## Still open (Stage 4)

- **The Stage 4 sheet.** It needs:
  - a fresh non-author FIT;
  - a two-seed retrain saved with `StackModel::save`;
  - `fbs-snapshot` and `fbs-compare` on the trained trunks;
  - the export gap on the sheets' evaluator, now that export mode is right at every position count;
  - laptop tok/s, with decode and context reported separately.
- **The attribution says where the gap comes from.** Stage 4 should read it on the trained trunk before deciding what to train against: the gate snap dominates on this random init.
- **Serving front ends and the pages,** as claude-123 listed.

## Plain summary

The previous unit could not say why the float model and its integer copy disagreed on most next tokens. This unit adds switches to the float reference, so each rounding step the integer version adds can be turned off.
- With every switch off, the float GPU model and the converted copy agree to within a millionth on every token and every sequence position. So the converter copies every number to the right place.
- The disagreement comes almost entirely from one rounding step: the integer version rounds the gate rotations to a fixed set of 120.
- The check also found a real bug in the float side's "export mode": it read the wrong numbers whenever it processed exactly one token at a time. That is now fixed and pinned by a test.
