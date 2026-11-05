# Copy termination-oracle ceiling

This record analyzes the sealed deepseek-153 decision traces without running a
new model forward. Each selected-token trace is truncated to the reference
length and given an oracle BOS.

Attempts 1 and 2 are preserved instrument faults: they correctly computed the
oracle body but reported the original no-stop `exact_reference` field.
Attempt 3 reports the oracle-stopped reading but omitted report binding and
mean-prefix output. Attempt 4 found the row/aggregate hash mismatch. Attempt 5
produced the correct bounded result; attempt 6 adds schema and arm-name checks.

On both trained artifact seeds:

| Panel | Longest exact | `k2>k1` | Depth 1 | Random | Writer |
|---|---:|---:|---:|---:|---:|
| train,train 2711 | **11 / 453** | 8 | 0 | 0 | 0 |
| dev,dev 2712 | **13 / 449** | 8 | 0 | 2 | 0 |

Longest-first passed every paired exact and body-position comparison against
`k2>k1`, depth 1, random depth and writer in both artifact/panel cells. It had
13/11 exact bodies and another small set of near misses two or three tokens from
the reference. This means perfect stopping alone would produce a bounded set
of complete replies, but most current copy trajectories still reconstruct
their body incompletely.

The next mechanism should therefore combine the longest-first copy read with
an actual termination signal and then target the copy body's last-token
errors. The oracle itself is not deployable and no language claim follows.

Published evidence is in [`evidence/`](evidence/). The raw sealed attempt
manifests remain in local storage. Generated panels and no new model output are
not committed.
