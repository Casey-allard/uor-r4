# PICKUP: native-space-state-contract, codex-8

- Branch: `codex/q31-integer-serving`
- Head: pending the delivery commit; check `git rev-parse HEAD`
- Task folder: `docs/loop/tasks/native-space-state-contract/codex-8/`

## Open items

- `IntegerStackSession` has not loaded the Q31 state into its real token recurrence.
- The next replay must compare raw transition words, emitted token ids, routes, carry and read
  choices on one fixed prompt.
- Package-wide `uor-r4-integer` formatting and Clippy baselines are red outside this unit.
- The live `.uor-models/native-joint-learning-2026-09-04/model-time.json` ledger is absent.
- PR review, CI, merge and fresh-main verification remain.

## Next step

After this boundary merges, open the next ORDER to add the loaded `Q31ServingSession` to
`IntegerStackSession`'s actual recurrence and prove bit-identical served/CUDA/Q31 words and token
ids. Start with:

```sh
git switch main
git pull --ff-only
rg -n "struct IntegerStackSession|stack_recurrence_core|Q31ServingSession" \
  crates/uor-r4-integer/src/stack
```

## Standing references

- CHARTER §3.7
- D85
- L-009
