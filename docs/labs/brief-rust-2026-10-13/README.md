# `brief` ports to Rust — section 1 and the pack renderer

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** session task (1), the Rust port (D27)

## What was ported

`crates/uor-r4-harness/src/briefing.rs`: `human_tokens`, `ContextPack` (blocks, provenance, dropped,
render), the title, **section 1 (the task's state)** and the **owner rules**. New command
`uor-harness brief [-m <milestone>] [-q <query>]`.

**Verified byte-identical to Python on the sections both provide**, and the parity check compares
real `brief` output rather than a helper, because the brief is what a lab reads to learn its task:

```
brief shared sections and token counts identical: ok
```

Content is compared block by block, so Python's unported sections are excluded rather than allowed to
mask a difference — and **the token counts are compared too** (`~190` and `~652` for the two milestone
sources, identical in both). A cost estimate that disagrees with the tool that produced it is a brief
nobody can budget from.

## Scope, and why it is a slice

The Python assembler is 452 lines and most of it reads and splits **prose** out of `session-goal.md`
and `DECISIONS.md` — the session contract, the lab prompt, the governance decisions, the mechanism
briefs and the receipt tail. That is a larger unit than one delivery, so this ports the part built on
the registry layer already in Rust and **states the remainder rather than omitting it**: `brief` prints
a line naming what it does not provide. A brief that silently lacked its session contract would read
as a brief with no session contract.

## Design notes

- **`human_tokens` counts CHARACTERS, not bytes** — `chars / 4`, floored, minimum 1. A brief is full
  of em-dashes and arrows, and the same text in UTF-8 and UTF-16 must cost the same.
- **An empty body is dropped with a reason**, not rendered as a heading with nothing under it. A bare
  heading reads as "there is nothing here", which is a different claim from "this could not be read",
  and only one of them is true.
- **The title has three forms** — milestone, query, or neither — and all three are live.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **106 passed** + 1 doc-test |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `cli.py check` | 0 errors |
| `check_no_softmax.py` | OK — 1648, unchanged |
| `parity.py` | **all ten comparisons ok**, including the new brief check |

## Limitations, stated

- **`brief` is a partial replacement.** Five of its sections are still Python; a lab that needs the
  session contract or the mechanism briefs must run `cli.py brief`. The Rust command says so on every
  run rather than implying completeness.
- **The parity comparison covers `-m m1` only.** The no-milestone table and a query title are tested in
  Rust but not compared against Python, so a divergence in those forms would not be caught.
- **`ContextPack::add` drops empty content silently from the caller's view** — it records the reason in
  `dropped`, but a caller that does not render `dropped` loses it. The renderer includes it, so the
  loss is visible in the output rather than in the API.
