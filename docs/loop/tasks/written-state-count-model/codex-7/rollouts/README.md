# [codex] Actual emitted tokens through EOS

Each TSV row is one prompt in one released seed. The columns record the selected source address, the zero-based first wrong output position, the expected and emitted token at that position, the output index of an explicit EOS action and the complete list of emitted IDs. `exact` means the entire reply equaled the value followed immediately by EOS, including length. `NA` is an absent address, token, or action. The `connector_ids` column counts BPE tokens in the requested connector. Oracle and Disabled are controls.

The summary compares each real treatment to WordStart and the combined arm to each individual one. Each comparison has later/same/earlier counts, a map of absolute index transitions, correct addresses gained and lost, and exact replies gained and lost. Exact replies are also grouped by where the selection actually landed. The signed exact-count interaction is CE − C − E + W. The role-pair counts require both questions on the same written facts to succeed. Leave-one-family-out counts keep the entire role pair. The old panels are primary (1,728), fresh (1,728), prior (288) and original (16). Transfer (1,728) is the new held-out panel.

The independent grade reconstructs each selected cursor, release reason, copy action and emitted token from the original prompt and recorded native proposal. It checks grade metadata, factor invariants, identical pre-release steps for EOS toggles, all four real arms' entity endpoints, and every historical WordStart, Disabled and Oracle trajectory against #4090. The writer verifies the four actual-feedback traces against the prefix-only run before comparison. Use the original bundle's writer report as PREVIOUS for each historical panel. Supply an empty array in place of PREVIOUS on transfer.

```sh
jq --slurpfile grade "$GRADE" --slurpfile previous "$PREVIOUS" \
   --arg split "$SPLIT" --argjson n "$COUNT" --arg format summary \
   -f ../grade.jq "$WRITER"

jq -r --slurpfile grade "$TRANSFER_GRADE" --argjson previous '[]' \
   --arg split transfer --argjson n 1728 --arg format tsv \
   -f ../grade.jq "$TRANSFER_WRITER"
```
