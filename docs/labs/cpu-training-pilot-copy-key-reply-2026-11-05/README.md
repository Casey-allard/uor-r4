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

## Longest-32 depth extension

Deepseek-163 tested the untried exact suffix range above depth four. On a new
seed-2752 dev panel with 458 reachable replies, the deepest-32 arm selected
depth 32 on 15,114/29,312 decisions and beat `k2>k1`, fixed depth 1,
random-32 and writer. It tied nested longest-4 exactly:

- train: **409/4,661** correct body positions for both arms;
- fresh dev: **300/4,758** correct body positions for both arms.

Longest-32 and longest-4 chose the same token at every fresh-dev decision.
They matched every scored train body position, with 101 train token
differences per seed after the reference. No arm emitted BOS or an exact
reference. The two fresh-dev runs repeated byte-identically.
This closes the exact-depth extension to 32, not exact copy or a different
geometric read. See
[deepseek-163](../../loop/tasks/cpu-training-pilot/deepseek-163/REPORT.md)
and [`evidence/depth32-2752/`](evidence/depth32-2752/).

## Geometric copy arbitration

Deepseek-165 replaced longest-first arbitration with sign-Hamming distance
between the current 256-coordinate Q12 writer query and each exact-copy
candidate token's shared vector-table row. Distance, deeper suffix depth and
lower token id formed the fixed tie order.

On a fresh seed-2762 dev panel with 443 reachable replies, the arm changed
5,106 and 4,891 decisions versus longest-32 across the two artifact seeds. It
scored **270 and 286/4,822** correct body positions against **624** for
longest-32, **547** for `k2>k1` and **414** for random-32. On train it scored
**174 and 195/4,661** against **409**. It beat fixed depth 1 and writer on
fresh dev, but failed every required comparison against the stronger copy
controls and the train non-inferiority floor.

Every geometric copy decision carried its selected distance, both fresh
processes repeated byte-identically, and all arms had zero exact, BOS-complete
and premature-BOS replies. The classification is **REJECT** for raw
token-row sign proximity as copy arbitration. See
[deepseek-165](../../loop/tasks/cpu-training-pilot/deepseek-165/REPORT.md)
and [`evidence/geometric-2762/`](evidence/geometric-2762/).

## Learned geometric copy read

Deepseek-167 trained a modulo-256 query-code readout on generated longest-32
train states and allowed it to choose only from the exact-copy candidate set.
On a fresh seed-2763 panel with 453 reachable replies, the learned read scored
**474 and 475/4,686** body positions against **474** for longest-32. The
shuffled readout scored 475 and 474. The paired longest and shuffled
comparisons both failed.

The learned read arbitrated 210 and 174 candidate decisions and changed 52 and
245 decisions versus longest-32. Its 1,024-row fit had only 116 and 125
leave-one-out top-1 rows, with 957/2,048 fitted radius. The result closes this
row-level nearest-example representation; it is not evidence against all
learned geometric attention. See
[deepseek-167](../../loop/tasks/cpu-training-pilot/deepseek-167/REPORT.md)
and [`evidence/learned-read-2763/`](evidence/learned-read-2763/).

## Exact-copy model lookahead

Deepseek-169 advanced every unique depth-1..4 candidate through a clone of the
exact forward and ranked the resulting next-state modulo codes by learned
readout distance. On fresh seed 2764 with 488 reachable replies, it scored
only **194 and 191/4,799** body positions versus **393** for longest-32. It
selected depth four on more than 25,000 of 31,232 decisions.

Target-shuffled lookahead made zero different decisions because target identity
was not part of the score. The registered control gate therefore stopped as
`INSTRUMENT_FAULT / CONTROL_INVALID`. See
[deepseek-169](../../loop/tasks/cpu-training-pilot/deepseek-169/REPORT.md)
and [`evidence/lookahead-2764/`](evidence/lookahead-2764/).

## Continuation-sensitive copy lookahead

Deepseek-171 made the next-token prediction part of candidate ranking. For
each unique exact-copy candidate, it retained the deepest source occurrence,
cloned the exact forward, fed the candidate, read the resulting learned target,
and compared it with the token that historically followed that occurrence.
Target shuffling then changed 167 and 278 decisions, so the repaired control
was non-vacuous.

The signal was too sparse to help. On fresh seed 2765 with 481 reachable
replies, continuation lookahead scored **242 and 262/4,899** correct body
positions against **511** for no-lookahead learned and longest-32. Only 231 and
209 of 30,784 evaluated decisions matched the historical continuation. It
changed about 4,500 decisions per seed and remained near 12.6k-12.9k generated
tokens/s, versus 66k-68k for longest-32. Every arm repeated deterministically
and emitted zero exact, BOS-complete or premature-BOS replies.

This closes the candidate-conditioned single-token learned lookahead
configuration. Longest-32 remains the stronger body selector; the active
completion gap calls for an independent termination path rather than another
scalar retune. See
[deepseek-171](../../loop/tasks/cpu-training-pilot/deepseek-171/REPORT.md)
and [`evidence/continuation-2765/`](evidence/continuation-2765/).

## Pairwise exact-copy candidate ranker

Deepseek-173 tested a different learning target: rank the currently available
exact-copy candidates with one small integer model over depth, query/token sign
agreement, eight rotated-row agreements, recency, frequency and writer
agreement. The shuffled-target and fixed depth-only arms were registered as
matched controls.

The fit stopped at its first eligibility gate. Longest-32-generated train
trajectories produced only **74** decisions where the reference token appeared
among at least two unique candidates, against the registered **640** floor. No
ranker was fitted into a report and no reply evaluation ran. This is an
insufficiency result for this trajectory source, not a negative for candidate
ranking with more supervised states. Seed 2766 is closed for this design. See
[deepseek-173](../../loop/tasks/cpu-training-pilot/deepseek-173/REPORT.md)
and [`evidence/copy-ranker-2766/`](evidence/copy-ranker-2766/).
