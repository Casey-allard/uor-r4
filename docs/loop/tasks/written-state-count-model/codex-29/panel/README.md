# D29 held source and role panels

The source panel has 31 prompts. Six families each cross two objects with a
short and a long expression of the same relation. All 12 short gold onsets
follow 2–3 actual connector pieces. All 12 long gold onsets follow 4–7. Every
long row and six short rows have an earlier legal competing start inside the
same assertion and copy bound. The remaining short rows have a legal later
suffix control. The three controls cover duplicate agreement, conflicting
descriptions of the same inspection, and no entry for the person asked about.

Four separate contextual prompts test two boundary switches. Within a pair
only the earlier context varies. The original token suffix from the factual
name through the entire query and assistant framing must be identical. The
same wording can describe mailing earlier magazine issues or returning
received issues, and can describe moving chairs designed for a residential
office or taking office chairs home. The gold tape changes, as does the
source address relative to the factual name. Both plausible starts are legal
copy controls under the same original fence. Grade each pair separately from
the six matched source families.

The role panel crosses the requested name, order of the two factual
assertions, and order of the request and exclusion clauses. A third person
appears as an unrequested role and holds an independent value in one family.
The other crossed family has only two people. Each family includes an
additional direct one-person question. Score the half-open requested and
excluded endpoint pairs separately. In the third-person block also report
whether either output slot chose the unrequested query occurrence.

The author must share only `held.blind.tsv` and `role.blind.tsv` for
independent language adjudication. Original ID tapes, framing and separate
requested endpoint files have the D28 ABI. Gold also records the cross
factors, contextual local tape and role endpoints. Neither a gold nor a
person file is an ordinary model input. The named supplied lanes are
diagnostic interventions with separate results.

## Build and reproduce

```sh
cargo build --locked --offline --release --manifest-path "$PANEL/Cargo.toml"
d29-rival-panel TOKENIZER NEW_DIRECTORY
```

The generator binds the current seed-1 tokenizer SHA, reconstructs protocol-2
original IDs, verifies named word boundaries and first real stop, and checks
each supported answer through EOS using an independently encoded assistant
message. Exact authored person, connector and object fields are compared
against the embedded D28 TRAIN, development and independent held fields.
D28 TRAIN was frozen before these prompts. No D29 fit or result is used to
generate the panel.
