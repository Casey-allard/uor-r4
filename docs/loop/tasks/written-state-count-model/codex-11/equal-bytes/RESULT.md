# [codex] Exact counts and rounded counts in equal-size files

Fourteen thousand seven hundred forty bigram keys fit in 86,670,824 bytes with either precision. Restoring the exact TRAIN count changed 12,895 of them. Both reloads kept the same unigram, conditional weight 102/256 and entire original seed-1 trunk. The power-of-two codec occupies a full u32 in this carrier. On 10,807,417 held targets, restoring precision gained 82,531 greedy hits. The change in likelihood was smaller. [Construction](construction.json) · [Device restart and parity](primary.json)

| Same keys, weight and file size | DEV-tune bits per byte | Novel bits per byte |
|---|---:|---:|
| Rounded survivor counts | 3.584614 | 3.671571 |
| Exact survivor counts | 3.581048 | 3.667959 |
| BPB saved by restoring counts | 0.003566 | 0.003611 |

The scopes contain 21,928,475 and 15,995,822 scored bytes respectively. Two freshly opened restored files produced matching GPU tape hashes and Q12 totals. Those scores qualify precision on this byte layout. A packed exponent layout could have a different storage tradeoff. The complete restored D11, raw device output and launch receipts were downloaded from the pinned store revision and compared byte-for-byte. [Wick evidence bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/75740a3a659787debc5958fdbb795fb07f3c38db/results/codex/content-support-count-20261118/precision-wick.tar.gz) · [Construction and refusal checks](../../../../../labs/kn-reference/instrument/src/bin/count-precision.rs)
