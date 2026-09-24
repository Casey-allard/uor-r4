# Current UOR-R4 research state

Updated September 24, 2026. **Pre-alpha; no useful general-language, coding,
frontier, geometric-advantage or full-path energy qualification.**

## Decision and active work

The owner supplied a whole-project stuck-point review and directed warranted
corrections before the pending merge. The [independent assessment](stuck-point-review-response-2026-09-24.md)
agrees with its central learning-method criticism, corrects outdated and
mathematical claims, and establishes [D8](DECISIONS.md#d8--correct-the-training-method-and-reference-ladder).
The [canonical plan](project-track.md) owns the persistent training/reference
ladder. #973 remains active under programme #820; their full acceptance is open.

**Park further A1–A4 local selector tuning.** Preserve its native serving and
exact-memory scaffolds. The next substantive model work is a coherent native
recurrent-memory learner with autodiff language credit, an explicit soft-to-hard
bridge and competitive ordinary controls at meaningful natural-text exposure.
The transformerless integer/table serving goal and D0-b/D4–D6 remain unchanged.

## Latest executed native result: A4

Four matched eight-epoch continuations consume 4,499,104 new token updates and
185.945 seconds of joint fitting. A4 selects the correct source on 2/24 new first
decisions in each arm; both controls select 0/24. All four produce **0/12 complete
correct read-enabled answers**. Loaded A4 retains only 28/167 C120 and 24/167 2I
admitted fit sources despite each eligible local update succeeding immediately.
Development bits/token is C120 A4/control **6.839477/6.799839**, and 2I
**6.798995/6.872785**. There is no model promotion.

The dedicated address coefficients stay fixed, but shared State updates and
causal inputs change actual fine codes. This corrects the original A4 report's
freeze wording. All 112 generation rows and eight full files per arm reproduce
in independent replay. 33 focused checks passed. See the [A4 result](integrated-attention-a4-result-2026-09-24.md)
and [compact evidence](../evidence/integrated-attention-a4-result-2026-09-24.json).

## Reference and next execution

#1014/#1017 weights, trainer-source manifests and #1017 train/dev/tokenizer
identities have been freshly verified against retained manifests. #1014 owns the
historical +2.677393-nat attention-off result; #1017 owns 1.572752 nats/token after
149,995,520 cumulative tokens. These are bounded ordinary-transformer reference
results on another tokenizer/corpus, not new executions or target serving.

The new offline `crates/uor-r4-training` loads the actual #1017 checkpoint and
**passes the pinned numerical/gradient integrity check on CPU and Metal**.
Each compares 131,072 logits over 32 input positions: maximum error is
0.000015259 / 0.000016212, with 32/32 top-one matches. All 56 parameter tensors,
including 18 Q/K/V tensors, have finite nonzero gradients; all three selected
finite differences pass and restoration error is zero. No optimizer step occurs.
[Source, hashes, device and results](../evidence/reference-autodiff-integrity-2026-09-24.json)
match the [pinned evaluator input manifest](reference-evaluator-v1.json), checked
independently. Optimized Rust compilation and independent source review passed.

**Next within rung 0:** complete the same-token reference/n-gram/cache baseline
table and generation replay, then implement the [specified recurrent learning
graph](project-track.md#rung-1-computation-graph-and-entry-gate). The native student,
soft-to-hard bridge and 600-cell diagnostic are not run. The 32-token training
prefix loss and integrity timing are not population quality or training throughput.

## Delivery, resources and unresolved limits

This work extends protected [PR #1387](https://github.com/UOR-Foundation/uor-r4/pull/1387).
The owner explicitly requested its merge when complete. Source, checks and
results are recorded here; live GitHub and the owning issue receipts establish
the actual protected merge and reviewed-tree equality.
The [A4 budget](../evidence/integrated-attention-a4-budget-2026-09-24.json) and
[review-correction supplement](../evidence/stuck-point-review-budget-2026-09-24.json)
retain cumulative accounting, physical reserve and stop margin. The
[cycle closeout](../evidence/stuck-point-review-closeout-2026-09-24.json) separates
actual fit/integrity costs from the full engineering cycle. No paid compute
or unique-artifact deletion is authorized or used.

The old #1017 revealed test is a fixed regression set, not a fresh final holdout.
Persistent sessions still need exact posting membership preserved through
saturated-page eviction/restore. Final integer export, geometry attribution,
useful complete outputs, D5 full-path access and physical energy remain gates.

## History and authority

The complete previous 4,324-line state record is preserved in the
[dated archive](current-state-archive-through-2026-09-24.md), with all original
relative evidence links. Read scoped history as needed. Start routine work from
this page, the [plan](project-track.md), [decisions](DECISIONS.md), and the exact
source/artifacts for the active rung; do not restart a whole-project survey.
