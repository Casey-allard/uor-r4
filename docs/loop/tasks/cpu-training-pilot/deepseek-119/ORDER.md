# ORDER: cpu-training-pilot, deepseek-119 (writer-cell termination gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `2833c451e`.

## Goal

Test a writer-cell termination rule after the single-radius point gate closed
negative. For every teacher-forced training query, derive the writer's top
admission cell from its nearest first and last orbit-root scores. Count
terminal and nonterminal
training rows per cell and learn only cells with strictly more terminal than
nonterminal evidence. At decode, a generated query in a learned cell names the
BOS stop sentinel.

Compare no gate, the learned cell set and an equal-size deterministic shuffled
cell control on both public seeds. This changes the termination decision rule,
not the failed point radius.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-cell-termination-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-119/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Registered mechanism

- The writer cell is the top-ranked of 256 first-root / last-root admission
  cells under the writer's exact score and tie order.
- Corpus EOS id 1 identifies the terminal row; prompt-final and earlier
  reply-content rows are nonterminal.
- This is a cell-local terminal-majority rule, not a class-normalized rate: a
  cell is learned only when its terminal count is greater than its
  nonterminal count. Ties and nonterminal-majority cells are excluded.
- The matched control has the same number of cells, sampled deterministically
  without replacement from observed cells outside the learned set using the
  model seed.
- An accepted generated query names decoder BOS id 0.
- Candidate success counts `bos_complete - premature_stop` and requires a
  strict gain over both no-gate and random controls on both seeds with no
  exact-reference loss.

## Done when

1. Gate derivation uses training examples only and deterministically records
   cell counts, selected cells, excluded ties/losses and control identity.
2. Finite tests cover cell derivation, local-majority selection, deterministic
   shuffled control, exact cell override, no override outside the set,
   premature-stop accounting, real decode stop behavior and no-gate digest
   preservation.
3. Fresh FIT pins the deepseek-109 trained artifacts, deepseek-110 no-gate
   digests, selection, arms, bar, cost and stop.
4. Both public seeds and a fresh seed-1 repeat run under D84 with exact
   CPU/serving parity, artifact hashes and ordered reply digests.
5. REPORT compares non-premature BOS, premature stop, exact and reachable-exact
   results and preserves the negative whether or not the candidate advances.
6. Tests, Clippy, formatting, evidence, loop gates, exact-head review,
   protected merge and fresh-main verification pass.

## Stop

Stop on evaluation leakage, post-observation tuning, artifact or no-gate digest
drift, a non-vacuously equal control, nondeterministic cell selection, missing
parity, counter underflow or any edit to Claude-owned long-window files.
