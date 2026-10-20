---
id: m3-nonrelation-prose-panel
title: Build a pre-registered non-relation prose panel
milestone: M3
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

M3 acceptance requires that non-memory prose never overwrites stored facts. Today nothing measures how often it does. This panel is the instrument that the [memory-turn discriminator gate](m3-memory-turn-discriminator-gate.md) and M3 qualification both read. It sits in front of the exact store described in [the relation read](../../mechanisms/relation-read.md).

## What is missing

In world=v2, ordinary small talk can compile as ASSERT or CORRECT. An example in #1718 compiled to CORRECT on relation 4 and overwrote a stored fact. No pre-registered set exists to measure false-assert and false-overwrite rates.

## Done when

A pre-registered non-relation prose panel with grading, plus the baseline false-assert and false-overwrite rates on world=v2. The pass bar for those rates is not yet registered; registering it is step 2 below.

## How to start

1. Read `crates/uor-r4-training/src/relation_compiler.rs` (`SavedCompiler::action`, around line 2219, and `table_action`, around line 2299) and `crates/uor-r4-training/src/stack_grounded_session.rs` (`apply_action`, around line 650). The panel must grade the `CompiledAction` the compiler returns (`Assert`, `Correct`, `QueryCurrent`, `Query` or `Unresolved`) and the `MemoryEffect` the session records (`Write`, `WriteDisabled`, `Read`, `Unresolved`).
2. Post a design note on the issue. It should define: the panel categories (small talk, opinions, questions with no relation, anaphoric turns, near-misses that look like statements); the grading rule (a turn is clean only if the action is `Unresolved`); the two rates, false-assert (ASSERT on a prose turn) and false-overwrite (CORRECT on a prose turn that changes a stored fact); and a proposed pass bar. Pass bars belong to the owner (CHARTER §5.2), so send the proposed bar as a decision card to the PM rather than fixing it yourself.
3. Write the panel as a plain-text file with one turn per line and a hash. Freeze it before looking at any compiler output: hash the file with SHA-256 and record the hash in the sheet. Keep no committed corpus files beyond the small panel the sheet names (CHARTER §3.13). Add a held-out slice that the world=v2 training data never contained.
4. Add the grader as Rust, with finite CPU tests, in a new module beside `relation_compiler.rs` or in `crates/uor-r4-training/tests/`. The tests should feed a stub compiler (the test code in `stack_grounded_session.rs` has a `TestCompiler`) and check each rate on hand-built cases. Run `~/.cargo/bin/cargo test -p uor-r4-training`.
5. Draft the one-page pre-registration sheet (panel hash, grader commit, rates, bar) and ask for a non-author FIT review. Running the world=v2 compiler over the panel to get the baseline rates is a GPU run, so it needs that FIT and a lease (CHARTER §4.5); the panel and grader do not.
6. After the FIT and an owner release, run the baseline on world=v2 and record both rates with the panel hash.
7. Open a PR that says `Closes #<this issue>` (or `References` if only the panel and grader landed). Record the result as one row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1718)
- `crates/uor-r4-training/src/relation_compiler.rs` (action policy, around lines 2219–2342)
- `crates/uor-r4-training/src/stack_grounded_session.rs` (`apply_action`, around lines 650–712)
