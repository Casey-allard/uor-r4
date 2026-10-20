# `lint` runs in Rust — the first command to leave the delegated set

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the unit that moves the objective's own measure

## What landed

**`uor-harness lint`** — `lint check` and `lint format` — **runs entirely in Rust, with no
delegation.** Verified: **`lint check` prints zero "not ported to Rust yet" notices**, and **`check`
still prints one**, which is correct because `check` is a different command and remains Python.

Everything the command needs is now Rust: **`check_pr`** runs the gates, **`format_pr_body`** renders
a compliant body, and the D23 scan is the **ratchet** rather than the retired script.

**Verified byte-identical to Python on 12 comparisons** — six bodies through `lint check` (output
*and* exit code) and six through `lint format`:

```
lint check:  6 identical, 0 differing
lint format: 6 identical, 0 differing
```

## Four divergences found by diffing the two commands, not by reading either

**1. The headline fix dropped the denominator.** `headline_present` rendered the *value* instead of
the whole headline, so it produced `headline 5 → <number/of>` where Python produces
`headline 5/232 → <number/of>`. **Found only because the body I had been testing with happened to
have a before value with no denominator** — the difference is invisible until a denominator exists.
The old test asserted `headline 37` and **passed while the code was wrong**, so it was strengthened
rather than left.

**2. `format` is a subcommand, not a flag.** `lint format` arrives with `format` in the argument list
because `lint` already claimed the command slot; I looked only for `--action`, so **`lint format`
silently ran `check` and printed a summary where Python printed a body.**

**3. An empty `--body ""` is not a body.** Python's check is **falsy**, so an empty string falls
through to the error and exit 2. Mine treated it as the body and linted it — **the two disagreed on
the one input where "no body" and "an empty body" are the same thing to a reader.**

**4. A doubled trailing newline.** `format_pr_body` already ends with one, and `println!` added a
second — **a blank line at the end of every rendered body, invisible in a terminal and not invisible
in a diff.** The fix belongs at the print, not in the string the function returns, **because the
function's contract is the byte-for-byte fixture.**

**None of these four was found by reading the code.** Each came from running both implementations on
the same input and diffing.

## Verification

| check | result |
|---|---|
| `lint check` vs Python — 6 bodies, output and exit code | **identical** |
| `lint format` vs Python — 6 bodies | **identical** |
| **`lint` delegated notices** | **0** |
| `check` still delegates | 1, correctly |
| `cargo test -p uor-r4-harness` | **362 passed** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **`check` still delegates, so the objective is not met.** `lint` is one command of the set;
  **`delegate_to_python` remains live**, and the goal's own condition is zero notices from `check`.
- **The six bodies are hand-written**, not real PR bodies. They cover empty, minimal, two decisions,
  a registry line at three, and prose-only — **not the shapes other labs actually write.**
- **`--action` is now accepted and ignored.** `format` is detected as a subcommand word, so a caller
  passing `--action format` gets `check` — **the flag is registered for the dispatcher's sake and does
  nothing.** Python has no such flag, so nothing invokes it, and **leaving it accepted is a small lie
  in the interface.**
- **`lint format`'s output is asserted byte-for-byte against a capture, not against the contract.**
  Where Python's formatter renders something odd, **the port reproduces it** — deliberately, and it
  is worth knowing before reading the formatter as an endorsement of the template.
