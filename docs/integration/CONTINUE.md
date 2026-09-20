# Continue UOR-R4 Geometric Language Model

Read [AGENTS.md](../../AGENTS.md), owner-adopted [DECISIONS](DECISIONS.md), the [machine policy](agent-execution-policy.json), [canonical plan](project-track.md), newest [current state](current-state.md), and [frozen-prior review](frozen-prior-review-2026-09-20.md).

The complete handoff was [DeepSeek — recover the frozen prior and diagnose its loops](deepseek-frozen-prior-step-2026-09-20.md), and it has now been **executed**: the corrected evaluation-only replay is recorded in the [replay receipt](../evidence/native_geometric_frozen_prior_replay_2026-09-20.txt) and the active [current state](current-state.md). It reproduced the recorded step-512 losses to ~1e-15 bits/target, repaired the occurrence permutation and document aggregation, passed the frozen 0.10-bit threshold on the legacy and spread-position panels, recovered the original exposure (4,096 windows = 257,113 targets), and showed an order-2 count reference on the same input beats the two-token learned predictor (3.9691 vs 7.1425 bits/target).

The next bounded increment is **one learned ordered prefix-state/read channel** with same-tail/different-prefix, state-disabled, order and matched capacity/cost controls, using that count reference as the comparator. Do not retrain this prior to improve the verdict, do not require perfect prior fluency, and keep exact occurrence/version memory distinct from coalescing modulo buckets.

Refresh origin/main, live #820/#973/#963/#964, artifact hashes and shared time/storage receipts. D0-b permits bounded low-bit additive linear maps; geometric routing remains preferred and offline Rust matmul is allowed. No general chat or energy advantage is established. Preserve all three model families, negatives and sealed data. Necessary local allowance extensions remain authorized when projected/recorded before use; no paid compute or deletion.

The [September 7 handoff](handoff-2026-09-07.md), [earlier September 19 handoff](handoff-2026-09-19.md) and [previous real-text training prompt](deepseek-realtext-prior-step-2026-09-20.md) are historical. Their dated next actions do not override the current prompt.
