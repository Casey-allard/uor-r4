# First wrong token and entity addressing on Wick

Each case table contains one row per prompt for the named seed and split. The current arm is the frozen #4087 selector. The span arm uses both boundaries and the variable connector bank. The endpoint-only and connector-only arms apply one change at a time. Gold name endpoints are inclusive. Value starts and first-error steps are zero-based. A missing token is NA, and an exact trajectory has first-error label exact. The first free token column contains its generation index and actual ID. The full target includes immediate EOS.

Use the JSON summaries for all six arms, source accuracy, first-error matrices, displacement within correctly anchored mentions and factor contrasts. The primary new split has 1,728 prompts per seed. The 288 earlier held-out cases and 16 original cases have separate tables. Row counts by requested connector length include both members of each role reversal wherever eligible. Both-role accuracy is computed using the two named entities' fixed connector lengths so that both questions stay in the same cell. Connector-only triplets retain every other prompt token when the requested connector is removed.

The conditional extraction population requires the frozen query and fact anchors to be at the same token offset inside the requested query and fact names. Source correctness counts the absolute start of the value in the original prompt. Whole-value correctness compares generated IDs. Exact reply correctness also checks immediate EOS. An arm can match the opening of a shared value while addressing the wrong fact. The name-span stage identifies the earliest recorded mismatch in the order query anchor, fact anchor, query span, fact span, value address and free generation. Valid span positions can still have an abstained source.

Reproduce one pair of exports from the codex-5 directory using the primary raw rollout JSON from the evidence bundle. Set SPLIT to new, held or original, and N to 1728, 288 or 16 respectively. SUMMARY and CASES are new output paths.

```sh
jq --arg split "$SPLIT" --argjson n "$N" --arg format summary -f summarize.jq "$RAW" > "$SUMMARY"
jq -r --arg split "$SPLIT" --argjson n "$N" --arg format tsv -f summarize.jq "$RAW" > "$CASES"
```

The export verifies arm coverage, matching reference IDs, actual step emissions, the first free emission, first-error index and tokens, connector gap, unchanged input anchors, transition counts and complete role pairs. On the new split it also checks all three requested connector lengths occur in every matched triplet. The fitted states, prefix choices and complete per-token writer actions belong to the sealed run bundle linked in the main report.
