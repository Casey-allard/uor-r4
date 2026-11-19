# Connector boundary witness

Two independently read pairs use the same target sentence within each pair. The preceding context changes the full noun phrase readers selected. This probe preserves the authored contexts and target sentences, places the target immediately after a period so it can bind as a fact, and asks explicitly for the full noun phrase. It supplies only the original query-name span to the frozen seed-1 D11 binder. The onset decision, candidate masks and cursor are the native operators.

| Context | Intended fact span | Joint candidate slots and source addresses | Gold start | Ordered candidate | Final binder source |
|---|---|---|---|---|---|
| Morgan, railway payroll | [50,53) | +2 help 54, +3 train 55, +4 drivers 56 | train drivers at 55 | 55 | abstains |
| Morgan, trucking school | [45,48) | +2 help 49, +3 train 50, +4 drivers 51 | drivers at 51 | 50 | 50 |
| Jo, assistance-animal clinic | [47,48) | +2 help 49, +3 guide 50 | guide dogs at 50 | 50 | 50 |
| Jo, pet shelter | [40,41) | +2 help 42, +3 guide 43 | dogs at 45 | 43 | 43 |

Slot offsets are measured from the last token of the factual name. Morgan occupies three tokens and Jo occupies one. Morgan gives an exact local contrast with both plausible onsets admitted. The two contexts retain the same eligible and word-start masks, relative shapes and ordered class/root sequences. The ordered head chooses +3 in both contexts. It therefore cannot distinguish these readings from the recorded local descriptor alone. In the railway context the preceding sentence “Morgan is an accountant and is forbidden to give driving instruction” also binds at [25,28). Its admitted starts are at least 19 tokens from the fence, beyond the exact writer's cap. The intended fact completes, but the final binder requires both bound assertions to complete and returns no source. This consensus failure is separate from the local onset collision.

The Jo pair also retains the same local descriptors. In both contexts the eligible mask is 7 and the word-start mask is 3. The +4 slot falls on the continuation token `ide`, following ` gu`. The late gold, ` dogs`, is at +5 and never enters the onset candidate set. A scoring-only change would leave that omission in place.

Absolute source, word-end, fence and query addresses differ across contexts. The comparison checks the original projected candidate keys after expressing those offsets relative to the factual name. The Morgan pair isolates the need for context outside the shared target sentence once both starts are visible. The Jo pair exposes the separate need for a horizon over whole words or a candidate window that admits +5. The railway row shows why a subsequent test must also identify the intended assertion before interpreting final source abstention.

Forcing either original source start in each context produces the complete literal and EOS. The Morgan tapes are `[1504,2141,85,1]` and `[2141,85,1]`. The Jo tapes are `[1694,428,2381,1]` and `[2381,1]`. The probe checks decoded literal text and cursor completion against the original prompt span. These four cases are finite, supplied-query serving observations on one frozen artifact. They do not contribute to a held reply or role score.

From this directory, use the pinned D21 parent and D15 tokenizer. The verification command writes the same compact JSON as [witness.json](witness.json).

```sh
cargo run --locked --quiet -- /absolute/path/to/seed1.d11 /absolute/path/to/tokenizer-seed1.json | jq -c .
```
