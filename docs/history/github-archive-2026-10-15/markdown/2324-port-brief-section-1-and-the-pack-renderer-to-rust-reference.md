# #2324 — Port brief section 1 and the pack renderer to Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T12:54:53Z
- **Updated:** 2026-10-13T12:57:19Z
- **Closed:** 2026-10-13T12:57:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2324
- **Merged:** 2026-10-13T12:57:00Z
- **Merge commit:** 0cf3d1d6b6665242fd7bda6bf94b3cc7ed1f00dd
- **Base / Head:** main / UOR-Foundation:deepseek/brief-rust-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Session task (1), the Rust port (D27).

## Result

Decision: **KEEP** (tooling).

`crates/uor-r4-harness/src/briefing.rs` adds `human_tokens`, `ContextPack` (blocks, provenance, dropped, render), the title, **section 1 (the task's state)** and the **owner rules**. New command `uor-harness brief [-m <milestone>] [-q <query>]`.

**Verified byte-identical to Python on the sections both provide** — compared against real `brief` output, not a helper, because the brief is what a lab reads to learn its task:

```
brief shared sections and token counts identical: ok
```

Content is compared block by block, so Python's unported sections are excluded rather than allowed to mask a difference — and **the token counts are compared too** (`~190` and `~652`, identical in both). A budget estimate that disagrees with the tool that produced it is a brief nobody can budget from.

## The port is a slice, and it says so

The Python assembler is 452 lines and most of it reads and splits **prose** out of `session-goal.md` and `DECISIONS.md` — the session contract, the lab prompt, the governance decisions, the mechanism briefs and the receipt tail. That is a larger unit than one delivery, so this ports the part built on the registry layer already in Rust, and **`brief` prints a line naming what it does not provide.**

**A brief that silently lacked its session contract would read as a brief with no session contract.**

### Two design points

- **`human_tokens` counts characters, not bytes** — `chars / 4`, floored, minimum 1. A brief is full of em-dashes and arrows, and the same text must cost the same regardless of encoding.
- **An empty body is dropped with a reason**, not rendered as a heading with nothing under it. A bare heading reads as *"there is nothing here"*, which is a different claim from *"this could not be read"* — and only one of them is true.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 106 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 319 passed
python3 scripts/harness/cli.py check           -> 0 errors
python3 scripts/check_no_softmax.py            -> OK (1648, unchanged)
python3 scripts/harness/parity.py              -> all ten comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/brief-rust-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **`brief` is a partial replacement.** Five of its sections are still Python; a lab needing the session contract or the mechanism briefs must run `cli.py brief`. The Rust command says so on **every** run rather than implying completeness.
- **The parity comparison covers `-m m1` only.** The no-milestone table and a query title are tested but **not** compared against Python, so a divergence in those forms would not be caught.
- **`ContextPack::add` drops empty content silently from the caller's view** — it records the reason in `dropped`, but a caller that does not render `dropped` loses it. The renderer includes it, so the loss is visible in the output, not in the API.
- `brief` is one of six remaining Python modules: `mistakes` (805), `prlint` (1133), `status` section 4, `doccheck` (374), and `check`'s 51 rules.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

