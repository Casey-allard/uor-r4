# #2410 — pivot_at_three and record_path ported; coverage 40 of 54 (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:38:30Z
- **Updated:** 2026-10-14T01:42:22Z
- **Closed:** 2026-10-14T01:41:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2410
- **Merged:** 2026-10-14T01:41:59Z
- **Merge commit:** 797d89d9ca51e13e305e85816ba04fc7620711b1
- **Base / Head:** main / UOR-Foundation:deepseek/pivot-record-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`pivot_at_three` and `record_path` — the first gate that stats the filesystem.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

| gate | severity | condition |
|---|---|---|
| `prlint.pivot_at_three` | error | the line is at 3/3 **and** no pivot is announced |
| `prlint.record_path` | warn | no dated record path **named**, or a named one that **does not exist** |

Coverage: **40 of 54**, up from 38.

## The details worth their tests

**`pivot_at_three` has two branches, and the `where` clause says which.** Either side can put the line at three — the declared count or the registry's — and the two messages send a reader to **different places**:

```
line 'starling read' is at registry count 3/3 and the body announces no pivot
line 'exact addressed memory' is at 3/3 declared and the body announces no pivot
```

**A line at three is not a failure; it is where a decision is owed**, so the fix offers both ways out. **Announcing a pivot silences it** — the gate asks for a decision, not for the line to stop.

**`record_path`'s second branch is the one worth having**: a PR naming a record it never wrote. **An existing record is asserted silent** — the property that keeps the gate usable rather than something labs route around.

**The fix text names *today*, which is why the date is a parameter** rather than read inside the gate: **a gate that reads the clock cannot be tested at a fixed instant** — the defect this session already fixed once in `parity`.

```
cargo test -p uor-r4-harness                    -> 345 passed (5 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 40 implemented
```

Record: `docs/labs/prlint-pivot-record-gates-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check_pr` is still not ported. Seven of fourteen gates remain**, plus the dispatcher and the `lint` command. **`lint` still delegates, and none of the eight ported gates is reachable from any command** — verified units, not a working feature.
- **`record_path` uses `Path::exists`, which returns `false` on a permission error.** Python's `Path.exists()` does the same, so they agree — **but a permission-denied record is reported as missing by both**, a wrong finding rather than a missing one.
- **`pivot_at_three` reads `has_pivot`, a regex for `pivot (card|recorded|posted|point|line)`.** **A lab announcing a pivot in wording outside that list is reported as announcing none** — inherited, and not covered by the fixtures.
- **The `today` parameter is passed to `pivot_at_three` and ignored.** It is in the signature for the shared entry point; **an unused parameter is noise, and removing it would mean splitting the entry point.** Recorded rather than hidden.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

