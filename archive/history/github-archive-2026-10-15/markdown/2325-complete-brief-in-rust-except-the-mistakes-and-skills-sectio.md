# #2325 — Complete brief in Rust except the mistakes and skills sections (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T13:03:13Z
- **Updated:** 2026-10-13T13:05:41Z
- **Closed:** 2026-10-13T13:05:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2325
- **Merged:** 2026-10-13T13:05:22Z
- **Merge commit:** d8109a8f13b842388ccd49bf49bc408af6168131
- **Base / Head:** main / UOR-Foundation:deepseek/brief-full-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Session task (1), the Rust port (D27). Continues #2324.

## Result

Decision: **KEEP** (tooling).

`registries.rs` gains the **decisions registry** — `DECISIONS.md` headings parsed into a view. `briefing.rs` gains the prose extractors and the rest of the assembler:

| | section | source |
|---|---|---|
| 1 | the task's state | registry (#2324) |
| 2 | owner rules in force | registry |
| 3 | session contract, clauses 1–10 | `session-goal.md`, reflowed |
| 3b | the lab prompt (FINISH LINE) | `session-goal.md`, fenced |
| 4 | governance decisions | `DECISIONS.md` |
| 5 | mechanism briefs | `docs/mechanisms/` |
| 6 | frozen acceptance panel | `docs/harness/panels/` |
| 9 | receipts in the last 24h | the ledger |
| 10 | the task as stated, and the loop | the caller |

**Verified byte-identical to Python — 81 lines across 16 shared sections:**

```
brief shared sections and token counts identical: ok
```

The parity check needed **no change**, because it already compares every section both implementations provide.

## Four bugs the comparison caught — each invisible to my unit tests

1. **`1 . **Your number.**`** — the clause split swallowed each clause's own full stop. **Worse than cosmetic:** the contract is the section a lab quotes back, and renumbering a binding document is a real change to it.
2. **The lab prompt lost its fence language tag.** Python renders `text Apply the common start prompt…`. It reads oddly and **it is what the authority produces**, so the port keeps it — stripping it was a silent difference between two tools reading the same file.
3. **`posted=false` vs `posted=False`**, and 4. **`(frozen: true)` vs `(frozen: True)`** — Python capitalises booleans, Rust's `to_string` does not. Two tools disagreeing about the same receipt.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 106 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 319 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check           -> 0 errors
python3 scripts/check_no_softmax.py            -> OK (1648, unchanged)
python3 scripts/harness/parity.py              -> all ten comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/brief-full-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **Two of ten sections remain Python**: the lessons that apply (7) and the skills (8). They need the `mistakes` and `skills` registries — separate modules, separate units of the port. **The Rust `brief` prints a line naming them** rather than letting the brief look complete.
- **Parity covers `-m m1`.** The no-milestone table, a query title, and a milestone with no panel are tested in Rust but **not** compared against Python.
- **`contract_list` matches Python's regex structurally, not exactly.** Python splits on a zero-width lookahead for `\d{1,2}\.\s+\*\*`; this splits on `" . **"`. They agree on the live document and on the clause separator, but a clause written without the space before its full stop would split differently. The test pins the live output, not the equivalence.
- **A reflowed contract is a rendering, not the authority** — `session-goal.md` remains binding and the brief cites it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

