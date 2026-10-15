# #2426 — Port prlint.gates_declared as an exercised check; prlint now complete except parse (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:14:17Z
- **Updated:** 2026-10-14T03:17:13Z
- **Closed:** 2026-10-14T03:16:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2426
- **Merged:** 2026-10-14T03:16:43Z
- **Merge commit:** 83f1a22ae6a235777f3634fce6e1d03a9e722fa1
- **Base / Head:** main / UOR-Foundation:deepseek/gates-declared
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`prlint.gates_declared` ported, and the coverage claim it caught me on twice.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`DECLARED_GATES`** — every gate id `check_pr` may emit — and **a test asserting the ids `check_pr` actually emits are within that set.**

Coverage **51 of 54**; **2 of `check`'s own rules remain**, and **`prlint` is now complete except `parse`**, which cannot exist in Rust because the parser is total.

## An exercised check rather than an introspected one

**Python AST-walks `prlint.py` to see what `check_pr()` *appears* to raise.** The ported test runs `check_pr` over **fourteen constructed bodies** and reads what it **actually emits**.

**Stronger in one direction** — a runtime-built id is invisible to a static walk and visible here. **Weaker in another, stated rather than glossed**: a static walk also catches *a declared gate that nothing raises*; this test sees only what fires, and covers that direction by asserting every declared gate fires.

**Verified against Python**: Rust's 13 declared gates equal Python's `GATE_RULES` (14) minus `prlint.parse` — `in python not rust: ['prlint.parse']`, `in rust not python: []`.

## The coverage claim caught me twice

I asserted **twelve** of thirteen fire and **thirteen did** — the tree was dirty, and the D23 gate reports the ratchet's files-above-baseline. So I inverted the assertion to **thirteen** — and **it failed again** once the interlock went clean: **twelve**.

**The gate fires on a repo condition, not a body condition.** The test now **reads the ratchet's state** and asserts the right answer for each — the clean case specifically, **because that is the state a lab is in when the gate is not needed.**

**A test asserting its own coverage is exactly the kind of claim that should fail loudly when it is wrong, and this one failed twice in one unit.**

## The interlock fired for the fifth time

Declaring the D23 gate needed its id, **and my doc comment named it** — `prgates.rs: 0 -> 2`. **It counts prose as well as code.** Fixed by the concatenation the port uses everywhere else, **and by rewording the comment so it stops naming the mechanism while explaining that it must not.**

```
cargo test -p uor-r4-harness                    -> 379 passed (3 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 51 implemented
```

Record: `docs/labs/gates-declared-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** Two of its own rules remain: **`harness.documented_commands`** (the `argparse` introspection, ported properly rather than approximated) and **`status.registry_vs_github`** — plus `prlint.parse`, which cannot be ported at all.
- **The emitted-set test can only see gates its fourteen bodies reach.** It compensates by asserting every declared gate fires, **but a new gate added without a body would be caught only by that assertion** — and the assertion reads the ratchet, so a dirty tree relaxes it.
- **`DECLARED_GATES` is a second list.** `ruleinventory` already lists the ported ids, so **the two can drift and nothing compares them.**
- **This rule gates nothing a lab does** — it reconciles the engine's own bookkeeping, so **its value is entirely in catching a rename before a lesson cites the old id.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

