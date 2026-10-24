---
id: m6-reasoning-panel-freeze
title: Build and freeze the m6-reasoning panel and oracles
milestone: M6
area: evaluation
status: done
open_to: anyone
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m6-reasoning-coding.md
scope: [crates/uor-r4-harness/src/panel*.rs, crates/uor-r4-harness/tests/panel*.rs, docs/harness/panels/**, docs/reference/storage.md, docs/labs/m6-reasoning-panel-freeze/**]
---

## Why it matters

The M6 bar, "Executable reasoning and coding on the same model", cannot be attempted without a frozen panel and oracles that grade deterministically. The panel is the shared yardstick for the exact-arithmetic mechanism, the ablation instrument and the final M6 qualification. It is built and frozen before any run it scores. See [where we are going](../../start/where-were-going.md) for how M6 follows M1 to M5.

## What is missing

The frozen m6-reasoning panel does not exist: it must cover unseen multi-step compositions, altered constraints, and executable coding tasks checked by deterministic compilation or test oracles (cargo test, exact output). Status is not started. `docs/harness/panels/m6-reasoning.json` is a stub: `frozen` is `false`, `grader` is empty, `bars` is empty, noise is "unstated - a bar must not be narrower than it", and a decisive run needs at least 200 fresh rows and at least 2 seeds. The numeric bar is not yet registered.

## Done when

A frozen, pre-registered panel with deterministic oracles for compositions, altered constraints and code tasks (D28: no external teacher). CHARTER §3.10 requires that panels are graded deterministically, with no neural comparator, control, teacher or grader.

## How to start

1. Read the stub and the rules. Open `docs/harness/panels/m6-reasoning.json`, `docs/harness/README.md` (the `uor-harness panel show|score` lines), `CHARTER.md` §3.10 and §4.5, `docs/reference/storage.md`, and `gh issue view 2034 --repo UOR-Foundation/uor-r4`. Look at `docs/harness/panels/m2-reply512.json` for what a frozen panel entry carries.
2. Post a panel design note on #2034 as your first deliverable. It must name three row families (unseen multi-step compositions, altered constraints, executable coding tasks). For each it must give how rows are generated from a fixed seed, the exact deterministic oracle (exact match, or compile plus `cargo test` or exact output for code), and the row count. The total must reach at least 200 fresh rows. It must say how overlap with every training corpus is excluded, and how generated code is run safely.
3. Build the generator and oracles in Rust only, with no new Python. Check how panels are wired with `rg -n panel crates/uor-r4-harness/src` and say in the note whether the code lives in `crates/uor-r4-harness` or a new small crate. Tests: the same seed gives the same bytes, each oracle accepts known-correct answers, and each oracle rejects known-wrong answers. Run `~/.cargo/bin/cargo test -p uor-r4-harness` (or your crate).
4. Measure noise with a lawful non-neural baseline such as a count table (CHARTER §4.4), not a neural model. Record the spread you can measure. Do not invent one, and the bar must not be narrower than it. Propose the numeric bar and the seed count as a decision card to the PM or owner; they are not yours to set (CHARTER §5.2).
5. Freeze it. Seal the rows unscored, record the row count and a SHA-256 content hash, and store the bytes as `docs/reference/storage.md` describes; do not commit the corpus. Then update `docs/harness/panels/m6-reasoning.json`: set `grader`, `noise` and `bars` and change `frozen` to `true`, once the owner has registered the bar.
6. Ask a non-author (a fresh-context reviewer counts) for a TEST FITNESS: FIT review of the overlap exclusion, the oracles and the seal before anything scores against the panel. This item releases no training or run.
7. Open a PR that says `Closes #<this issue>` when the frozen panel and its registration are in. There is no M6 notebook page yet. Add `docs/research/notebook/m6-reasoning-coding.md` in the format of `docs/research/notebook/README.md` with one row for the panel.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:19`](../../../ROADMAP.md)
- issue#2034
