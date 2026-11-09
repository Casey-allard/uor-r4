# ORDER: one-model-end-to-end, deepseek-265 (real carry geometry diagnosis)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ea3398701`.

## Goal

Determine why the trained readout carry changes later ranks but not the first asked-value choice,
without changing the registered claude-339 outcome or creating work for Claude.

The unit has two bounded questions:

1. In TRAIN d3mwcap6, how far are identifiable question rows from their first answer rows and
   scored value rows, and how many are reachable by the readout carry's 32 rows?
2. At the first reply position of the official carried artifacts, what do the ask-head and carry
   contributions do to the asked value's rank versus the exact `-C` control and matched failures?

This is a CPU-only diagnostic and instrument unit. It trains nothing, uses no GPU or pod, moves
no bar, changes no serving arithmetic or weights, and makes no capability claim. Claude's
two-seed outcome and row-4c decision remain authoritative.

## Cancel probe (run first)

1. The official claude-339 carried and `-C` artifacts, reports, panel inputs and pinned tokenizer
   are available locally or from the published read store.
2. TRAIN d3mwcap6 and its manifest are available, and their pinned hashes match.
3. The merged CPU/integer trace facilities can expose the first reply's ask/carry geometry.
4. No open PR owns this branch or the new diagnostic binary.

Record each result in this folder before changing source outside it. If an input is unavailable,
the affected question is `NOT_RUN`; the other question may continue. A missing input is an
instrument result, not a mechanism negative.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-265/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-cpu-train/src/**`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `docs/harness/claims.json`

Anything else is a stop-and-rescope.

## Done when

1. A deterministic Rust census reports TRAIN question-to-first-answer and
   question-to-scored-value distances, quantiles, extrema, the count and fraction at or within
   32 rows, corpus/source coverage, window coverage, and explicit unidentified-row exclusions.
2. The census includes offset or shuffled controls that prove the distance result is not an
   artifact of token proximity alone.
3. A deterministic Rust trace of official first-reply positions reports, per depth, copy/type
   selection, Hamming distances, ask-table response, carry contribution, asked and other ranks,
   the chosen token, and point-key status. Positive, failing and exact `-C` cases are compared.
4. Negative controls remove or shuffle the carry or ask contribution and fail to reproduce the
   claimed first-position effect.
5. A receipt pins inputs, commands, counts, output hashes, exclusions and software identity, and
   the REPORT classifies diagnostics versus capability evidence and reads negatives first.
6. Focused tests, negative-control tests, changed-file rustfmt, strict Clippy, full loop gates,
   policy checks, fresh exact-head review, protected merge and fresh-main verification pass.

## Stop and ask if

Nothing waits on the owner or Claude. Do not touch Claude's branch, pods, runs, leases, sealed
artifacts or decision review. Do not revise the outcome map or claim a mechanism result from an
unidentified question row, a missing control or a non-reproduced artifact.
