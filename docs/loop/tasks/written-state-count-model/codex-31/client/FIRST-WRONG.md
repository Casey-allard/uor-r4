# [codex] First wrong token in the fitted P1 client

All twelve scored replies diverged at step zero. Both read modes chose original ID **799** on every fixed-gold prompt. The winning Q48 mass was **213,599,350,318,210** with Pick and **213,599,350,367,942** with read disabled on rows 31001–31005. Row 31006 had respective winning masses of **229,952,693,195,806** and **231,120,153,046,176**. The ranks below include all 4,095 legal original IDs and break equal masses by the smaller ID. [Per-arm integer evidence](first-wrong-mass.json)

| Prompt | First gold ID | Gold rank in both modes | Gold Q48, Pick enabled | Gold Q48, read disabled | Pick successor and rank |
|---|---:|---:|---:|---:|---:|
| 31001 | 581 | 96 | 1,237,799,510 | 1,237,799,510 | 439 at 10 |
| 31002 | 3646 | 2928 | 1,084,197 | 1,084,197 | 439 at 10 |
| 31003 | 581 | 96 | 1,237,799,510 | 1,237,799,510 | 2593 at 411 |
| 31004 | 3646 | 2928 | 1,084,197 | 1,084,197 | 2593 at 411 |
| 31005 | 1667 | 2555 | 1,849,296 | 1,849,296 | 1049 at 30 |
| 31006 | 756 | 383 | 122,038,184 | 122,657,766 | 1049 at 7 |

Both asked-person swaps changed the encoded prefix. With fact order fixed, they changed zero of 4,095 fitted buckets, zero candidate words in any of the 14 KN, ML, uniform and Pick expert lanes, and zero combined candidate masses in either read mode. Reversing the facts while holding each asked person fixed changed the Pick successor from 439 to 2593 at the same predecessor index, distance and tie count `[3,12,1]`. The other 13 expert lanes stayed bit-identical, and only those two candidate IDs changed combined mass with Pick enabled. The same reversals changed zero candidate words without the read. The frozen candidate rows carry exact ML probability words, while raw count and total words are outside these particular full-vocabulary tapes. [Crossed comparisons](first-wrong-mass.json)

At the first error, the Pick successor was neither the gold ID nor ID 799 on any scored prompt. The gold and the winner therefore each received zero direct Pick mass. The actual picked candidate gained exactly 65,536 Q48 units on rows 31001–31005 and 1,421,817,741,312 on 31006, with its original fitted bucket and all other lanes held fixed. The two full read modes use different fitted buckets. The weighted KN lane alone accounts for 213,599,349,998,727 Q48 units of the representative order-seven winner on 31001 with the read enabled. The longest active count order was seven on rows 31001–31005 and eight on 31006. The observed error involves both failure to nominate the target and dominance of the same non-gold count winner. Later positions follow the generated trajectory after this first error.

The post-score Rust analyzer opened sealed Wick copies of the original CUDA candidate vectors and expert rows. It reconstructed the exact fixed-bucket mixture after multiplying unsigned Q32 weights by Q48 expert masses, verified the device argmax and its ties, then applied the already-frozen gold key. It ran after both blind CUDA processes finished. Its output SHA-256 is `d24a5104fe50dbb50aa7df51e59a0c1a407d01448a36935264ed092679ebd154`. The raw archive at `/mnt/plexpool/uor-r4-corpus/runs/20261120-complete-c1-31/repaired-v4/client-v4/full/runs/count31-repaired/v4/client-real` strictly rehashed, and primary and cold step and receipt files compared byte for byte. [Post-score analyzer](tool/src/bin/first-wrong-mass.rs) · [Blind client protocol](SHEET.md)
