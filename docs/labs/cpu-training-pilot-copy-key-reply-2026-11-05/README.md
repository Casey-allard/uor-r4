# CPU longest-first copy selection in complete replies

This record contains two attempts. Attempt 1 stopped before producing a
comparison result because its freshly generated panels contained assistant
responses of 49 and 46 tokens under a 32-token cap. It is preserved as an
input/config instrument fault.

Attempt 2 used the maximum supported 64-token cap and new unopened
m-world-v2 panels at seeds 2711 and 2712. Every reply was reference-reachable.
On both public trained artifact seeds and both panels, longest-first exact copy
beat `k2>k1`, fixed depth 1, random available depth and the unmodified writer
on correct reference body positions by the registered paired rule.

Correct body positions for longest-first were **409 / 4,661** and
**556 / 4,614**; controls were:

| Panel | `k2>k1` | depth 1 | random | writer |
|---|---:|---:|---:|---:|
| train,train | 376 | 158 | 216 | 60 / 59 |
| dev,dev | 458 | 192 | 307 | 55 / 49 |

The two writer columns are artifact seeds 1 and 2; copy-arm trajectories were
identical across those seeds because every decision was a copy decision.

No arm produced a BOS-complete or exact-reference reply. All 453 replies on the
train panel and 449 replies on the dev panel hit the 64-token cap in every arm.
The prompt-derived copy stream excludes BOS, and there were zero writer
fallbacks, so this override cannot itself emit the stopping sentinel. The
classification is **selection-surrogate positive but termination-limited**,
not a complete-reply or language result.

Published evidence is in [`evidence/attempt-1/`](evidence/attempt-1/) and
[`evidence/attempt-2/`](evidence/attempt-2/). Raw token/mask payloads and full
decision reports remain in local sealed evidence and are omitted from Git. The
committed panel manifests bind the corpus hashes, and the summary reports
retain every arm, comparison, random-control count and decision hash. The
published subset verifies with
[`PUBLISHED-SHA256SUMS`](evidence/PUBLISHED-SHA256SUMS).
