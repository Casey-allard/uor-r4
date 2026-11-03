# ORDER: apply the registered served-comparison result

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `75086c1d3`.

## Goal

Apply the bar in
[`one-model-end-to-end-served-comparison-2026-11-02.md`](../../../../harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md)
to the four runs Claude completed at `36ca53c81`, update the deepseek-5 record, and state whether training as served is ready for a longer language run.

This is a decision/record unit. It does not rerun training or alter model mechanics.

## Cancel probe

1. **Are the four runs absent?** No. Claude posted finite losses, checkpoint rounding, served-evaluator scores, nearest hits, openings and decode throughput, plus the sealed store object SHA-256.
2. **Can this host independently download the private object?** No Hugging Face credential is configured. The decision records that limitation and uses the sealed handoff summary rather than claiming a local readback.
3. **Is the bar already applied in the repository?** The deepseek-5 report still says `NOT_RUN`, and the sheet has no result section. Not cancelled.

## Files in scope

- `docs/harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-5/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-10/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/harness/claims.json`

## Done when

1. The instrument door, operational door and quality smoke are applied exactly as written.
2. The report states the sealed handoff SHA-256 and the local inability to re-download it.
3. No longer training/language run is recommended while the operational door fails.
4. The 49-pass activation-scale loop is named only as an unmeasured cost hypothesis.
5. The notebook and roadmap reflect the result without changing milestone status.
