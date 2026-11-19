# D26 source-to-writer replay

The diagnostic runs the real exact-copy cursor from each sealed device selection
on its original prompt tape. The ordinary reference and read-disabled captures
come from the unmodified D21 assertion-agreement parent. The grader independently
verifies the captured role, source, counts, emitted IDs, all cursor words, text
and stopping reason. The verified no-source writer follows its own original
integer predictions until EOS or the 16-token cap. GPU-selected source
interventions and ordinary client results appear as separate fields.

Run from the repository root with the original D21 parent in `PARENT`, its
tokenizer in `TOKENIZER`, and the sealed scratch directory in `Q`.

```sh
WRITER=docs/loop/tasks/written-state-count-model/codex-26/writer
CARGO_TARGET_DIR="$Q/target-views" cargo build --locked --release --manifest-path "$WRITER/Cargo.toml"
GRADE="$Q/target-views/release/d26-source-writer"
"$GRADE" "$PARENT" "$TOKENIZER" "$Q/panel" "$Q/score-stage" "$Q/score-01" "$Q/ordinary" > "$Q/writer-grade.json"
jq -f "$WRITER/summary.jq" "$Q/writer-grade.json" > "$Q/writer-summary.json"
```

The grader checks the cold bank and scores byte for byte. It compares every
GPU score to the Rust source bank's signed L1 distances from the staged
candidate, including masked sentinels, and independently checks the selection
rule. Swapped input words must exactly reproduce the nominated donor context
while leaving address, eligibility and local coordinates intact. The held
receipt also compares local features by relative candidate position within
each pair. The wide-local results and all distances must agree across views
and their donor runs.

A forced gold or opposing position must reproduce its locked original IDs
through EOS. The receipt includes both IDs at the first mismatch and the
zero-based offset, including wrong EOS and missing IDs. On unsupported rows
the expected tape is the tokenizer's canonical leading-space refusal through
EOS. The grader separately records source absence, unsupported source release,
and the decoded literal refusal through EOS.

## Loaded opt-in recurrence carrier

The carrier mode verifies the artifact's identity bindings and compares
the loaded native candidate masks, exact original positions, signed local
and recurrence words, selected address, reason, distances, margin, and
writer fence to the sealed measurement. It records the supplied all-assertion
result and ordinary client decision in addition to isolated-target replay.

```sh
"$GRADE" carrier "$CARRIER" "$TOKENIZER" "$Q/panel" "$Q/score-stage" "$Q/score-01" \
  "$Q/ordinary/recurrence" > "$Q/writer-carrier.json"
```

The final capture-directory argument is optional for native replay only.
With it, provide one fresh protocol-2 ordinary capture per row as
`ROW.json`, using original user text, read enabled, temperature 0 and a 16-token
cap. The supplied-target measurements and candidate parity provide no
ordinary model gain without matching ordinary transcript evidence.
