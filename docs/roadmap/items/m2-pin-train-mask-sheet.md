---
id: m2-pin-train-mask-sheet
title: Settle whether the pinned M2 TRAIN-mask sheet is live after D37, and re-pin it
milestone: M2
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-training/**, data/**, docs/harness/prep/m2-pin-train-mask-sheet*, docs/harness/milestones.json, docs/labs/m2-pin-train-mask-sheet/**]
---

## Why it matters

M2 is the first simple conversation: at least 256 of 512 saved questions and at least 40% of fresh ones, graded by fixed deterministic checks ([where we are going](../../start/where-were-going.md), Stage 5). Before any GPU training or fresh-transfer evaluation, the charter requires a one-page pre-registration sheet with a non-author FIT (CHARTER §4.5). The masked TRAIN count reference is built and tested, and the Rung 0 sheet already carries its pins (see below). What no record settles is whether that sheet is still the live one. A sheet that is ready for FIT shortens the path once the owner decides on the M2 hold. See also [the transformer-to-geometry map](../../start/transformer-vs-geometry.md).

## What is missing

The record of the count reference (`archive/labs/count-reference-2026-10-13/README.md`) and the `next` text in `docs/harness/milestones.json` still say the physical sheet verdict is PENDING until the sheet adds four pins: the mechanical TRAIN-mask path and checksum, the expected count, the CLI, and the new source. **That text is stale on the pins.** Amendment 6 of the sheet (`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`, commit `be5d8cd6e`, #2330) added them and restored the verdict to `TEST FITNESS: FIT` (independent Claude review, `#2029` comment 6146716704, at `9e904a834` plus Amendments 5 and 6). The sheet pins:

- mask member `mw-balp/train/response_mask.u8` (7,512,797 bytes, SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`), taken from the parent archive `data/ft-balp.tar`;
- `expected_train_targets=2425925` (an independent data-only check counts 2,425,925 eligible TRAIN targets);
- the full `geometric-stack train ...` CLI line, which refuses to run without `train_mask`;
- the source: the reference is frozen at `2fd6358bbc52d2a202a872ad0954ebb12894606f`, which passes 11 finite controls (9 metadata, 2 CUDA), and protected PR #2328 merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`.

What is not settled is what that sheet is today. Amendment 7 (D32, 13 October) froze it as a design record that is not executable, because its training law is a floating-point optimizer, and it called for a new pre-registration on a discrete learning law. D37 (18 October, `docs/integration/DECISIONS.md`) later revised D32's learning-path float ban: floating point is legal in offline training with a reported export gap. No record says whether D37 makes the Amendment 6 sheet live again or whether a new sheet is still wanted. M2 training and rental stay HELD until the owner releases them, and a FIT releases nothing.

## Done when

A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## How to start

1. Read the prep sheet `docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`: the CLI and mask pins (lines 228 to 235), `## Verdict` (line 319) and `## D32 freeze (Amendment 7)` (line 334). Then read D37 in `docs/integration/DECISIONS.md` and the count-reference record. Note where the record and `docs/harness/milestones.json` still say PENDING.
2. Post the stale-PENDING finding on `#2030`, with the pin values copied from the sheet, and ask the PM whether the Amendment 6 sheet is live under D37 or a new discrete-law sheet is wanted. Do not decide this yourself: a change to a status or bar is a decision card to the PM or owner (CHARTER §5.2). Record the answer.
3. Re-check the pins against current main, on a CPU, with no training. Inspect the pinned source with `git show 2fd6358bbc52d2a202a872ad0954ebb12894606f --stat`, then read `crates/uor-r4-training/examples/geometric-stack/native_rung0.rs` (the response-masked TRAIN count) and `crates/uor-r4-training/examples/geometric-stack-native.rs` (the `geometric-stack` example target). Build it with `~/.cargo/bin/cargo build -p uor-r4-training --features native-rung0 --example geometric-stack`. The example may need CUDA for its two CUDA controls; the 9 metadata controls are host checks. The mask file lives in the parent archive `data/ft-balp.tar`, which is not in the repository: ask on `#2030` for access before hashing it with `shasum -a 256`, and without it stop at comparing the sheet text with the source. Never run the `train` command in the sheet: training is held.
4. Draft the one-page sheet on `#2030` as the first deliverable. It pins the mask path and checksum, the expected count, the exact CLI line, and the source SHA, and it states the hold status (training and M2 stay HELD) and the bars unchanged from Amendment 5. If the PM says the sheet is superseded, carry the mask checksum, the count and the source into the new discrete-law sheet instead; the CLI line changes with the learning law and must not be reused as written.
5. Ask for a non-author FIT (a fresh-context reviewer counts under CHARTER §4.5) at the exact head of the sheet.
6. Open a PR that says `References #2030` (the sheet is ready for FIT, but the owner's hold release is not part of this item). Record the result as one row in the closest research notebook under `docs/research/notebook/` (see `docs/research/notebook/README.md`; no M2 page exists yet, so propose the line with the row).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2030
- [`archive/labs/count-reference-2026-10-13/README.md`](../../../archive/labs/count-reference-2026-10-13/README.md)
- [`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`](../../../docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md)
- [`docs/harness/milestones.json`](../../../docs/harness/milestones.json)
- [`docs/integration/DECISIONS.md`](../../../docs/integration/DECISIONS.md) (D37)
