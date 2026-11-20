# D27 assertion-selection panel

`cells.tsv` defines six independent crossed cells. In each cell, the requested
relation, assertion order, assignment of the two objects to the two relations,
and presence of a trailing refusal instruction vary independently. Nina and
Owen each appear in an exact wording cell and a lexical transfer cell. For
transfer, the assertion verb differs from its query counterpart. TRAIN has
two other people and both pairs of overlap relation words. The transfer verbs
are absent from TRAIN. These transfer rows use the same sentence frame as
overlap, so the contrast targets lexical transfer within that frame.

The 32 TRAIN rows comprise two 16-row cells. The 92 held rows comprise 32
overlap, 32 lexical transfer, and 28 controls. The controls cover duplicate
agreement, duplicate conflict, absent relation, a relation word in the wrong
assertion's value, competing person, relation carried only by the competing
person, and explicit exclusion. Each control has a bare version and a version
with the same question followed by a refusal instruction.

Run from the repository root. `TOKENIZER` is the same protocol-2 vocabulary as
D26, with SHA-256 `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.
The destination must be new.

```sh
PANEL=docs/loop/tasks/written-state-count-model/codex-27/panel
CARGO_TARGET_DIR="$Q/panel-target" cargo build --locked --manifest-path "$PANEL/Cargo.toml"
"$Q/panel-target/debug/d27-relation-panel" "$TOKENIZER" "$Q/panel-02"
```

For blind language adjudication, show only `held.blind.tsv`, with one row ID and
unannotated user message per line. Input rows use the same IDs in every file.
Every address is a zero-based half-open span on the original BOS-inclusive BPE
tape. The extractor rejects a span that would split a token.

| Output suffix | Fields and use |
| --- | --- |
| `.blind.tsv` | Row ID and unannotated user message for human reading and ordinary client capture. |
| `.blind.jsonl` | `row`, `ids`. Original protocol-2 input including the assistant prefix. |
| `.framing.jsonl` | `row`, `content`, `query`. Supplied user-content and question spans. The question stops at its question mark. |
| `.person.jsonl` | `row`, `requested`, `excluded`. Supplied query-person spans, with null exclusion where absent. |
| `.relation.jsonl` | `row`, `relation`. Supplied query relation-word span. It identifies a word in the question itself, including transfer paraphrases. |
| `.gold.jsonl` | Host-only key with stratum, factorial coordinates, instruction flag, original assertion spans, supported assertion indices, value spans and expected tape. |

The factorial coordinates are cell, requested relation index, assertion order
and object exchange. The separate `instructed` boolean completes the crossed
design. `selected` lists zero-based assertion indices, including all compatible
and all conflicting duplicates, independently of the final release decision.
A native source decision must release no span when the requested relation is
absent or when supporting assertions conflict. An exact refusal tape is scored
only where the prompt requests one and its supported values are absent or in
conflict. On bare unsupported rows, `expected_tape` is null and the no-source
criterion still applies. On supported rows, the expected tape consists of the
original assertion's object IDs followed by EOS. Duplicate agreement requires
the object IDs to match exactly across the two original addresses.

The ordinary path receives the unannotated message or original IDs only.
Supplied framing, person and query-relation spans are intervention inputs.
The assertion and object addresses never enter a blind prediction packet.
This preparer constructs fixtures and references without loading a model or
executing a selector or writer.
