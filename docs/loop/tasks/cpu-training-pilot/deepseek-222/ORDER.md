# ORDER: cpu-training-pilot, deepseek-222 (release nearest-rival claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b594be94d`.

## Goal

Record protected delivery of deepseek-221, verify its fresh-main source and
sealed evidence, and remove the completed `deepseek/nearest-rival-gain` claim.

## Done when

1. Fresh-main source, driver and evidence hashes match the reviewed head.
2. The nearest-rival library and driver tests pass on fresh main.
3. The claim is removed and this release REPORT merges through the queue.

## Stop rule

Stop on source/evidence drift, test failure or overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-222/ORDER.md
