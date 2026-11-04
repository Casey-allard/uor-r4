# ORDER: cpu-training-pilot, deepseek-122 (within-cell orbit termination)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `cb457faa4`.

## Goal

Test the next distinct termination hypothesis after the point-radius and
top-writer-cell majority gates both failed: terminal and ordinary continuation
rows that share a writer cell may differ in their cell-relative orbit phase.

For each Q12 query, retain the top writer cell and a 32-integer signature made
from the 16 exact root L1 costs in each of the first and last 8-coordinate
blocks, with each block's winning cost subtracted from its 16 entries. Train
only from reply endpoints:

- terminal: the final content row before corpus EOS;
- nonterminal: prompt-final and earlier reply-content rows.

Within a shared cell, name decoder BOS only when the query's orbit signature
is strictly closer to a terminal training signature than to any nonterminal
training signature, with the exact integer L1 distance and no threshold tuned
on evaluation. Ties do not stop. Compare against no gate and a deterministic
within-cell class-label permutation that preserves cell, signature multiset
and class counts.

This is a D84 CPU development screen. It cannot qualify serving or language
capability; GPU remains the qualification authority.

## Cancel probe

At the base, `reply_eval.rs` contains the failed absolute point-radius gate and
the top-cell membership gate. `writer_query_cell` returns only the first root
pair, and no producer or evaluator uses all 16 root costs as a cell-relative
signature. The two prior configurations are closed negatives, not the mechanism
proposed here.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-orbit-termination-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-122/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The fixture exports and tests the exact cell-relative signature, exact
   integer distance, strict-terminal stop rule, no-stop tie, and deterministic
   matched label control.
2. The prep sheet is FIT-reviewed before public evaluation and registers a
   paired, noise-bounded non-premature BOS-gain / exact-non-loss bar plus cap,
   context and cost stops.
3. Both public seeds run the trained gate, no gate and matched control over the
   existing 114-prompt / 16-token panel; seed 1 repeats without timing.
4. The fully-served base forward used by every gate arm retains exact
   CPU/integer parity for Q12 queries, tokens and kept cells; the orbit rule is
   a deterministic post-process of those audited Q12 values.
5. Focused and full CPU release tests, strict Clippy, formatting, report,
   roadmap, catalog and loop gates pass.
6. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop on CPU/serving divergence, nondeterminism, more than 2 GiB peak RSS,
training loss substituted for reply evidence, any edit to Claude-owned
float-training files, an evaluation threshold tuned after seeing the panel, or
any seed or repeat above 10 minutes.
