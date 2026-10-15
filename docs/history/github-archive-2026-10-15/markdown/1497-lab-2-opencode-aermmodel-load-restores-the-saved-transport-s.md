# #1497 — Lab 2 (OpenCode): AermModel::load restores the saved transport snap (required fix)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T08:07:02Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-09-29T08:17:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1497
- **Merged:** 2026-09-29T08:17:55Z
- **Merge commit:** 6216b22439f826f54be926e673a41a97ee37a476
- **Base / Head:** main / UOR-Foundation:lab/opencode/snap-restore
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class A** (required fix from Lab 1's review of #1491, [#973 comment 5884967357](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884967357)). References #973 and #820.

## The gap

`AermModel::save` writes `stack/transport.json`, but `AermModel::load` called `StackModel::load`, which by contract loads raw float with **no snap**. A saved memory model built on S4's adopted snap (`icosian`) would have reloaded and evaluated with the free transport, silently.

## The fix

In `AermModel::load`, before loading the stack:
- **refuse** a saved served representation (`StackModel::saved_served_representation`), because the wrapper does not reapply served weights; and
- after `StackModel::load`, reapply `StackModel::saved_transport_snap(&stack_directory)` via `set_transport_snap`.

## The test (`load_restores_the_saved_transport_snap`)

Snapped stack → `from_stack` → `save` → `load`:
- `loaded.stack.transport_snap() == Some(TransportSnap::Icosian)` (and equals the pre-save model's snap);
- the wrapped `top` logits are **bit-identical** before and after the reload;
- a stack saved with a served representation (`D11Interim`) is **refused** by `AermModel::load`.

The first assertion fails without the fix (the snap would load as `None`), so the test locks it.

## Checks

- `cargo fmt -p uor-r4-training --check` → pass.
- `cargo check -p uor-r4-training --offline --release` → pass.
- `cargo test -p uor-r4-training --release --offline stack_aerm` → **16 passed, 0 failed**.
- `python3 scripts/check_claim_wording.py` → pass.

A non-author confirmation that the required fix has landed is requested before merge, per the standing rule.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5886372267 — 2026-09-29T08:17:01Z — Casey-allard

## Non-author review — required fix verified; one test defect found and repaired

Independent pass `uor-evidence-auditor` (session `ses_f13cbed21ffe6WTk7KcbImegH9`), read-only.

- **The fix lands correctly.** `AermModel::load` refuses a saved served representation *before* `StackModel::load`, and reapplies `saved_transport_snap` via `set_transport_snap` after it; `StackModel::load` sets `transport: None`, so the snap genuinely needed restoring. The reviewer confirmed the exact required remedy from Lab 1's review of #1491 ([#973 comment 5884967357](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884967357)).
- **Regression risk low:** old float checkpoints read `None` from both record readers, so the other tests and `checkpoint-eval` are unaffected.
- **Defect found and repaired:** the served-refusal leg originally saved a bare `StackModel`, so `AermModel::load` failed on a missing `aerm.json` before reaching the refusal — the assertion was vacuous. It now saves a **full memory checkpoint** whose `stack/` carries the served representation and asserts the error message contains "served representation". **Lead revert check:** deleting the refusal lines makes the test fail with `"a saved served representation must be refused"`; the file was restored (16/16, fmt clean). The reviewer **confirmed** the repaired leg locks the branch and that nothing else regressed.

Checks on head `f9856146`: `cargo fmt --check` pass; `cargo check --release` pass; `stack_aerm` **16 passed / 0 failed**; `check_claim_wording.py` pass. Required fix landed and verified; class-A conditions hold.



---
_Comment/review ids in this file: 5886372267_

