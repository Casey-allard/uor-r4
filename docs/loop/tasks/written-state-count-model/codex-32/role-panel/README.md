# D32 frozen D26 role comparison

The replay uses D30's 22 exposed user texts and sealed original-address
annotations. The panel has 16 crossed requests, two direct requests,
two unspecified requests and two incidental-only mentions. Each factorial
arm receives the same user text, protocol-2 tokenizer, D26 recurrence
carrier, 16-token decode cap and enabled exact copy. The runner accepts
only the separately sealed user-text TSV. The grader opens the gold file
after capture and checks the entire original token tape on every record.
Historical-inherited must reproduce the stored D26 decision, sources,
emitted IDs, cursor steps, stop and decoded text before any arm is scored.

On Wick, the sealed inputs are under
`/home/callard/uor-codex/20261120-context-swap-30/custody-v1`.
Use a fresh output directory for each capture.

```sh
raw="$custody/raw"
cargo build --locked --offline --release --manifest-path "$panel/Cargo.toml"
"$panel/run.sh" "$chat" \
  "$raw/inputs/local/codex/pre-gate-source-26/serve-native-01/recurrence.d11" \
  "$raw/inputs/local/codex/public-m1-predictor/tokenizer.json" \
  "$raw/panel/role.blind.tsv" "$captures"
"$grader" "$raw/panel/role.gold.json" "$raw/ordinary/d26" "$captures" > "$grades"
```

Stored-only grading reads the pre-existing D30 candidates and ordinary
client output. Verify the matching entries in `SHA256SUMS.final` before
running it.

```sh
"$grader" --stored "$raw/panel/role.gold.json" "$raw/ordinary/d26" \
  "$raw/role-d26.json" > "$historical_grades"
```

An endpoint is present only if one candidate has the entire half-open
original span. Admission is role-specific. In the stored arm, requested
admission means a recorded contrast and excluded admission means the
full-prefix excluded score exceeds the requested score. The historical
trace used the entire original tape as its outer region. A missing gold
label means the role should be unassigned. Control rows have no unique
answer tape. Their reference refusal is a comparison, while a selected
copy start is a false write.

The grader retains all eight role-decision words, the selected original
source, the assertion witnesses when captured, the first output difference
from historical and the first difference from the supported expected tape.
When the contextual source trace is available, the grader shows whether
the expected assertion was bound and whether the expected original start
was enumerated, eligible, selected locally or released by the paired reader.
Exactness checks the complete emission and EOS stop. Exact-copy completion
also checks the writer's final cursor and delimiter release. For a future
transfer claim, freeze concealed user texts and independent annotations
covering unseen names and phrasings and apply the registered GPU review.
