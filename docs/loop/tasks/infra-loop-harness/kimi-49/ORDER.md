# ORDER: infra-loop-harness, kimi-49

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `340ec841a`. Setup row: 2 (item-file maintenance).

## Goal

Repair `m1-fresh-panel-transfer.md` so its older prose agrees with the current ready state and the registered fresh-panel sheet. Do not change its status, bar, threshold, dependencies, scope or live issue state.

## Cancel probe (run first)

The item says no model has passed bars (a)–(d) and that the fresh size/noise/threshold is unregistered, while its next paragraph and `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` say Steps 1–3 and 5 are complete and Step 4 is ready. The sheet registers 14,479 eligible targets, 3,594 P_succ rows and paired thresholds. This is contradictory item text, not missing mechanism work. `classification: instrument`. **Not cancelled: correct the prose only.**

## Files in scope

- `docs/roadmap/items/m1-fresh-panel-transfer.md`
- `docs/loop/tasks/infra-loop-harness/kimi-49/**`

## Done when

- The item no longer says nothing exists to score or that the fresh size/noise/threshold is unregistered. It points to the existing passing model and registered sheet without changing the done-when bar.
- `roadmap check` passes and the diff contains only the intended item prose. The generated README is refreshed separately after merge, not edited in this item-input PR.
- REPORT, full gates/local policy, fresh exact-head review, queue and fresh-main verification are complete.

## Stop and decide if

- Any bar, threshold, scope, status or dependency needs changing; that requires a separate decision unit.
- The prep sheet itself needs a status/prose update, or closed-#2820 needs reconciliation. Those are separate units, not incidental edits here.
- The fresh panel would be opened or any GPU run started. This unit is host documentation only.
