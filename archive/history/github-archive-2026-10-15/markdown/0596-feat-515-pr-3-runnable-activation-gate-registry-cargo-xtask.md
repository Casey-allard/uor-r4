# #596 — feat(#515): PR-3 — runnable activation-gate registry (cargo xtask gate / gates)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T21:52:11Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-10T22:11:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/596
- **Merged:** 2026-08-10T22:11:12Z
- **Merge commit:** fc113dd4135c85c01cde06ef6243e4170518a53d
- **Base / Head:** main / UOR-Foundation:r5/xtask-gate-registry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-3 — runnable activation-gate registry

The #515 register keeps every dormant / ruled-out mechanism in `model/ledger.toml` as an `open` claim, and each open claim names an **activation gate**: the pre-declared measurement that would promote the mechanism back onto the serving path. This turns that prose register into a runnable one.

- `cargo xtask gates` — lists every open claim, extracts its activation-gate clause, classifies the measurement it demands (**mechanism-comparison** vs **metric-adoption**), and reports status.
- `cargo xtask gate <claim-id>` — the same for one claim; errors on an unknown id.

It reads the ledger via `repo_model::Model`, so it always reflects the committed claims — no hardcoded list. Every open claim reports **NOT CLEARED (dormant)**: an open claim is measured and reported, never asserted (R2), and the moment a gate clears the mechanism is activated and the claim leaves `open`. The value is that the command prints *exactly what would clear each gate*, in one place, re-runnable.

**Extension point:** per-claim measurement runners attach here keyed by claim id — as a mechanism's harness is wired against the Gate C fixtures (`crates/uor-r4-core/tests/fixtures`), the command begins reporting a live `CLEARED` / `NOT CLEARED` verdict for it instead of the declared-bar reminder.

Wired into `xtask/src/main.rs` beside `check-model` / `audit-deferral` / `audit-limits`. Current register: 21 open claims (12 mechanism-comparison, 9 metric-adoption). 4 unit tests cover clause extraction + classification.

Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

