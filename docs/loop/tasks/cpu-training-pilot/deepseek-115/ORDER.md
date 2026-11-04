# ORDER: cpu-training-pilot, deepseek-115 (repair termination-gate EOS boundary)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `4cc080c6f4b256f36e17468ec2aa7c7b284ff464`.

## Finished before this repair

- `1dd53c193` introduced the training-derived termination-point gate,
  matched nonterminal control, finite tests and driver wiring.
- `4cc080c6f` recorded fresh-context non-author FIT for source head
  `1dd53c193`.
- The registered seed-1 attempt refused before gate evaluation because the
  selected training replies end in corpus EOS id 1, not decoder BOS id 0. No
  gate or candidate result was produced.

## Goal

Repair the instrument boundary and repeat the deepseek-114 experiment. Corpus
replies carry EOS id 1. The final reply-content query therefore predicts EOS;
the gate remaps an accepted EOS decision to the decoder's BOS stop sentinel.
Normalize a trailing corpus EOS from references while retaining trailing-BOS
compatibility for synthetic callers.

This is a pre-result instrument repair. It does not change the gate mechanism,
selection, training replay, arms, cap, random control, radius rule or candidate
bar.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-termination-gate-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-114/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-115/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Required changes

- Require terminal training replies to end in corpus EOS id 1.
- Derive terminal queries from the final content row before EOS.
- Normalize one trailing corpus EOS from complete-reply references, retaining
  old trailing-BOS compatibility.
- Add finite tests for EOS endpoint derivation and EOS reference normalization.
- Record the failed seed-1 attempt as an instrument refusal and discard its
  partial artifacts from evidence.
- Obtain fresh FIT at the repaired source head before rerunning seed 1.

## Done when

1. All deepseek-114 requirements otherwise remain unchanged.
2. The corrected model-free reference census is reported.
3. Both seeds and a fresh seed-1 repeat pass the FIT-reviewed commands.
4. The REPORT records the refusal, repair, gate diagnostics, three-arm results,
   checks and exact candidate verdict.
5. Tests, Clippy, formatting, evidence, loop gates, fresh exact-head review,
   protected merge and fresh-main verification pass.

## Stop

Stop on evaluation leakage, post-result tuning, artifact or no-gate digest
drift, an EOS/BOS ambiguity not covered above, nondeterminism, counter overflow
or a repaired source change outside this order.
