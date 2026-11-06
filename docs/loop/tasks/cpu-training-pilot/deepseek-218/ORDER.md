# ORDER: cpu-training-pilot, deepseek-218 (release prompt-prefix claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `76fcbae7c`.

## Goal

Record protected delivery of deepseek-217, verify its fresh-main source and
evidence hashes, and remove the completed `deepseek/prompt-prefix` claim.

## Done when

1. Fresh-main source and evidence hashes match the reviewed head.
2. The prefix-continuation tests pass on fresh main.
3. The claim is removed and this release REPORT merges through the queue.

## Stop rule

Stop on source/evidence drift, test failure or overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-218/ORDER.md
