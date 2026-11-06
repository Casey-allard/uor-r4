# ORDER: cpu-training-pilot, deepseek-226 (release aligned endpoint claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `326821252`.

## Goal

Record protected delivery of deepseek-225, verify its fresh-main source and
sealed evidence, and remove the completed
`deepseek/content-prefix-endpoint` claim.

## Done when

1. Fresh-main source, retrieval, driver and evidence hashes match the reviewed
   head.
2. The aligned-endpoint and driver tests pass on fresh main.
3. The claim is removed and this release REPORT merges through the queue.

## Stop rule

Stop on source or evidence drift, test failure or overlapping claim.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-226/ORDER.md
