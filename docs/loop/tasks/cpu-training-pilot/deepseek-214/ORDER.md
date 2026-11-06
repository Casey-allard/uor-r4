# ORDER: cpu-training-pilot, deepseek-214 (release reply-ranker claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `e40abe7b9`.

## Goal

Verify the protected delivery of the deepseek-213 whole-reply ranker
rejection on fresh `main`, record the merge receipt and remove the completed
coordination claim.

This is release bookkeeping only. It does not change the rejected ranker or the
serving default.

## Done when

1. Fresh-main source, compact evidence and published checksum hashes match the
   reviewed deepseek-213 head.
2. Fresh-main focused and full CPU tests pass.
3. The `deepseek/prompt-reply-ranker` claim is removed.
4. A release REPORT merges through the protected queue.

## Stop rule

Stop if the reviewed source or evidence differs on fresh `main`, any claimed
test fails, another lab's claim overlaps this bookkeeping change, or any source
behavior change would be required.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-214/ORDER.md
