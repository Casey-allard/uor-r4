# #2392 — The pre-registration sheet was omitting D31 and D32 (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T22:47:30Z
- **Updated:** 2026-10-13T22:50:01Z
- **Closed:** 2026-10-13T22:49:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2392
- **Merged:** 2026-10-13T22:49:29Z
- **Merge commit:** 431aabeb67ab7dd0f06b28eb26493b4a16c19653
- **Base / Head:** main / UOR-Foundation:deepseek/d31-d32-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The `prep` sheet was omitting D31 and D32.** Tooling only.

Decision: **KEEP** (tooling).

## What was wrong

`AGENTS.md` gained a **D32** block during this session, and I noticed while reporting that the `prep` rule text still named D21–D29. Verified:

```
registry owner_rules: D21 D22 D23 D24 ... D25 ... D28 D29
D31 present: False      D32 present: False
APPLICABLE_RULES: ("D21","D22","D23","D24","D25","D28","D29")   # both implementations
```

**Both D31 and D32 had been issued and neither was on the sheet a lab receives.**

## Why this is not a cosmetic gap

**This is the defect that put a banned grader on a sheet, one rule later.** The rule text a lab reads **is the rule text it follows**, and a `prep` sheet **is the D22 gate** — the thing that decides whether a lab may compute at all.

**D32 supersedes D25's floating-point allowance.** A sheet citing D25 and not D32 **told a lab that scalar floating-point geometric GPU work is permitted** — the exact opposite of the rule now in force. **That is not a missing footnote; it is an instruction to do the prohibited thing.**

**D31 governs what happens when an instrument check fails** — a pre-registration states checks, so omitting D31 omits the rule about what those checks may conclude.

## Fixed

Added to the registry's `owner_rules` **and to `APPLICABLE_RULES` in both implementations**, verified by rendering a sheet from each:

```
rust sheet:   - D21 - D22 - D23 - D24 - D25 - D28 - D29 - D31 - D32
python sheet: - D21 - D22 - D23 - D24 - D25 - D28 - D29 - D31 - D32
```

**Two tests pin it:** the set is nine ids, and **every cited rule has registry text** — because a cited rule with no text prints a hole that **reads as "no rule applies" rather than "the registry is missing one."**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 318 passed (2 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
python3 scripts/check_no_softmax.py             -> OK, no softmax added
parity.py                                       -> exit 0
```

Record: `docs/labs/prep-rules-d31-d32-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **This fixes the two rules I found missing, not the mechanism that let them go missing.** `APPLICABLE_RULES` is still a hand-kept tuple in two implementations, **so the next owner rule will be omitted in exactly the same way** until the sheet renders from the registry instead of a constant. **`parity` keeps the two tuples equal; nothing keeps either equal to the registry.**
- **The rule text is my paraphrase** drawn from `DECISIONS.md`, which stays authoritative — **and a paraphrase can drift from its source.**
- **`D30` was already in the registry and D31/D32 were not**, so the registry and `DECISIONS.md` had already diverged for at least one rule boundary.
- **I found this by reading `AGENTS.md` when it changed mid-session, not by any check.** **No rule in `check` compares `APPLICABLE_RULES` against `owner_rules`** — and **that comparison is the durable fix.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

