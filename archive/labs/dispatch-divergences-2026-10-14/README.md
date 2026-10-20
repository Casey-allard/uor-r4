# Three rules that were registered and never ran

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the divergences the `check` dispatcher revealed when it was first compared to Python

## What landed

**Three rules that were registered, counted, and never dispatched are now wired**, and the
dispatcher's verdict **matches Python's exactly**:

| | before | after |
|---|---|---|
| Rust failing rules | 8 | **9** |
| Python failing rules | 9 | 9 |
| **rust-only / python-only** | 1 / 3 | **0 / 0** |

## The three, and each was a different kind of missing

**1. `budget.no_new_spending_class` — implemented, tested, never called.**
The function existed and had its own tests; **`budgetrules::all()` did not include it**, so the
dispatcher synthesised an `ok` record for it on every run.

**2. `harness.state_formats` — implemented as part of the contract, never as a rule.**
`check` ran the format contract separately, so **the rule id appeared in the table and in the output
with nothing behind it.** The ported rule now calls the checker directly, **which removes Python's
subprocess and its `PROBLEM`-line string parsing** — the two are the same check without the round
trip, and this one cannot disagree with itself.

**3. `status.heartbeat_receipts` — registered with NO IMPLEMENTATION AT ALL.**
It was in `ruleinventory`, in the dispatch table, and in the coverage count — **and there was no Rust
function for it anywhere.** Ported here, together with `status.registry_vs_github`, which had been
ported two units earlier and never wired either.

## Why only a comparison could find this

**A rule that does not exist and a rule that passes are identical in the output.** Both produce no
finding; the dispatcher prints an `ok` record; the count is the same. **The only way to see it is to
run both implementations and diff which rules fail** — which is what the previous unit's dispatcher
made possible.

**This is the third, fourth and fifth instance of "registered but not running" in three units** —
after `harness.owner_rules_applied`, and after `cmd_check` itself.

## What the dispatcher now checks, and what that says

**`status.heartbeat_receipts` is the rule that would have caught my own stale presence.** It requires a
heartbeat within 24h for every in-progress milestone, **and it had never run in Rust.** A lab reading
`uor-harness check` saw it as a pass.

## Verification

| check | result |
|---|---|
| the dispatcher's failing set vs Python's (`checkrules.run`) | **9 = 9, zero either way** |
| `cargo test -p uor-r4-harness` | **391 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0 |

## Limitations, stated

- **Rust fails one rule Python does not: the D23 gate.** That is correct and deliberate —
  `checkrules.rules()` does not run the PR-body gates, so Python's `check` never evaluates it.
  **A reader comparing the two totals will see 9 and 9 and 49 and 57 and conclude they agree**, and
  two of those three numbers mean different things.
- **`heartbeat_receipts` reads this month and the previous one, unconditionally**, where Python reads
  the previous month **only when this month has fewer than 5 rows**. **On a quiet month boundary the
  port reads more, which can only make it fire LESS often — a heartbeat in the previous month
  satisfies it where Python might already have reported one missing.**
- **The rule fires on `status == "in-progress"` only.** A milestone marked in-progress by a typo in
  the registry is checked; one that should be and is not, is not — **the rule trusts the registry
  about which milestones matter, and that trust is unverified.**
- **`registry_vs_github` is called with `refresh=false`**, so it reads the cache and reports
  *"unavailable"* rather than fetching. **A `check` run cannot warm its own cache**, which means the
  rule's verdict depends on whether some earlier `status --refresh` ran.
