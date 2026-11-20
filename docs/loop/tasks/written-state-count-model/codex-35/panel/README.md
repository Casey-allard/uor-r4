# D35 fresh value-onset panel

The reference split has 144 supervised prompts across six TRAIN families and
246 fresh prompts across 13 held families. The held set contains 216 crossed
prompts, nine missing-fact controls, nine conflicting-fact controls, and four
causal triples. Each triple changes the earlier context while preserving the
assertion and question, with early-onset, late-onset and underdetermined members.
Groups are scenario families. Within each crossed family, factors are requested
entity, fact order, query/exclusion order, and modifier. Modifier levels are
bare, “only” and “also”. The complete object begins after the modifier. Missing,
conflicting and underdetermined inputs have no supported answer. The first two
also lack a unique supplied fact. The underdetermined input retains its one
available assertion in the supplied diagnostic.

The fixture author marks exact half-open raw user-text byte spans as the source
is assembled. The generator independently maps those edges to original
protocol-2 token positions, checks complete person-head identity, and checks the
expected tape against the assistant framing and EOS. Admission to the existing
bounded writer depends on the real frozen candidate iterator. Natural values
may exceed the writer's seven-token completed-span limit. Keep every held row
and report unreachable onsets separately from ranking errors.

Build both tools on Wick from this directory. Generate and score with a private
umask. The generator has the fixture templates. The separate capture binary
never loads a held key. The native process receives ordinary raw text and only
receives query/fact head spans for the labelled supplied diagnostic.

```bash
cargo build --locked --release --manifest-path Cargo.toml
umask 077
d35-panel-freeze "$TOKENIZER" "$FROZEN"
d35-panel project-train "$CHAT" "$D26" "$TOKENIZER" "$FROZEN" "$GO_TRAIN_PROJECT" "$TRAIN_PROJECTION"
d35-panel project-held "$CHAT" "$D26" "$TOKENIZER" "$FROZEN" "$TRAIN_PROJECTION" "$GO_PROJECT_HELD" "$SPLIT"
```

The frozen tree contains:

- `train/supplied.jsonl` with text and supplied query/fact head spans, and
  `train/labels.json` with supervised original source bounds.
- `held/ordinary.jsonl` with row number and whole user text. The diagnostic
  `held/supplied.jsonl` adds query/fact head spans where applicable. The
  held projection produces 228 supplied rows. Every ordinary row remains in
  the index, including empty pools.
- `offline/key.json` with original IDs, source, byte and token bounds,
  group, stratum, and expected complete tape through EOS where supported.
  Provide it exclusively to the offline grader.

The TRAIN packet admits every row with the exact original gold source range and
at least one eligible rival. Both geometries keep the same candidates, fences,
local features and row order. The independent TRAIN projection recorded 36
admitted rows and 72 ordered pairs per geometry. Four original TRAIN families
failed the admission rule entirely. The model-railway family supplied 24
admitted rows and the conservation family supplied 12. The review should treat
these as two effective TRAIN families, with all three modifier levels. Each
geometry retains the complete projection and omission records.
`fact-prefix` and `candidate-prefix` have the device packet names `train.bin`,
`ordinary.bin` and `diagnostic.bin`, plus manifests and zero-based packet
indexes. Projection skips generation and checks for zero generated IDs.

Once the corresponding on-host receipts are issued, collect two independent
native passes. Both modes use the same executable and D26 reader. Each capture
root has `ordinary` and `supplied`, with `baseline`, `fact_true`,
`fact_reversed`, `candidate_true` and `candidate_reversed` folders.
The runner uses up to six concurrent child processes per invocation and
writes each row exactly once. Baseline can run separately while the rank banks
are fit and packed.

```bash
d35-panel baseline "$CHAT" "$D26" "$TOKENIZER" "$FROZEN/held" "$GO_BASELINE" "$BASELINE"
d35-panel capture "$CHAT" "$D26" "$FACT_D35" "$CANDIDATE_D35" "$TOKENIZER" \
  "$FROZEN/held" "$GO_SERVE" "$CAPTURE" "$BASELINE"
```

For a full pass without a separate baseline receipt, omit the trailing baseline
root. Capture transcripts preserve the exact selected source, original input,
generation and writer trace. `captures.sha256` follows deterministic mode, arm and
row order. Archive partial output in its original root if an invocation fails.

Receipts have `decision: "GO"`, `host: "wick"`, the named phase, and the native
executable SHA-256 in `native_sha256`. `train-project` binds
`train_manifest_sha256`; `project-held` binds both `train_manifest_sha256`
and `held_manifest_sha256`. `baseline` binds `held_manifest_sha256`.
`serve` binds `held_manifest_sha256`, `fact_carrier_sha256`, and
`candidate_carrier_sha256`. If reusing an independent baseline pass, include
`baseline_manifest_sha256` for that pass's `captures.sha256`.
