# ORDER: cpu-training-pilot, deepseek-216 (release lexical ranker claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `10977505e`.

## Goal

Verify the protected delivery of the deepseek-215 lexical ranker rejection on
fresh `main`, record the merge receipt and remove the completed coordination
claim.

This is release bookkeeping only.

## Done when

1. Fresh-main source, compact evidence and published checksum hashes match the
   reviewed deepseek-215 head.
2. Fresh-main focused and full CPU tests pass.
3. The `deepseek/prompt-reply-lexical` claim is removed.
4. A release REPORT merges through the protected queue.

## Stop rule

Stop if the reviewed source or evidence differs on fresh `main`, any claimed
test fails, or another lab's claim overlaps this bookkeeping change.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-216/ORDER.md
