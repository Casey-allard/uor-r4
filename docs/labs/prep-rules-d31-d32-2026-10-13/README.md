# The pre-registration sheet was omitting D31 and D32

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the defect class found as Antigravity's Defect 6, recurring one rule later

## What was wrong

`AGENTS.md` gained a **D32** block during this session, and I noticed while reporting: **the `prep`
rule text still named D21–D29.** Verified:

```
registry owner_rules: D21 D22 D23 D24 OWNER-* D25 OWNER-* D28 D29
D31 present: False      D32 present: False
APPLICABLE_RULES: ("D21","D22","D23","D24","D25","D28","D29")
```

**Both D31 and D32 had been issued and neither was on the sheet a lab receives.**

## Why this is not a cosmetic gap

**This is the defect that put a banned grader on a sheet, one rule later.** The rule text a lab reads
**is the rule text it follows**, and a `prep` sheet is the D22 gate — the thing that decides whether a
lab may compute at all.

**D32 supersedes D25's floating-point allowance.** A sheet citing D25 and not D32 would tell a lab that
**scalar floating-point geometric GPU work is permitted** — the exact opposite of the rule now in
force. That is not a missing footnote; **it is an instruction to do the prohibited thing.**

**D31 governs what happens when an instrument check fails** — a pre-registration states checks, so a
sheet that omits D31 omits the rule about what those checks may conclude.

## Fixed

**D31 and D32 added to `docs/harness/milestones.json`'s `owner_rules`**, with text drawn from
`DECISIONS.md`, **and to `APPLICABLE_RULES` in both implementations** — Python and Rust. Both now
render nine rule ids on a sheet, verified by rendering one from each:

```
rust sheet:   - D21 - D22 - D23 - D24 - D25 - D28 - D29 - D31 - D32
python sheet: - D21 - D22 - D23 - D24 - D25 - D28 - D29 - D31 - D32
```

**Two tests pin it:** the set is nine ids, and **every cited rule has text in the registry** — because
a cited rule with no text prints a hole that **reads as "no rule applies" rather than "the registry is
missing one."**

## Verification

| check | result |
|---|---|
| Rust sheet cites D31 and D32 | **yes** |
| Python sheet cites D31 and D32 | **yes** |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **318 passed** (2 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | exit 0 |

## Limitations, stated

- **This fixes the two rules I found missing, not the mechanism that let them go missing.**
  `APPLICABLE_RULES` is still a hand-kept tuple in two implementations, **so the next owner rule will
  be omitted in exactly the same way** until the sheet renders from the registry instead of a
  constant. **`parity` keeps the two tuples equal; nothing keeps either equal to the registry.**
- **The rule text is my paraphrase**, drawn from `DECISIONS.md`. The decisions file stays
  authoritative, and **a paraphrase can drift from its source.**
- **`D30` was already in the registry and D31/D32 were not**, so the registry and `DECISIONS.md` had
  diverged for at least one rule boundary before this.
- **I found this by reading `AGENTS.md` when it changed mid-session**, not by any check. **No rule in
  `check` compares `APPLICABLE_RULES` against `owner_rules`** — and that comparison is the durable fix.
