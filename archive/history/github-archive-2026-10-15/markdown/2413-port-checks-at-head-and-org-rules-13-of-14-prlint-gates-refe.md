# #2413 — Port checks_at_head and org_rules: 13 of 14 prlint gates (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:57:10Z
- **Updated:** 2026-10-14T01:59:41Z
- **Closed:** 2026-10-14T01:59:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2413
- **Merged:** 2026-10-14T01:59:12Z
- **Merge commit:** bcd2d8192545c78c43c0152db4ad27e9d549157a
- **Base / Head:** main / UOR-Foundation:deepseek/final-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`checks_at_head` and `org_rules`: 13 of 14 prlint gates, and why the 14th cannot exist.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

| gate | severity | condition |
|---|---|---|
| `prlint.checks_at_head` | warn | the exact-head-checks box is **not ticked** and the body **lists no commands** |
| `prlint.org_rules` | info | no `References #N` or `Closes #N` line |

Coverage: **45 of 54** — **13 of `prlint`'s 14 gates in Rust.**

## `checks_at_head` accepts the weaker of two evidences, deliberately

**Either a pasted command or a ticked box satisfies it.** A pasted command is evidence unticked; a ticked box is a claim. **The gate asks whether a reader can see *something*, and both answers are visible.** Asserted, along with the two negatives: **an unticked box is not evidence, and neither is a ticked unrelated item.**

## And the fourteenth gate is unrepresentable, not forgotten

**`prlint.parse` fires only when `parse_pr_body` raises.** **The Rust parser returns no `Result`** — it is total, and a malformed body yields empty fields. **So there is no state in which this gate could fire**, and writing it would mean a function nothing can reach or a `Result` added solely to have somewhere to put the error.

**That divergence is deliberate and in the safe direction**: Python reports *"body could not be parsed"* and **skips every gate**; Rust parses what it can and **runs every gate on the result.**

**Documented in the module docstring rather than left as a silent gap**, because **a gate that is missing and a gate that cannot exist look identical from a coverage report.**

```
cargo test -p uor-r4-harness                    -> 356 passed (5 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 45 implemented
```

Record: `docs/labs/prlint-linking-gates-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check_pr` is still not ported, and this is what matters most. Thirteen gates are now Rust and not one is reachable from any command** — `lint` still delegates, so the port has **verified units and no working feature.** **Until `check_pr` lands, the coverage number describes code nobody can run.**
- **`ok`/`summary` are not ported either.** `check_pr` returns them alongside `problems`, and **`ok` decides the exit status** — so the ported fields are not sufficient to replace the command even now.
- **The `parse` divergence means the two implementations differ on a malformed body**, deliberately and in Rust's favour. **`parity` cannot compare that case**, because it cannot make Python's parser raise on demand.
- **`checks_at_head`'s box match is a substring test on a lowercased label.** A ticked *"checks run at this exact head (pending)"* satisfies it — **the same looseness Python has, and the reason the gate can be satisfied by a claim rather than evidence.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

