# #2441 — Wire three rules that were registered and never ran (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T04:52:32Z
- **Updated:** 2026-10-14T04:54:58Z
- **Closed:** 2026-10-14T04:54:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2441
- **Merged:** 2026-10-14T04:54:36Z
- **Merge commit:** 91b93a32ae3c1c3e7f95f8ea66dc3d61bbbf2427
- **Base / Head:** main / UOR-Foundation:deepseek/dispatch-fixes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Three rules that were registered and never ran.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**Three rules that were registered, counted and never dispatched are now wired**, and the dispatcher's verdict **matches Python's exactly**:

| | before | after |
|---|---|---|
| Rust failing rules | 8 | **9** |
| Python failing rules | 9 | 9 |
| **rust-only / python-only** | 1 / 3 | **0 / 0** |

## The three, each a different kind of missing

**1. `budget.no_new_spending_class` — implemented, tested, never called.** The function existed with its own tests; **`budgetrules::all()` did not include it**, so the dispatcher synthesised an `ok` record for it every run.

**2. `harness.state_formats` — implemented as part of the contract, never as a rule.** `check` ran the format contract separately, so **the rule id appeared in the table and the output with nothing behind it.** The ported rule now calls the checker directly — **removing Python's subprocess and its `PROBLEM`-line string parsing**, so the two are the same check without the round trip.

**3. `status.heartbeat_receipts` — registered with NO IMPLEMENTATION AT ALL.** In `ruleinventory`, in the dispatch table, in the coverage count — **and no Rust function anywhere.** Ported here, with `status.registry_vs_github`, which had been ported two units earlier and never wired.

## Why only a comparison could find this

**A rule that does not exist and a rule that passes are identical in the output.** Both produce no finding, both print an `ok` record, the count is the same. **Only running both implementations and diffing which rules fail can see it** — which is what the previous unit's dispatcher made possible.

**Third, fourth and fifth instance of "registered but not running" in three units** — after `harness.owner_rules_applied` and after `cmd_check` itself.

**And `status.heartbeat_receipts` is the rule that would have caught my own stale presence.** It requires a heartbeat within 24h for every in-progress milestone — **and it had never run in Rust.**

```
cargo test -p uor-r4-harness                    -> 391 passed, 4 ignored
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/dispatch-divergences-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **Rust fails one rule Python does not: the D23 gate** — correct and deliberate, since `checkrules.rules()` does not run the PR-body gates. **A reader comparing totals sees 9 and 9 and 49 and 57 and concludes they agree — and two of those three numbers mean different things.**
- **`heartbeat_receipts` reads this month and the previous one unconditionally**, where Python reads the previous month **only when this month has fewer than 5 rows**. On a quiet month boundary the port reads more, **which can only make it fire less often.**
- **The rule fires on `status == "in-progress"` only**, so **a milestone marked in-progress by a typo is checked and one that should be and is not, is not** — it trusts the registry about which milestones matter, and that trust is unverified.
- **`registry_vs_github` is called with `refresh=false`**, so it reads the cache and reports *"unavailable"* rather than fetching. **A `check` run cannot warm its own cache.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

