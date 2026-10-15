# #2419 — lint runs in Rust: the first command to leave the delegated set (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T02:31:59Z
- **Updated:** 2026-10-14T02:36:24Z
- **Closed:** 2026-10-14T02:36:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2419
- **Merged:** 2026-10-14T02:36:05Z
- **Merge commit:** 07e4a13aa14ba0eb597f81a83581bd1a99d8487e
- **Base / Head:** main / UOR-Foundation:deepseek/lint-command
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`lint` runs in Rust — the first command to leave the delegated set.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`uor-harness lint`** — `lint check` and `lint format` — **runs entirely in Rust with no delegation.** Verified: **`lint check` prints zero "not ported to Rust yet" notices**, while **`check` still prints one** — correctly, because `check` is a different command and remains Python.

Everything it needs is now Rust: **`check_pr`** applies the gates, **`format_pr_body`** renders a compliant body, and the D23 scan is the **ratchet** rather than the retired script.

**Byte-identical to Python on 12 comparisons** — six bodies through `lint check` (output *and* exit code) and six through `lint format`:

```
lint check:  6 identical, 0 differing
lint format: 6 identical, 0 differing
```

## Four divergences found by diffing, not by reading

**1. The headline fix dropped the denominator.** It rendered the *value* instead of the whole headline — `headline 5 → <number/of>` where Python gives `headline 5/232 → <number/of>`. **Found only because my test body happened to have a before value with *no* denominator** — invisible until one exists. **The old test asserted `headline 37` and passed while the code was wrong**, so it was strengthened.

**2. `format` is a subcommand, not a flag.** It arrives in the argument list because `lint` claimed the command slot; I looked only for `--action`, so **`lint format` silently executed `check` and printed a summary where Python printed a body.**

**3. An empty `--body ""` is not a body.** Python's check is **falsy** — an empty string falls through to exit 2. Mine linted it, **disagreeing on the one input where "no body" and "an empty body" are the same thing to a reader.**

**4. A doubled trailing newline.** `format_pr_body` already ends with one and `println!` added a second — **a blank line ending every rendered body, invisible in a terminal and not in a diff.** The fix belongs at the print, **because the function's contract is the byte-for-byte fixture.**

**None of the four was visible in the code. Each came from running both implementations and diffing.**

```
cargo test -p uor-r4-harness                    -> 362 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/lint-command-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates, so the objective is not met.** `lint` is one command of the set; **`delegate_to_python` remains live**, and the goal's own condition is **zero notices from `check`.**
- **The six bodies are hand-written**, not real PR bodies — **not the shapes other labs actually write.**
- **`--action` is now accepted and ignored.** `format` is detected as a subcommand word, so `--action format` yields `check`. **Python has no such flag, so nothing invokes it — but leaving it accepted is a small lie in the interface.**
- **`lint format`'s output is asserted against a capture, not the contract.** Where Python's formatter renders something odd, **the port reproduces it** — deliberately, and worth knowing before reading the formatter as an endorsement of the template.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

