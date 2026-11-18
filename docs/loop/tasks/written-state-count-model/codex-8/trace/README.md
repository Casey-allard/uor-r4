# [codex] Independent reconstruction of the first entity loss

The population is frozen from #4093 before looking at new outcomes. In original tape order, the 563 lines consisting of the bare-prefix SHA-256, a tab, the case ID and a newline have SHA-256 `46df490c5688a9ae540d37c846ea89d574c9f8cd3a9c1f3b3dd44741c5a10ab6` at both released seeds.

The CUDA predictor returns a pair of integer query-role and fact-context costs for each tape position. Its blind output includes both original and repaired choices on the same model, tape and saved state. [audit.jq](audit.jq) independently enumerates every eligible earlier identical-token address, adds the two device costs, and sorts the full tuples (sum, query position, fact position). It then recomputes tokenizer word bounds, tests overlap, matching within-word offsets and ordered token identity, and sorts the filtered tuples. Every reconstructed winner and every returned span bound must equal the corresponding device result. Full words are compared piece by piece. A synthetic pair with matching first two IDs and an unequal third piece supplied a finite rejection witness.

Only the grader uses the annotated requested query and requested fact to find the best ground-truth-compatible tuple among all candidates. The original rank and the rank after whole-word filtering are one-based. Cost ties favor the lower query position, then the lower fact position. The final null gate is recorded separately from the first stage that loses the requested entity. Query and fact are one joint decision.

[summarize.jq](summarize.jq) checks the two source choices and three sets of emitted IDs against the bare writer, reconstructs zero-based first mismatch on literal value plus EOS, and reports both members of each matched role pair. A requested-source hit counts only when the selected address equals the annotated start. An exact reply counts by token length and literal IDs. The focal TSVs retain all original abstentions, including persistent misses.

Run both commands against the bundle's matching seed and split. `GRADE` names the grade tape for this split. `BANK` names the frozen span state, which contains the tokenizer classes. `BASE` and `FIX` are blind selector outputs. `BW`, `RW` and `OW` are the actual-feedback baseline, repair and anchor-oracle writer outputs.

```sh
jq --slurpfile grade "$GRADE" --slurpfile bank "$BANK" --slurpfile repair "$FIX" \
  -f audit.jq "$BASE" > ranks.json
jq --slurpfile grade "$GRADE" --slurpfile base "$BW" --slurpfile fix "$RW" \
  --slurpfile oracle "$OW" --arg format summary -f summarize.jq ranks.json
```

[Counts for all four panels](MEASUREMENTS.json) are identical at both seeds. On the 563 focal rows, the best requested tuple was present and whole-word compatible every time. It ranked second on 340, third on 137, fourth on 68, fifth on 5 and sixth on 13. The chosen wrong pair won by lower cost on 501, and by the positional tie-break on 62. The chosen two words differed at the final BPE piece on all 563. Post-filter, the requested pair ranked first on 473, second on 77 and third on 13.

[Two literal traces](EXAMPLES.json) show the cost and output transitions. The repair emits an exact answer on 441 former abstentions. Of these, 439 select the correct entity pair and value address; two select a repeated connector token as the anchor and still land on the requested address. All 90 wrong entity pairs after filtering comprise 72 excluded-name to distractor-fact bindings and 18 bindings involving tokens outside the annotated entity spans. Thirty-four other rows now have the correct entity and still miss the value address.

[Seed 1, 563 rows](seed1-transfer-nulls.tsv) · [Seed 2, 563 rows](seed2-transfer-nulls.tsv)
