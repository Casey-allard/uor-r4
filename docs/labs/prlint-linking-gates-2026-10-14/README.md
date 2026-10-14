# `checks_at_head` and `org_rules`: thirteen of fourteen gates, and why the fourteenth cannot exist

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `lint`/`prlint` port

## What landed

| gate | severity | condition |
|---|---|---|
| `prlint.checks_at_head` | warn | the exact-head-checks box is **not ticked** and the body **lists no commands** |
| `prlint.org_rules` | info | no `References #N` or `Closes #N` line |

Coverage: **45 of 54** — **13 of `prlint`'s 14 gates in Rust.**

## `checks_at_head` accepts the weaker of two evidences, deliberately

**Either a pasted command or a ticked box satisfies it.** A pasted command is evidence unticked; a
ticked box is a claim. **The gate asks whether a reader can see *something*, and both answers are
visible** — so accepting the weaker is the design rather than an oversight, and it is asserted.

**An unticked box is not evidence, and neither is a ticked unrelated item**, because the match is on
`exact head` or `check(s) run`. Both asserted.

## And the fourteenth gate is unrepresentable, not forgotten

**`prlint.parse` fires only when `parse_pr_body` raises:**

```python
try:
    fields = parse_pr_body(body)
except Exception as exc:
    return {"ok": False, "problems": [Problem("prlint.parse", "error", ...)]}
```

**The Rust parser returns no `Result`** — it is total, and a malformed body yields empty fields.
**So there is no state in which this gate could fire**, and writing it would mean a function nothing
can reach or a `Result` added to the parser solely to have somewhere to put the error.

**That is a deliberate divergence, and it is the safe direction.** Python reports *"body could not be
parsed"* and **skips every gate**; Rust parses what it can and **runs every gate on the result.** A
body Python would refuse outright is one Rust checks normally.

**Recorded in the module docstring rather than left as a silent gap** — because **a gate that is
missing and a gate that cannot exist look identical from a coverage report**, and `parity` reports
`prlint.parse` as still in Python, **which is true and is not a port that was forgotten.**

## Verification

| check | result |
|---|---|
| both gates' messages and severities against Python | asserted |
| commands **or** a ticked box each satisfy the check gate | asserted |
| an unticked or unrelated box does not | asserted |
| `Closes` links the PR as `References` does | asserted |
| a complete body raises neither | asserted |
| `cargo test -p uor-r4-harness` | **356 passed** (5 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **45 implemented** |

## Limitations, stated

- **`check_pr` is still not ported, and this is the thing that matters most.** **Thirteen gates are
  now Rust and NOT ONE is reachable from any command** — `lint` still delegates, so the port has
  verified units and no working feature. **The next unit is `check_pr` itself, and until it lands the
  coverage number describes code nobody can run.**
- **`ok`/`summary` are not ported either.** `check_pr` returns them alongside `problems`, and
  **`ok` is what decides a non-zero exit** — so the two ported fields are not sufficient to replace
  the command even now.
- **The `parse` divergence means the two implementations differ on a malformed body**, deliberately
  and in Rust's favour. **`parity` cannot compare that case**, because it cannot make Python's parser
  raise on demand.
- **`checks_at_head`'s box match is a substring test on a lowercased label.** A label like *"checks
  run at this exact head (pending)"*, ticked, satisfies it — **which is the same looseness Python
  has, and the reason the gate can be satisfied by a claim rather than evidence.**
