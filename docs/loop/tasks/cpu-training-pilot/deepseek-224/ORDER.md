# ORDER: cpu-training-pilot, deepseek-224 (release content-length claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `ed0295ba1`.

## Goal

Record protected delivery of deepseek-223, verify its fresh-main source and
sealed evidence, and remove the completed `deepseek/content-length` claim.

## Done when

1. Fresh-main source, driver and evidence hashes match the reviewed head.
2. The stored-length and driver tests pass on fresh main.
3. The claim is removed and this release REPORT merges through the queue.

## Stop rule

Stop on source or evidence drift, test failure or overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-224/ORDER.md
