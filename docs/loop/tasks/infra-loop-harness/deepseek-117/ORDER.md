# ORDER: infra-loop-harness, deepseek-117 (calibrate the cycle probe)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `8f2414f2e`.
Setup row: 2 (record maintenance and model-contract tooling).

## Goal

Calibrate the arm-r cycle fixture against the real sealed artifact. The first
artifact audit showed that an eight-token budget stopped at the cap before the
cycle rule could act. Use prompts that exercise one exact short cycle and one
ordinary terminal stop at a twelve-token budget, update the contract hash and
regenerate the views.

## Cancel probe

At the base, the cycle fixture expects `short_cycle` at eight tokens for
prompts that finish or hit the cap instead. The unit stands.

## Files in scope

- `conformance/stack/**`
- `conformance/generated/**`
- `docs/model/STACK-CONTRACT.md`
- `docs/harness/gates.json`
- `docs/loop/tasks/infra-loop-harness/deepseek-117/**`

## Done when

1. The first fixture case produces the exact `short_cycle` stop on arm r.
2. The second records any declared terminal stop and still rejects unreported
   exhaustion.
3. The contract hash and generated views are current.
4. A real arm-r audit passes all representation and probe checks; the arm-rt
   audit may retain its known copied-value probe negative.
5. Gates, report, exact-head review, CI, merge and fresh-main verification pass.

## Stop and report

Stop if calibration changes serving code, an artifact, a model result or the
meaning of the cycle rule. This unit changes only the probe and its identity.
