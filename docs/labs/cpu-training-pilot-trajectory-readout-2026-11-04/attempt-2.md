# Attempt 2: corrected-source reproduction

Attempt 2 rebuilt and reran the registered seed-1 select-only preflight from
implementation source `4388db081b30ee0c554f99b41500c667cf08eb8c` after a fresh
review found that the full-generation fallback predicate compared
reference-aligned decisions with all generated decisions.

The repair adds an explicit total-generated-decision count and pins a capped
generation whose reference horizon is shorter than the reply. It does not
change the trajectory construction, inputs, split, threshold rule or preflight
floor.

The corrected run reproduced the attempt-1 preflight report and structured
failure byte-for-byte:

| Metric | Code | Trajectory |
|---|---:|---:|
| Rows | 894 | 894 |
| Covered rows | 807 | 806 |
| Leave-one-out top-1 | 300 | 297 |
| Ambiguous rows | 277 | 254 |
| Fitted radius | 537 | 1,080 |

The code and trajectory target-mapping hashes and the 632/633
changed-assignment count also match attempt 1. The registered preflight
therefore stopped before seed 2, generation and the repeat.

Attempt 1 remains unchanged at its original seal. Attempt 2 is under
[`evidence-attempt-2/`](evidence-attempt-2/); its `SHA256SUMS` is
`140cf3f54cf60931d149bd89359444e2caf67629f0e2485dcc4aa4ee5e288ee8`.
