# Where the repaired selector first loses the answer

The #4094 repair still misses 219 of 1,728 focal replies. Its first failed decision is the entity pair on 116 and the value address on 103. The original 563-abstention cohort accounts for 122 of those misses. The 1,165 prompts that already had an address account for the other 97. This is a regrade of the sealed Wick outputs. Both seeds gave the following counts on the same prompts. [Original experiment](../codex-8/REPORT.md) · [Rank reconstruction](../codex-8/trace/audit.jq) · [Complete writer trajectories by case](../codex-8/writer/rollouts)

| Panel, 1,728 prompts each | Wrong entity pair, missed reply | Correct pair, wrong value | First wrong at 0 | First wrong at 1 | Exact through EOS | Wrong pair among exact replies |
|---|---:|---:|---:|---:|---:|---:|
| Primary | 731 | 186 | 677 | 240 | 811 | 80 |
| Earlier fresh | 197 | 552 | 714 | 35 | 979 | 0 |
| Focal transfer | 116 | 103 | 165 | 54 | 1,509 | 4 |
| Renamed challenge | 110 | 115 | 171 | 54 | 1,503 | 2 |

“Wrong pair” means the selected positions fail to match the annotated requested query and its fact. “Correct pair, wrong value” means both occurrences match and the selected source differs from the annotated value start. A different pair occasionally routes to the right address, so pair recovery and emitted-answer recovery are measured separately. Offsets count emitted BPE tokens from zero. Exact requires the full expected token sequence including immediate EOS.

| Focal cohort | Rows | Wrong pair, missed reply | Correct pair, wrong value | First wrong at 0 | First wrong at 1 | Exact through EOS |
|---|---:|---:|---:|---:|---:|---:|
| Previously abstained | 563 | 88 | 34 | 68 | 54 | 441 |
| Previously returned an address | 1,165 | 28 | 69 | 97 | 0 | 1,068 |

Of the 116 focal wrong-pair misses, 99 bind the excluded query occurrence to its fact and 17 bind words outside the annotated entities. The wrong pair has strictly lower energy on 108 and the same energy on eight. Four further wrong pairs tie and reach the correct value. Across these 120 wrong-pair selections the requested compatible tuple ranks second 105 times and third 15 times. Swapping the tie order alone cannot recover the 108 strictly lower-cost selections.

For the 103 focal cases with the right pair and wrong value, the selected start minus the requested start, in prompt tokens, is **−2** for all 45 “found and hid” misses, **−1** for all 43 “found just” misses, and **+2** for all 15 “held” misses. A fixed shift cannot recover all three. These are strata of this tape. Different connector phrases and question wordings are needed to test transfer.

The focal oracle that supplies both requested entity anchors and leaves the value selector intact moves 53 first errors from 0 to exact and 54 from 1 to exact. It also moves two exact replies to 0. Another 112 remain wrong at 0. Its total is 1,614 exact, a net gain of 105. The two arms share the exact writer. This comparison isolates the consequences of supplying both anchors together.

## Reproduce the tables

Use the checkout containing this file and the [pinned Wick archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/a7aa91284de556f70c3f2472e68bedeeab9cb1e5/results/codex/entity-abstention-20261118/entity-abstention-wick.tar.gz). Set `archive` to its downloaded path. Its SHA-256 is `2ad769cbd249705dcc84a178a016b53c74f82795078549f0c2232c0d9bd55654`. The filter checks case ID, prefix hash, requested and selected addresses and original null status against the committed TSV. It recomputes the first mismatching token from the emitted arrays for the repair and anchor oracle. The output includes the two cohorts, occurrence classes, energy ties, ranks, signed offsets and paired transitions. All four seed-to-seed comparisons passed with jq 1.7.1.

```sh
set -eu
base=docs/loop/tasks/written-state-count-model
out=$(mktemp -d)
for panel in primary fresh transfer challenge; do
  for seed in 1 2; do
    stem="seed$seed-$panel"
    tar -xOzf "$archive" "evidence/results/trace-independent/$stem-ranks.json" > "$out/$stem-ranks.json"
    jq --rawfile writer "$base/codex-8/writer/rollouts/$stem-cases.tsv" \
      -f "$base/codex-9/residuals.jq" "$out/$stem-ranks.json" > "$out/$stem.json"
  done
  cmp "$out/seed1-$panel.json" "$out/seed2-$panel.json"
  jq '.all' "$out/seed1-$panel.json"
done
jq . "$out/seed1-transfer.json"
```
