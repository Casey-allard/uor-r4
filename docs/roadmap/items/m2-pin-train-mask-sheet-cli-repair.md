---
id: m2-pin-train-mask-sheet-cli-repair
title: Repair the discrete M2 sheet's non-executable FIT
milestone: M2
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [m2-pin-train-mask-sheet]
notebook: docs/research/notebook/m2-grounded-reply.md
scope: [docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md, docs/labs/m2-pin-train-mask-sheet/**, docs/loop/tasks/m2-pin-train-mask-sheet/**, docs/research/notebook/m2-grounded-reply.md]
---

## Why it matters

A FIT sheet is the authorization basis for later GPU work. A flag-only repair is insufficient if the registered training and deterministic reply-evaluation chain cannot execute end to end.

## What is missing

Post-merge audit of #3300 found that the discrete sheet's Stage T and trunk commands omit required arguments and run trunk Step 0 before its checkpoint exists. The final M2 grounded-reply evaluation has no command, and the loop only repeats the earlier M1 Stage T/trunk recipe rather than demonstrating the M2 student runner. These are instrument defects, not model evidence.

## Done when

The sheet no longer presents a non-executable chain as FIT. Record the exact CLI and missing-runner findings, replace the misleading recipe and verdict with `NOT_FIT` unless the complete chain exists, bind the remaining implementation to `m2-train-grounded-replies`, and obtain fresh D68 and non-author fitness reviews. No GPU work or model execution.

## How to start

- Compare `docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md` with `crates/uor-r4-discrete/src/bin/stage-t-run.rs` and `trunk-run.rs`.
- Preserve the valid data pins and bars; do not change the milestone bar or run a model.

## Rules that apply

- **Pre-registration first** (CHARTER §4.5): a FIT must describe a configuration that can actually run; this host-only repair needs no GPU sheet or pod.
- **Instrument first** (§4.2): classify the invalid recipe as an instrument defect, never as a model or mechanism negative.
- **GPU only** (§3.5): do not execute model training, evaluation or parity on the host while repairing the record.
- **Records** (§5.6): preserve the original finding and add one notebook row with exact negative scope.

## Sources

- [`docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`](../../../docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md)
- [`crates/uor-r4-discrete/src/bin/stage-t-run.rs`](../../../crates/uor-r4-discrete/src/bin/stage-t-run.rs)
- [`crates/uor-r4-discrete/src/bin/trunk-run.rs`](../../../crates/uor-r4-discrete/src/bin/trunk-run.rs)
