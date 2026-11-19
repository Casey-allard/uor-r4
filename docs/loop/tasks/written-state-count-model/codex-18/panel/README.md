# Factorial held panel

Both pinned seed tokenizers reproduce the same 42 ordinary user questions. The
panel has eight matched triples, 12 supported coverage controls, and six
abstention controls. Every matched group uses identical nonlexical observations
and reverses the correct source start across all three positions. All 36
supported original spans can statically copy their complete literal through
EOS. The positional ceiling is 8/24 on the matched subset and 20/36 on all
supported rows. All candidates on all 36 supported rows have the required class
support in the frozen 200-record TRAIN bank. These checks consume no fitted
distances or model predictions.

The tokenizer-only generator follows
[the codex-17 v5 panel](../../codex-17/panel-v5/README.md). Its held roots contain
separate grades and anonymous candidate shapes. Give independent readers only
`public-text.json`. Give operators the original prompts and independently
sealed requested-role endpoints. Keep expected answers and source labels with
the adjudicators.

## Reproduction

Build the panel Cargo manifest with `--offline --locked`, then invoke the
binary as `value-start-factorial-panel SEED TOKENIZER NEW_ROOT` for each seed.
Use the tokenizer SHA-256 pins in the generator. Each output root contains
`held/` with the original protocol-2 token prefix, question bounds, candidate
geometry signatures and literal labels. A replay of both seeds matched all five
files per seed byte for byte. `INPUTS.sha256` covers the generated inputs and
the preflight audit files relative to the local panel root.

Run the earlier `panel-v5/ceiling.jq` on each held `candidates.json`. Run
`shapes.jq` on the same input, then give the anonymous shapes to the earlier
`train-supplement/coverage.jq` alongside the frozen base and supplemental
TRAIN candidates and grades. The preflight quotes both seed counts in
`summary.json`. Read the archive comparison inputs pinned by
`history-files.sha256` and derive the unique JSON strings with
`jq -nc '[inputs | .. | strings] | unique'`. Pass them as `history` to
`novelty.jq`, with held public text as `raw`, held grades as `grades` and
the frozen base and supplemental TRAIN grades as `base` and `supplement`.

`adjudicate.jq` compares two separately supplied reader files against both
tokenizers' authored readings. The reader schema is
`{index, person, answer, status}`. Its report retains any disagreement. The
[codex-17 requested-name sealer](../../codex-17/endpoints/README.md) can reuse
the frozen base and supplement input files and accept the new blind held raw
text plus person-only projections from the readers.
