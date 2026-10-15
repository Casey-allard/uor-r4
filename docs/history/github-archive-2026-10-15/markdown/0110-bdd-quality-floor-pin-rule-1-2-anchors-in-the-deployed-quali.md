# #110 — BDD quality floor: pin Rule 1+2 anchors in the deployed quality gate

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T15:50:24Z
- **Updated:** 2026-07-23T16:08:27Z
- **Closed:** 2026-07-23T16:08:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/110
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The deployed quality gate (`validate_quality_report`, src/r4g1.rs:618) only checks Rule 1+2 top-1 agreement *relative* to the TLA3 baseline. Nothing pins the absolute quality the #65 chain achieved, so a change that degrades both graph and baseline together (or degrades bits/token, which the gate never inspects) would pass silently. The CI trend alarm (`scripts/check_gate_c_regression.py`) catches this only on the trend workflow path, not at artifact load time, and not as behavior-level BDD.

## Proposal

Enforce the chain's pinned anchors as an absolute floor in the quality gate, using the same margins as the trend alarm so there is one definition of digression:

- Rule 1+2 top-1 agreement ≥ 0.317 − 0.02 = **0.297**
- Rule 1+2 bits/token ≤ 9.86 + 0.10 = **9.96**

Missing metrics remain compatible with older reports (current behavior). Add BDD scenarios covering: accept at the pinned anchors, reject on bits/token digression, reject on top-1 digression.

## Definition of Done

- [ ] `validate_quality_report` rejects reports below the absolute floor (both metrics), with era-noted constants matching `scripts/check_gate_c_regression.py`
- [ ] BDD scenarios added and green under `cargo test --test bdd`
- [ ] Four gates clean (fmt / clippy / tests / κ unaffected); PR merged via merge queue

## Acceptance Criteria

- A report at the chain anchors (31.7086% / 9.8612 bits) passes; a report at 10.5 bits or 25% agreement fails with a floor error.
- `grep -n "0.297\|9.96" src/r4g1.rs scripts/check_gate_c_regression.py` shows both sites consistent.

Refs #65 (final anchors), #77 (trend alarm)


## Comments (1)

### comment 5060671492 — 2026-07-23T16:08:25Z — auser

## Done — DoD evidence

**Merged via merge queue: PR #111**

- [x] `validate_quality_report` rejects reports below the absolute floor: top-1 < 0.297 or bits/token > 9.96, era-noted constants in `src/r4g1.rs` (`QUALITY_FLOOR_TOP1_AGREEMENT`, `QUALITY_FLOOR_BITS_PER_TOKEN`) matching `scripts/check_gate_c_regression.py`
- [x] BDD scenarios green under `cargo test --test bdd` — 16/16, including accept-at-anchors, reject-on-bits-digression, reject-on-top1-digression; the "at TLA baseline" scenario updated (at-baseline 18.11% is now below floor and rejected)
- [x] Four gates clean: `cargo test --workspace --offline`, clippy `-D warnings`, `cargo fmt --check`, κ reproduction (2 passed, non-vacuous)

**Acceptance criteria verified:**
- Report at the #65 anchors (31.7086% / 9.8612 bits) passes; 10.5 bits or 25% agreement fails with a "digresses" floor error (BDD scenarios pin both).
- `grep -n "0.297\|9.96" src/r4g1.rs scripts/check_gate_c_regression.py` — both sites carry the same floor/ceiling numbers (constants written as `0.317 - 0.02` / `9.86 + 0.10` with the pin explicit).

Closing.



---
_Comment/review ids in this file: 5060671492_

