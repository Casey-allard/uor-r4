# D21 paired post-run readout

The audit verifies each consumed file against the original result manifests
and checks the bindings to the sealed labels, blind prefixes and tokenizer.
For every supported prompt it
reconstructs the expected response from the labelled original prefix slice
with EOS appended. It compares GPU writer output and independent observation
records, recomputes exactness and the first mismatched output token, and
checks the writer source against the decoded value trace. No model runs at
this step.

Build the tokenizer piece viewer using this standalone Cargo manifest with
`cargo build --offline --locked`. Invoke the readout once for each completed
seed:

```bash
readout.sh RUN_DIR SEALED_PANEL TOKENIZER PIECES_BIN NEW_OUT
```

The resulting Markdown table lists all 96 supported prompts and all six
unsupported controls. The machine-readable `readout.json` retains each arm,
aggregate and factorial-cell comparison. Prompt ordinals, source offsets,
and vocabulary IDs are recorded in distinct fields. The token piece object
retains lossless bytes even when a token alone is invalid UTF-8. A mismatch
has a zero-based index and either token can be null when one sequence
ends before the other.

The paired classifications distinguish retained correctness, recovery,
regression, and two incorrect results. Two incorrect emissions with
different token tapes count separately. Source kinds distinguish a correct
value, another person's literal, suffixes, connectors and null sources.
An earlier unseen gold row has an unseen ordered candidate at the correct
source with a later candidate present. A wrong release in that subset also
requires a new wrong source and a changed unseen candidate status. The
compact JSON keeps aggregate and paired changes. The full readout retains
every prompt and decoded source trace.

Once both seed readouts exist, use
`jq -s -f compare-seeds.jq SEED1/readout.json SEED2/readout.json` to count
which prompts reproduce each paired outcome across the two frozen seeds.
The comparison keeps a denominator of 96 supported prompts per seed.
