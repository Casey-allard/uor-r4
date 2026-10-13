# The `DIRECTION:` declaration form is now read — closing the gap I flagged when porting `briefing`

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the stated gap in `docs/labs/port-check-briefing-2026-10-13/README.md`

## The gap

When I ported the `briefing` rules I recorded this limit in plain terms:

> **`declared_lines` reads the session-goal file only, and only the `Line "<name>"` form. The Python
> also matches a second form (a `DIRECTION:` prefix on a lab prompt) which this port does not
> implement — so a line declared only in a `DIRECTION:` block would be invisible to the Rust rule
> while Python sees it.**

**That gap is now closed.**

## Two differences, and the second is the one that mattered

**1. The `DIRECTION:` form.** Python matches `DIRECTION:[^\n]*?` followed by a declaration, and
**labels the finding** `"lab prompt DIRECTION"`. Rust now does both — **the label matters because it is
what a finding cites**, and a line found via a lab prompt should say so rather than pointing the reader
at the contract.

**2. `Line` is matched ANYWHERE, not at a word start — and this was the actual defect.** My parser
required a word boundary before `Line`, which Python's regex does not. **So my matcher was strictly
narrower than Python's in a way that could only ever find *fewer* declarations** — and **fewer
declarations reads as agreement.** That is the same shape as every silent-guard failure in this port:
the check reports clean because it looked at less.

**A `DIRECTION:` declaration would have been missed twice over** — once by the form, and again by the
boundary rule, since `DIRECTION: ... Line "x"` has a non-boundary `Line`.

## Measured against the live file

```
declared lines: 1
  "session-goal contract" evaluation integrity 1/3
lines_agree: 0 finding(s)          (Python: all pass)
```

**One declaration, correctly labelled, and the counts agree.** The `DIRECTION:` form has **no
instances in the live file today**, which is exactly why the gap was latent: **both implementations
agreed because they agreed on zero.** A latent gap that agrees is the most dangerous kind — it looks
like a passing check.

## Verification

| check | result |
|---|---|
| Rust vs Python, `lines_agree_with_prompts` | **0 findings; Python all pass** |
| the live declaration parses with its label | **yes** |
| a `DIRECTION:`-form declaration is read and labelled | asserted by test |
| a declaration after other prose on one line is read | asserted by test |
| a plain declaration is labelled as the contract form | asserted by test |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **318 passed** (3 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 322 passed |
| `parity.py` | exit 0 |

## Limitations, stated

- **The gap was latent and this is the fix, not a behavioural change on today's file.** With no
  `DIRECTION:` instances, **the two implementations produced identical results before and after** —
  so the tests are the only evidence the fix works, not a count that moved.
- **The `where_` label is inferred from the line prefix**, not from matching Python's regex structure.
  A declaration whose `DIRECTION:` sits on a *previous* line would be labelled as the contract form.
  Python's `[^\n]*?` is same-line, so the behaviours agree — **but by two different arguments rather
  than by construction.**
- **`declared_lines` still matches `Line` inside any text**, including prose that mentions the word.
  Python has the same looseness, and **the `count n/m` requirement is what keeps it from firing on
  prose** — a sentence containing both `Line` and a `n/m` figure would be read as a declaration by
  both.
- **`check` still delegates.** This port is rule-level, not command-level.
