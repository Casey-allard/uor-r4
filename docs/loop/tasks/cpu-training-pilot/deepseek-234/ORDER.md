# ORDER: cpu-training-pilot, deepseek-234 (release calibrated-branch claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `672fb6eea`.

## Goal

Record protected delivery of deepseek-233 and verify fresh-main source and
evidence. The manual claim path did not create a `docs/harness/claims.json`
entry, so there is no committed claim record to remove here.

## Done when

The reviewed source and evidence match fresh main, focused tests pass, the
merged remote implementation branch is deleted only after GitHub reports the
PR merged, and this REPORT merges through the protected queue.

## Stop rule

Stop on drift, test failure, a missing merge record, or an overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-234/ORDER.md
