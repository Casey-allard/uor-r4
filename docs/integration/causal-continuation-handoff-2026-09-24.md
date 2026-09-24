# Causal continuation handoff: September 24, 2026 UTC

Read [the executed result](causal-continuation-result-2026-09-24.md), [machine evidence](../evidence/causal-continuation-summary-2026-09-24.json), [verification](../evidence/causal-continuation-verification-2026-09-24.json), and [the original plan](causal-continuation-plan-2026-09-24.md). The terminal geometric/transformerless objective and exact ownership contract are unchanged. No global model or API default was promoted.

## Retained state

- `.uor-models/causal-continuation-2026-09-24/causal-candidate.tlx`: `1ac700658a476e356ff64db95003277f30c27b3d181ca183444dd7621c5d955f`. Measured causal16/16, gradient-withheld4/4, temporal32/32, training36/36, class4/4, historical held-out3/3. Repository loss6.343260311; not the best repository-only prose model.
- `ordinary-control.tlx`: `8934cac3c5698d5491c1e2217929b77eae6457d12fd87866fd0d7c63bed74996`. Same broader data/schedule, causal8/16 and withheld0/4; slightly better text loss.
- `final.tlk`, `checkpoint-128.tlk`, `control-128.json`, `control-256.json`, `model-256.tlx`: exact optimizer/sampler/validation continuation and reference model. The final checkpoint's SHA-256 is `673c551f32c8c2786aa7127a6ea3ccfbe4f0aa5219a25c3f572c39ad07f20f94`.
- `relative.q8l`, `registry.json`, protocol files and verified manifest: finite geometric learner and source bindings. Not a lexical serving artifact.

All11 full report roots, frozen executable, source inputs, compiler failures, test identities and analysis are under `/Users/casey.allard/uor-r4-investigations/causal-20260924T000007`. The prior session's artifacts remain in their original directory. A Git clone alone does not contain these ignored payloads.

## New interfaces

`TlTrainer::checkpoint` and `from_checkpoint` serialize/reload latent parameters, both Adam moments, optimizer configuration/step and a bound `TrainingCursor`. `.tlx` is still only a served model. `from_model_fresh_optimizer` still means new optimization, not exact resume.

`TlTrainer::train_intervened_batch` is explicit training-only copied-feedback supervision. Selected evidence/facts stay fixed. Its presence must never be interpreted as changing what runtime Copy may emit.

`TlReadPlan::compile` plus `score_into` provides exact sparse readout into caller-owned buffers. Only successful readout calls are measured allocation-free. Original model and plan coexist; prepare once per loaded model, not per token.

`relative_action_learning` learns Q8 actions from typed relation/query/key supervision. `relative-control-audit` supplies the strengthened ordinary control that matches it. It is not a learned language-attention path.

`UOR_PRINCIPAL_RUN=causal` enters the new runner; `UOR_CAUSAL_MODE` is `train`, `evaluate`, `cost` or `geometry`. `train` additionally uses `UOR_CAUSAL_DATA`, `UOR_CAUSAL_ARM=ordinary|intervened`, `UOR_CAUSAL_STEPS` and optional `UOR_CAUSAL_CHECKPOINT`. A new exclusive `--state-probe` root, parent `.tlx`, preserved repository `--docs`, tokenizer and source revision are required.

## Exact replay of the completed experiment

The following resumes batch128 to256 using the original source-bound executable. Choose a new root name; existing roots are deliberately rejected.

```sh
R="$HOME/uor-r4-investigations/causal-20260924T000007"
D="$HOME/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs"
M="$HOME/uor-r4/.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx"
UOR_PRINCIPAL_RUN=causal UOR_CAUSAL_MODE=train \
UOR_CAUSAL_ARM=intervened UOR_CAUSAL_DATA="$R/data" \
UOR_CAUSAL_STEPS=256 \
UOR_CAUSAL_CHECKPOINT="$R/intervened-1/checkpoint-128.tlk" \
RAYON_NUM_THREADS=1 "$R/causal-lexical" \
  --state-probe "$R/intervened-replay-NEW" --artifact "$M" --docs "$D" \
  --source-rev 4af04cf56f213df40b78c73a7c51ed5274517857
```

The recorded replay matched `final.tlk`, `final.tlx`, `selected.tlx`, validation history and generated outputs byte-for-byte. The gradient source and harness/data identities are bound: editing their implementation changes the experiment. To begin a longer/new schedule, define and bind a new phase rather than changing256 to512 and calling it the same resume.

## Known limits and next decisions

The four gradient-withheld feedback cells were still an exposed selection gate. Test new payloads/spans and whole scoped sessions before a broad causal qualification. The historical32/4/3 panels do not qualify arbitrary memory, and the no-Copy-event control remains insensitive. No new default Studio/API or full composite-session acceptance is asserted.

Final source files pg74,pg219 and the Rust Book ownership chapter are now exposed. New final evaluation requires a newly frozen source set. Retain the three-source gain and the causal arm's small loss penalty versus ordinary training together. All four model-generated Rust outputs failed compilation; the supplied compiler control must never be counted as coding ability.

The Q8 learner and repaired ordinary signed-permutation comparator both score1,024/1,024. Do not resurrect the weak ordinary864/1,024 result as evidence of geometric superiority. Learn language-conditioned relative operations next, preserving exact selected payload identities and measuring competing source choice.

The cost result is optional compiled sparse readout on the same model, about10.4x faster for tested pre-tokenized prompts and generations. It is not full application latency, reduced model size, or energy. Preserve logical plan payload versus allocated capacity distinctions. Resource and preservation records are linked from the result; the model store and all unique experiments remain local.
