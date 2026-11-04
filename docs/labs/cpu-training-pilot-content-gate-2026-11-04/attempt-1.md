# Attempt 1: superseded instrument evidence

Attempt 1 ran implementation source
`18b0bc8c07720160913435f338ae9f43c1b0194f`. Exact-head delivery review
found that the driver's paired comparisons used
`candidate_only + control_only` in the square-root threshold, while the
ORDER and FIT sheet registered only `control_only`.

This changed the displayed decision for seed 1's true-vs-shuffled BOS
comparison: the recorded threshold was 14, while the registered rule produced
9 and passed with 10 candidate-only rows. The training-recall and exact-reply
failures independently remained negative.

The complete attempt-1 run remains in
[`evidence-attempt-1/`](evidence-attempt-1/) with its original checksums and
[resource ledger](attempt-1-resource-ledger.json). It is superseded for the
unit decision by attempt 2 at implementation pin `249b29607`, which applies
the registered formulas and reruns both seeds plus the seed-1 repeat.
