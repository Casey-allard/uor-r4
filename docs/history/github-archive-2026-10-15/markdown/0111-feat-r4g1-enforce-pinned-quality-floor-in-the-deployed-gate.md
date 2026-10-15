# #111 — feat(r4g1): enforce pinned quality floor in the deployed gate (#110)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T15:57:32Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-07-23T16:08:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/111
- **Merged:** 2026-07-23T16:08:03Z
- **Merge commit:** 7b051d668df1371279f7f579bb51439210db0cfa
- **Base / Head:** main / UOR-Foundation:issue-110-quality-floor-bdd
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #110

## What

The deployed quality gate (`validate_quality_report`) previously enforced only a *relative* check: Rule 1+2 top-1 ≥ TLA3 baseline. A change that degraded graph and baseline together — or degraded bits/token, which was never inspected — would pass silently. This PR pins the absolute quality floor the #65 chain achieved, enforced at artifact load time and covered by BDD:

- Rule 1+2 top-1 agreement ≥ **0.297** (pin 0.317 − 0.02)
- Rule 1+2 bits/token ≤ **9.96** (pin 9.86 + 0.10)

Constants carry an era note tying them to `scripts/check_gate_c_regression.py` (same numbers, one definition of digression); both sites move together when a compiler redesign legitimately shifts the anchors. Missing metrics remain compatible with older reports, as before.

## BDD coverage (features/suites/r4g1_compile_quality.feature)

- accept a graph at the pinned anchors (31.71% / 9.8612 bits)
- reject on bits/token digression (10.5 bits; relative check still passes, so only the floor can fire)
- reject on top-1 digression (25%; same isolation)
- the pre-existing "accept a graph at the TLA baseline" scenario is updated: at-baseline quality (18.11%) is now *below the floor* and must be rejected — that expectation change is the point of the gate, and the relative-baseline reject scenario remains for the teacher-side regression case

## Verification

- `cargo test --test bdd`: 16/16 scenarios pass
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
- `cargo fmt --check`: clean
- `cargo test --workspace --offline`: green
- κ reproduction: unaffected (gate-side change, not compiler), verified

Refs #65 (anchors), #77 (trend alarm)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

