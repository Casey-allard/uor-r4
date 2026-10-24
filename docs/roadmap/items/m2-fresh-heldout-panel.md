---
id: m2-fresh-heldout-panel
title: Build a genuinely fresh held-out reply panel for M2
milestone: M2
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m2-grounded-reply.md
scope: [data/**, docs/harness/panels/**, docs/mechanisms/native-geometric.md, docs/labs/m2-fresh-heldout-panel/**, docs/reference/storage.md]
---

## Why it matters

Without a fresh panel the M2 bar of 40% on genuinely fresh questions cannot be attempted. The only earlier fresh check scored 0 of 128 and showed that 437 of 512 was panel fitting. Building and sealing the panel needs no GPU and is open to anyone. See [where we're going](../../start/where-were-going.md).

## What is missing

The saved parent scored 437/512 on the frozen development panel but 0/128 on the opened transfer draw, and the audit calls the reply model a panel-fitting failure on rows already seen. The 128-row draw was opened, so it cannot serve as fresh again. A fresh 256-row panel (`m2-fresh256`) is fully staged and sealed: 256 rows / 128 opposite-role histories / 16 blocks, sealed artifacts (`plan.json 0ed1ab98…`, `inputs.json 1de2fc28…`, `labels.json 3b5c5477…`, `manifest.json 7c33f3f7…`), 0 exact/word/2-gram overlaps against all 74 historical literals, all four question-blind and role-swap controls at 0/128 pairs through the real tokenizer + FrozenAnswers + pair_id grader, and measured noise (single-model 16-block cluster SE 6.24 rows, 8.64 at the 40% bar). The owner registered the bar `pairs_both_complete_distinct >= 52/128` (single-row complete >= 104/256). The non-author FIT of construction/overlap/control/seal passed (deepseek-43); `frozen: false` only because the maintainer upload to the HF store is outstanding, and no candidate has been scored (NOT_RUN; M2 training stays HELD). The M2 bar needs at least 40% on fresh questions, with the existing graders unchanged.

The frozen panel file (`docs/harness/panels/m2-reply512.json`) names the grader as the saved native learner's own complete-reply evaluator (`uor-r4-core::native_geometric` compiler, exact store, emitter). CHARTER §4.3 asks for at least 2 seeds or at least 200 fresh rows with bars no narrower than measured noise. The 128-row draw was below that; the staged panel states its measured noise.

## Done when

A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

## How to start

1. Read the old panel and why it failed. Open `docs/harness/panels/m2-reply512.json`, `docs/research/open-questions.md` (line 74), `docs/mechanisms/native-geometric.md`, `CHARTER.md` §4.3 and `STATUS.md`. The `docs/labs/m2-masked-train-reference-2026-10-13/README.md` file cited as a source does not exist in this checkout; use the panel file instead. From the repository root, build the harness (`CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo build --release -p uor-r4-harness`), put it on your path (`export PATH="$PWD/local/harness-target/release:$PATH"`, as `docs/harness/README.md` does) and run `uor-harness brief -m m2` to see the milestone context.
2. Post a written registration draft on #2030 as your first deliverable. It must say how the fresh questions are produced, and that the source is independent of the 512 development rows, the opened 128-row draw and every training corpus. It must say how overlap is excluded (exact and near-duplicate checks against all three), the row count (at least 200, so 40% is at least 80; choose it from a noise estimate), and that the grader is not touched.
3. Measure noise before fixing the bar: score the saved parent on the development panel with the existing grader and record the spread you can measure. The panel file says no spread is stated, so do not invent one. The bar must not be narrower than what you measure.
4. Build the panel with Rust tooling only (no new Python). Freeze it: record its row count and content hash, add it as a new frozen entry beside the old one in `docs/harness/panels/`, and keep the rows sealed and unscored. Do not commit the corpus itself. `docs/reference/storage.md` says where stored bytes live: the private HF store `caseyallard/uor-r4-store`, whose `data/` prefix holds shared inputs. Because that store is private, hand the sealed bytes and their hash to a maintainer to place there; the repository carries only the frozen panel entry (row count and hash).
5. Ask a non-author (a fresh-context reviewer counts) to check the overlap exclusion and the seal before anything scores against the panel. The M2 training run stays HELD until the owner releases it, and this item does not release it.
6. Open a PR that says `Closes #<this issue>` when the sealed panel and its registration are in. Mention #2030 (M2) in the PR. There is no M2 notebook page yet; add `docs/research/notebook/m2-grounded-reply.md` in the format of `docs/research/notebook/README.md` with one row for the panel.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:17`](../../../STATUS.md)
- [`docs/research/open-questions.md:74`](../../../docs/research/open-questions.md)
- issue#2030

