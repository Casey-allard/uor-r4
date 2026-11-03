# REPORT: register the served-rounding cost item, deepseek-78

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base: `a9d0461e2`.

## What did not work

- **The normal claim was refused.** `one-model-end-to-end` is already claimed by
  open PR #3583. Its batched-scorer unit is independent, so adding this repair
  to that PR would combine two units.
- **The activation-scale loop is not established as the cost cause.** Replacing
  its 49 sequential passes with one bounded host exponent read preserved scale
  bits, quantized values and straight-through gradients on every tested
  threshold and 1,000,000 random f32 inputs. A full
  `16 x 192 x 256` CPU activation took 4.94 ms with that replacement and 5.03 ms
  with the original in the release probe, about a 2% change.
- **The vectorized threshold-count variant was slower on CPU.** It took 5.30 ms
  for the same full activation. It may still change GPU launch cost, so the
  host probe does not select or reject it for CUDA.
- **The CPU host cannot run the qualified training graph.** It correctly
  refuses the CUDA SpiralCore/FBS model, so the complete stage profile remains
  GPU-only.
- **The power interruption lost the original local profiling context.** The
  probe was rebuilt from the pinned source; no prior measurement was reused.
- **The first exact-head review requested changes.** It found that the item
  could select activation or table-copy repairs after profiling but did not
  scope their source files, and that its rerun condition did not require
  passing the doors. deepseek-79 repairs both.

## What changed

This registration unit adds one roadmap item, two ORDERs, this REPORT, and the
shared claim entry that records the branch. It does not change serving code, an
artifact, a model, an acceptance bar, or Claude's active copy-key branch.

The initial proposed implementation scope was the integer snapshot quantizer,
which runs on every `--train-as-served 1` step without overlapping PR #3583's
batched-scorer files. deepseek-79 extends the final item scope to the float
activation, export-rounding and served-loop sources that its profile can
select.

Size at the record-repair head: 8 changed files, 434 insertions, 0 deletions;
seven documentation records and one shared claim entry. No code or tests
changed.

## Measurements

Host release probe on the AMD Ryzen 9 9950X workstation:

| Probe | Current | Candidate |
|---|---:|---:|
| 9,472 export grid values | 10.799 ms | 0.426 ms |
| `16 x 192 x 256` activation | 5.03 ms | 4.94 ms |

The bounded grid implementation matched the current output on 1,000,000
random finite f32-derived values and on every grid code plus the adjacent f64
below and above it. This is diagnostic evidence for the new item, not a GPU
stage attribution or an operational-door pass.

## Checks run at the head

- `cargo run --locked -p xtask -- roadmap check`: 135 items OK.
- `git diff --check origin/main...HEAD`: passed.
- Host Rust/Candle probe: forward and straight-through controls passed.
- Remote CI at pre-repair head `6e3ce8da7`: 10 checks passed; 13
  path-appropriate checks skipped.
- `uor-harness check`: exit 0 with no errors; repository-wide advisory warnings
  did not name the changed files.
- Full CUDA profile and four-job rerun: `NOT_RUN` on this host.

## Skills used

No available Codex skill applied. The repository charter, loop instructions,
served-comparison report, floating-point exporter and existing task records
were read directly.

## Still open

- Land this item registration.
- Claim `served-rounding-cost` once its scope is present on fresh `main`.
- Add the bit-exact bounded grid repair and its regression tests.
- Run the GPU stage profile before changing activation or table-copy mechanics.
- Rerun the unchanged four-job comparison only after the exact repairs.

## Plain summary

Training as served currently takes almost seven times as long as plain training,
so longer model runs would waste most of their time in overhead. The new item
gives that focused repair its own claim, scope and exact rerun. A small host
probe found a likely table-rounding cost that can be made about 25 times cheaper
without changing its output, but it did not prove that this alone will pass the
2x gate.
