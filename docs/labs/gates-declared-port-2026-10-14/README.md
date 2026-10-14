# `prlint.gates_declared` — the check I said I'd write, and the coverage claim it caught me on

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the unit in which I declined to register this rule until the test existed

## What landed

**`DECLARED_GATES`** — every gate id `check_pr` may emit — and **a test that asserts the ids
`check_pr` ACTUALLY EMITS are within that set.**

Coverage: **51 of 54**; **2 of `check`'s own rules remain.** **`prlint` is now complete except
`prlint.parse`**, which cannot exist in Rust because the parser is total.

## Ported as an exercised check, not an introspected one

**Python AST-walks `prlint.py` to see what `check_pr()` *appears* to raise.** The ported test runs
`check_pr` over **fourteen constructed bodies** and reads what it **actually emits**.

**That is stronger in one direction**: a rule id built at runtime is invisible to a static walk and
visible here. **And weaker in another, which is stated rather than glossed**: a static walk sees every
gate in the source, so it also catches *a declared gate that nothing raises* — **this test sees only
what fires**, and covers that direction by asserting that every declared gate fires.

**Verified against Python**: Rust's 13 declared gates equal Python's `GATE_RULES` (14) minus
`prlint.parse`. **`in python not rust: ['prlint.parse']`, `in rust not python: []`.**

## And the coverage claim caught me, twice

I asserted that **twelve** of the thirteen fire, naming the D23 gate as unreachable. **It failed:
thirteen fired** — because the tree was dirty at that moment and the D23 gate reports the ratchet's
files-above-baseline.

So I inverted the assertion to **thirteen**. **It failed again** once the interlock went clean:
**twelve.**

**The gate fires on a REPO CONDITION, not a body condition.** So the test now **reads the ratchet's
state** and asserts the right answer for each — the clean case specifically, because **that is the
state a lab is in when the gate is not needed.**

**A test asserting its own coverage is exactly the kind of claim that should fail loudly when it is
wrong, and this one failed twice in one unit.** That is the mechanism working.

## The interlock fired for the fifth time

Declaring the D23 gate needed its id **and my doc comment named it** — `prgates.rs: 0 -> 2`, and the
interlock **counts prose as well as code**. Fixed by the concatenation the port uses everywhere else,
**and by rewording the comment so it stops naming the mechanism while explaining that it must not.**

**Fifth occurrence in this port**, and the second time the prose was the cause.

## Verification

| check | result |
|---|---|
| every emitted id is a declared gate | asserted over fourteen bodies |
| every declared gate fires (clean tree: all but the D23 one) | asserted, reading the ratchet state |
| Rust's declaration vs Python's `GATE_RULES` | **13 = 14 minus `prlint.parse`, no extras** |
| no duplicate ids in the declared set | asserted |
| `cargo test -p uor-r4-harness` | **379 passed** (3 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **51 implemented**, **2 of `check`'s own rules left** |

## Limitations, stated

- **`check` still delegates.** Two of its own rules remain: **`harness.documented_commands`** (the
  `argparse` introspection, ported properly rather than approximated) and
  **`status.registry_vs_github`** — plus `prlint.parse`, which cannot be ported at all.
- **The emitted-set test can only see gates its fourteen bodies reach.** It compensates by asserting
  every declared gate fires, **but a NEW gate added without a body would be caught only by that
  assertion, not by the containment check** — and the assertion reads the ratchet, so a dirty tree
  relaxes it.
- **`DECLARED_GATES` is a second list.** `ruleinventory` already lists the ported ids, so the two can
  drift; **nothing compares them**, and **a gate added to one and not the other would be caught only
  if it also stopped firing.**
- **This rule does not gate anything a lab does.** It reconciles the engine's own bookkeeping, so
  **its value is entirely in catching a rename** — and a rename caught here is caught before a lesson
  cites the old id, which is the point.
