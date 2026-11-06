# ORDER: cpu-training-pilot, deepseek-228 (release branch endpoint claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `5f19e0d82`.

## Goal

Record protected delivery of deepseek-227, verify fresh-main source and sealed
evidence, and remove the completed `deepseek/content-branch-endpoint` claim.

## Done when

1. Fresh-main source and evidence hashes match the reviewed head.
2. The branch and driver tests pass on fresh main.
3. The claim is removed and this release REPORT merges through the queue.

## Stop rule

Stop on source or evidence drift, test failure or overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-228/ORDER.md
