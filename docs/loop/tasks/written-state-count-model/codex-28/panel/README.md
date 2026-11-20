# D28 original value-onset inputs

The matched holdout crosses six lexical families with two original connector lengths and two original object phrases. Within each family the short and long assertions express the same relation and use the same two objects. The short form keeps the first object token within the old three-piece window. The long form moves it beyond that window. Every matched row has a competing whole-word start and an independently encoded complete answer through EOS. Three additional controls cover two matching facts, conflicting facts, and no assertion about the person requested. In each split the tokenizer locates original half-open spans and rejects a selected start on a continuation piece, a split token boundary, an invalid fence, or a value that cannot reproduce the normal assistant answer tape.

The controlled primary cells are one assertion with one named referent. They isolate source boundary behavior after the query role and assertion are supplied. The ordinary client lane exercises the same raw text through role, selection, and writer. The independent reader receives only `held.blind.tsv` before returning an answer or a reason to abstain. `held.person.jsonl` and `held.gold.jsonl` contain diagnostic supervision and stay outside the ordinary client inputs.

TRAIN uses different names, complete object strings, framing, and exact connector phrases from held. Each TRAIN candidate retains its original signed role, original coordinates, complete first-word extent and contiguous descriptor pieces. The training profile uses all of the local connector at offsets one to three and the last three contiguous connector pieces at larger offsets. `train.exemplars.jsonl` is an audit projection. The fit staging tool independently reads the canonical TRAIN tapes and the gold key, recomputes the candidate descriptors against the loaded assertion classes and D11 roots, and refuses a disagreement.

Build with the checked manifest and lock file. Use fresh output directories.

```sh
d28-onset-panel train TOKENIZER NEW_TRAIN_DIRECTORY
d28-onset-panel held TOKENIZER NEW_HELD_DIRECTORY
jq -n --slurpfile fit TRAIN/train.exemplars.jsonl \
  --slurpfile held HELD/held.gold.jsonl -f census.jq
```

The census checks only shape and class support. It computes no held geometric distances and must not be used to adjust the fitted bank or select rows. A count of zero remains in the predeclared denominator.

## Split chronology and freeze

The initial TRAIN draft generated 48 prompts in 12 lexical families. Its
first successful generation yielded 169 original-address examples. The first
development generator also exposed a tokenizer boundary on one proposed
value. Correcting that draft yielded the 27-row development packet
`held-v2`. I inspected its anonymous tokenizer shape support before adding
five generic lexical families to diversify the TRAIN profile. The resulting
`train-v3` had 68 prompts and 226 examples. An exact connector-surface
overlap was then removed on TRAIN (the attic documentation switched from
“saves” to “stores”), yielding `train-v4` with the same counts. The earlier
`held-v2` packet is development only. With the final TRAIN packet it had
two-class support for 6 of 12 early and 9 of 12 late gold descriptors, and
zero of 24 rows had two-class support for every enumerated candidate.
These counts precede distance and veto decisions.

The final TRAIN froze before a new holdout was authored. Its independent D11
staging reproduced 226 records, including 68 positives and 158 negatives.
The exact packet SHA-256 is
`dfcd22f0f4f75c6a0298c6b2f1363562d189b58508911b39d18877f41506bd9c`.
The fit binding receipt is
`c25a27254249e3f31e33ab0bf1f0133bc3464aa03d480fec72c5cd0cbdd5166a`.
`TRAIN-FROZEN.sha256` is the complete local TRAIN and fit input file manifest.

Run the external held compiler after the TRAIN freeze. The external lane has
distinct ledger framing, a question ending at the requested name, and
agreement controls that explicitly ask for one shared complete wording. Its
fields must be distinct from the embedded TRAIN and development authored
fields. Pass a separately authored controls file to include new agreement,
conflict, and absent-person cases. The base assigns candidate-independent row
identities. The external controls begin at base plus 80.

```sh
d28-onset-panel held TOKENIZER NEW_HELD_DIRECTORY CELLS_TSV 28200 [CONTROLS_TSV]
```

Cell columns are family, person, short connector, long connector, first item,
second item. Control columns are stratum, asserted person, requested person,
first connector, first item, second connector, second item. Both use literal
tab separators.
