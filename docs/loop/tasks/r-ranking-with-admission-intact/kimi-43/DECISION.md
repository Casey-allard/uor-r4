# D68 decision: close the diagnostic, not the full-dose ladder

## Whole-project context

#2884 is a research diagnostic, not an M1-close-path item. Its unchanged done-when asks for one admission-intact run with conditional ranking reported against count tables. The later FIT registered a stronger three-arm, two-seed, full-dose experiment after #2836. Stage T did not qualify that full-dose ladder: its FrozenZero arm stopped at batch 3,008, Relation stopped at Q1, and its two seed bindings produce identical table/gate trajectories.

D67 fresh review `agent-44` at `8fdc3f9778c2f990a91f6e281ad53e37d265f9be` failed dispatch: `Arm-Rel-K16` is rejected by the A8-only validator and baseline, prior measured Relation throughput projects above the 35-minute gate, the current trainer has Q1/consecutive-rejection stops, the three-arm grading outputs are absent, and local disk was below the ORDER margin. No pod launched. Record: #2884 comment `6424741498`.

D68 fresh review `agent-45` at decision head `de79c3df67d52304a1f894efd0b848a941673953` returned **D68 DECISION REVIEW: PASS**, conditional on a negative diagnostic close with exact scope and no full-dose, independent-seed, fresh-panel, above-unigram or M1 claim. Owner log: #2884 comment `6424978255`.

## Decision

Close #2884 `done` only because its literal diagnostic done-when is already met by sealed #2831 Q1 evidence and count-table comparisons. Supersede kimi-43's broader six-run plan with kimi-44. Do not alter the item's done-when, the original FIT, CHARTER, PLAN, mechanism status or any milestone bar.

This is a negative close, not a model success. The deferred full-dose question remains explicitly `NOT_RUN`; the original FIT and D67 failure preserve its instrument defects. The related existing longer-training route is #2878 (`r-full-epoch-revisit-schedules`), but it does not authorize or subsume this ladder. If the exact ladder is proposed again, it requires a repaired design and implementation, a new sheet, fresh FIT and D67 before any spend.

## Evidence and required wording

- Sealed #2831 archive: `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`; Q1 evaluation: `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`; Q1 checkpoint: `9a7dd77037dc3e239a73a3900c62fad41279a936727b950b9224f11f2b273184`.
- Q1 gate-relative admission: 256/256 occupied cells, mean 126.49 admitted tokens/row, 4,292/14,287 full-panel admitted and 1,106/3,847 P_succ admitted. P_succ fell 245 rows from 1,351; do not say admission was unchanged.
- Ranking: 871/4,292 = 20.29% conditional top-1 and 1,455/4,292 = 33.90% conditional top-8. Full-panel top-1 is 871/14,287, below Unigram 1,226 and T2 6,300.
- Host counting of stored GPU rows only reproduces 972/4,292 = 22.65% admitted targets equal to token 1, the full-panel unigram argmax. This same-row ordinary control exceeds the writer by 101 rows. The 754/1,920 comparator belongs to Rung-0 Q4 and is not Q1 admission-matched.
- At 4,292 admitted rows, tying full-panel Unigram requires 28.56% conditional top-1; tying T2 would require 146.78%, impossible. Even a perfect ranker needs at least 6,300/14,287 = 44.10% admission to tie T2. Ranking and admission are both bottlenecks.
- Stage T is supporting context only. Its two seeds are a determinism replicate, not independent training evidence; neither arm reached Q4. No fresh evaluation occurred.
- Additional GPU cost in this unit: $0. The full-dose Step 1 checkpoint exists, but its matched GPU scorer was not run; do not call Step 1 executed or the whole ladder measured as non-executable.

## Close conditions

The REPORT leads with the ordinary same-admitted-row unigram loss and includes all evidence limits above. The item's stale “What is missing” receives a dated correction while its done-when stays unchanged. The notebook records one negative result. Exact-head review verifies the sealed artifact checksum and claims; no host model evaluation or GPU rerun is part of this close.
