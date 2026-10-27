---
id: m1-close-saved-model
title: Close M1 with one saved discrete model passing all five bars
milestone: M1
area: training
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks]
scope: [crates/uor-r4-discrete/**, crates/uor-r4-integer/**, docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**, docs/research/notebook/m1-discrete-attention.md]
notebook: docs/research/notebook/m1-discrete-attention.md
---

## Why it matters

This is the M1 closing condition and unblocks M2, M4 and everything after.

## What is missing

Nothing passes today: best top-1 is 65 and 463 of 14,287 against 1,226 and 6,300; K16-A fails G-Adm on both seeds (2,237 and 3,167 against 4,894) and K16-R collapsed to 19 or 20 occupied cells. The 116/232 reply bar is BLOCKED under R14 and is not a closing condition. The #2029 body still lists the old 0.90 BPB bar, which the 19 October owner decision replaced.

**Current state on `main` (26 Oct 2026, `antigravity-100` #3293).** All upstream M1 building blocks and evaluators are now on `main`: `m1-a8-full-dose-rerun` (`#2831`, `#3232`) and `m1-bvg-k-stage-t-training` (`#2836`, `#3239`) reached `5,009 / 14,287` DEV admission (`G-Adm` PASS, `256 / 256` occupied cells `G-Occ` PASS) with checkpoint-restricted `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084`; `m1-within-cell-writer` (`#3267`, `#3280`) proved that `90.48%`–`92.18%` of missed `T_restr` hits lose within their own cell under `OuterL1V1`; `r-rank-objective-keeps-cells` (`#2883`, `#3290`) is `done`; `m1-export-gap-and-toks` (`#2839`, `#3287`) qualified exact zero `UORLUT01` export gap (`14,287 / 14,287` row agreement) and `232,126–232,879` response `tokens/s` on 1x RTX 5090; and `m1-fresh-panel-transfer` (`#2841`, `#3292`) landed the `d11-scorer fresh-score` gate and `TEST FITNESS: FIT` sheet (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`, held on a two-seed DEV bar-(a)–(d)-passing model). What remains is assembling and evaluating a two-seed discrete geometric model with exact-memory within-cell token selection exported to `UORLUT01` that passes bars (a)–(d) on `ft/dev` (`> 1,226` unigram and `> 6,300` `T2`) and bar (e) on the sealed fresh panel (`#2841`).

## Done when

One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

## How to start

Open these first:

- `CHARTER.md (4.8)`
- `docs/integration/DECISIONS.md (D39)`
- `docs/start/where-were-going.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:14`](../../../ROADMAP.md)
- [`docs/start/where-were-going.md:26-36`](../../../docs/start/where-were-going.md)
- issue#2029

