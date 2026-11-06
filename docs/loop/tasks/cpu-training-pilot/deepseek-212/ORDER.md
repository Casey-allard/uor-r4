# ORDER: cpu-training-pilot, deepseek-212 (release compressed VSA claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `2253ede36`.

## Goal

Verify the protected delivery of the deepseek-211 compressed prompt-VSA
rejection on fresh `main`, record the merge receipt and remove the completed
coordination claim.

This is release bookkeeping only. It does not change the experimental result or
the serving default.

## Done when

1. Fresh-main source, compact evidence and published checksum hashes match the
   reviewed deepseek-211 head.
2. Fresh-main serving, OpenAI loopback, CPU and focused tests pass.
3. The `deepseek/prompt-vsa-compression` claim is removed.
4. A release REPORT merges through the protected queue.

## Stop rule

Stop if the reviewed source or evidence differs on fresh `main`, any claimed
test fails, another lab's claim overlaps this bookkeeping change, or any source
behavior change would be required.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-212/ORDER.md
