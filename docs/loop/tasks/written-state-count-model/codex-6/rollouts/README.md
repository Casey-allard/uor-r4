# Paired boundary and first-error traces

Each TSV row describes one prompt in one seed. It gives the requested value position, frozen inclusive entity spans, candidate masks, and all six trajectories’ selected positions, first error indices and expected/actual IDs. Each mask uses bits 0, 1 and 2 for one, two and three connector IDs. An exact trajectory has index `exact`; an absent ID or source has `NA`. A null selected position hands the ordinary generator the original prompt. A later first mismatch can occur when the writer copies another value with the same opening.

The summaries compare WordStart, Ordered and Both directly with Current. They also compare Both against each isolated treatment. The full trained-connector inventory is reconstructed from the original labeled TRAIN tapes. True coverage means the annotated connector's raw ID sequence occurs in that inventory, regardless of which occurrence the model selected. A proper prefix means only an initial subsequence occurs. Conditional extraction counts require corresponding frozen anchor offsets inside the two correct requested-name mentions. The historical terminal-piece cohort adds a frozen source one ID before the value with observed ID 1935. On the primary rows those cohorts correspond to 105 instances of the two-ID “chose” and 99 instances of the three-ID “also chose” where Current copied “ose”.

The jq program checks the selected endpoints and masks against the supplied prefix IDs and tokenizer classes. It compares the recorded first divergence with the emitted step IDs, matches abstentions to the full copy-disabled trajectory and checks all role pairs. The count and grade files must be from the panel indicated by SPLIT, and SPAN_STATE must be the matching frozen #4089 seed state. Run from the session directory.

```sh
jq --slurpfile training "$TRAIN" --slurpfile grading "$GRADE" --slurpfile span_state "$SPAN_STATE" \
  --arg split "$SPLIT" --argjson n "$COUNT" --arg format summary -f summarize.jq "$WRITER" > summary.json
jq -r --slurpfile training "$TRAIN" --slurpfile grading "$GRADE" --slurpfile span_state "$SPAN_STATE" \
  --arg split "$SPLIT" --argjson n "$COUNT" --arg format tsv -f summarize.jq "$WRITER" > cases.tsv
```
