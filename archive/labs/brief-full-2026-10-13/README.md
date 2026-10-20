# `brief` is now Rust for everything but the mistakes and skills sections

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** session task (1), the Rust port (D27). Builds on #2324.

## What was added

`crates/uor-r4-harness/src/registries.rs` gains the **decisions registry** — `DECISIONS.md` headings
parsed into a view (`parse_decisions`, `is_decision_heading`, `Registries::decision`,
`Registries::decisions_for`). `briefing.rs` gains the prose extractors and the rest of the assembler:

| | section | source |
|---|---|---|
| 1 | the task's state | registry (ported in #2324) |
| 2 | owner rules in force | registry |
| 3 | session contract, clauses 1–10 | `session-goal.md`, reflowed |
| 3b | the lab prompt (FINISH LINE) | `session-goal.md`, fenced |
| 4 | governance decisions | `DECISIONS.md` |
| 5 | mechanism briefs | `docs/mechanisms/` |
| 6 | frozen acceptance panel | `docs/harness/panels/` |
| 9 | receipts in the last 24h | the ledger |
| 10 | the task as stated, and the loop | the caller |

**Verified byte-identical to Python: 81 lines across 16 shared sections** — `brief shared sections and
token counts identical: ok`. The parity check compares real `brief` output and does not need changing,
because it already compares every section both implementations provide.

## The four bugs the comparison caught

Every one was invisible to the tests I wrote, and each would have shipped a brief that disagreed with
the tool beside it:

1. **`1 . **Your number.**`** — the clause split consumed each clause's own full stop. Worse than it
   looks: the contract is the section a lab is most likely to quote back, and renumbering it is a
   change to a binding document.
2. **The lab prompt lost its fence language tag.** Python renders `text Apply the common start
   prompt…`; stripping the tag was a silent difference between two tools reading the same file. It
   reads oddly and it is what the authority produces, so the port keeps it.
3. **`posted=false` vs `posted=False`.** Python renders booleans capitalised; Rust's `to_string` does
   not. Two tools disagreeing about the same receipt field.
4. **`(frozen: true)` vs `(frozen: True)`** — the same bug in the panel section.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | 106 passed + 1 doc-test |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed **on 3.15 and 3.9** |
| `cli.py check` | 0 errors |
| `check_no_softmax.py` | OK — 1648, unchanged |
| `parity.py` | all ten comparisons ok |

## Limitations, stated

- **Sections 7 and 8 are still Python**: the lessons that apply and the skills. They need the
  `mistakes` and `skills` registries, which are separate modules and separate units of the port. The
  Rust `brief` **prints a line naming them** rather than letting the brief look complete.
- **Parity covers `-m m1`.** The no-milestone table, a query title, and a milestone with no panel are
  tested in Rust but not compared against Python.
- **`contract_list` matches Python's regex structurally, not exactly.** Python splits on a
  zero-width lookahead for `\d{1,2}\.\s+\*\*`; this splits on `" . **"`. They agree on the live
  document and on the clause separator, but a clause written without the space before its full stop
  would split differently. The test pins the live output, not the equivalence.
- **A reflowed contract is a rendering, not the authority.** The port reflows clauses onto their own
  lines for readability; `docs/labs/session-goal.md` remains the binding text, and the brief cites it.
