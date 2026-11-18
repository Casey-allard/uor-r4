# [codex] Actual-feedback CE replay under entity source replacement

Each arm uses the same #4093 native-generation, exact-copy and EOS CUDA images. The writer accepts one released model, one bare tape and one sealed selector handoff at a time. Only the continuation source enters the copy cursor. The arm is metadata; native generation, copy, EOS, their precedence, the frozen initial prompt fence, and the 16-step decode cap use the same typed operations in every arm. The unchanged replay checks the device's integer copy words, committed ID, native prediction, admitted cell count, read vector and top eight against serving on the actual emitted prefix.

Reproduction uses the standalone scorer built on Wick with the existing copy-completion feature:

```sh
d11-scorer completion-finite MODEL
d11-scorer entity-writer MODEL BARE CHAIN_PICKS NEW_RAW
```

The [compiled manifest](INPUTS.json) binds the three input digests together. Experimental and oracle source files run in separate processes. Every output preserves the per-prefix fingerprint and full sequence of source/cursor and native/committed-token transitions. The [independent grader](grade.jq) joins the completed trajectories to the grade, checks the original prefix arrays, reconstructs each exact copy/guard decision, scores literal equal-length value plus EOS, and compares the historical baseline to #4093 for all three reused panels. The renamed challenge does not have a historical writer file. The baseline-null cohort and complement together cover every row, without conditioning admission on treatment success. Positive and negative movement refer to the zero-based first incorrect committed ID and count exact as later than any mismatch. `source_recovered` requires the original requested address.

In each [case table](rollouts/seed1-transfer-cases.tsv), address is the selected absolute prefix position, `first_wrong` is a zero-based output index or `exact`, and `emissions` includes the committed stop. The summary also grades both requested roles in each fixed-facts pair. A matching first word from a distractor counts as a later first error, while the requested-source count requires exact address equality. Reproduce the historical summary from the bound files as follows. For the new challenge pass `--argjson history '[]'` in place of the historical input.

```sh
jq --slurpfile repaired "$REPAIR_RAW" --slurpfile oracle "$ORACLE_RAW" \
   --slurpfile grades "$GRADE" --slurpfile bare_rows "$BARE" \
   --slurpfile history "$HISTORIC_WRITER" --arg grade_hash "$GRADE_SHA256" \
   --arg bare_hash "$BARE_SHA256" --arg format summary -f grade.jq "$BASE_RAW"
```
