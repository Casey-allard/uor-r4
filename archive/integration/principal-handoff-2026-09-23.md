# Principal handoff: September 23, 2026

> **Continuation update, September24 UTC:** read [the causal continuation handoff](causal-continuation-handoff-2026-09-24.md) before repeating work. Exact optimizer resume and the bounded feedback repair are now executed; generated language/coding and broader session qualification remain open. Earlier artifacts and their evidence below are preserved.

## Start here
Read [the executed result](principal-continuation-result-2026-09-23.md), [the machine evidence](../evidence/principal-continuation-summary-2026-09-23.json), and [the Hamilton-vector interpretation](hamilton-vector-interpretation-2026-09-23.md). The terminal objective is unchanged: useful, fully transformerless geometric language modeling, with geometry replacing floating-point matrix multiplication in serving. A count table is a comparator/component, not the final architecture.

## Retained state
The new **ordinary-lexical research candidate** is `.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx`, SHA-256 `69e8b88b41bb09d9149be1ac7e83e5e50db2974f470dce05405a55cc1f0bfb7e`. It scores 6.27370696 development bits/target and retains the authored 32/36/4/3 panels. The constant-rate candidates lose one of three held-out examples; they are preserved negatives, not alternatives to promote by prose score alone. A second shuffle seed confirms the decaying recipe. This is not a complete scoped-session/Studio/API qualification.

Two `PRC2` bundles are retained beside it: `prose-ratio-candidate.prc` and `grounded-warm-ratio-candidate.prc`. The latter scores 5.01773800 development bits/target while retaining the same authored grounding panels. Both use frozen alpha28/32 and gamma8/32 relative to the Fit unigram. The primary coefficient choice used 114,364 Tune targets; interpolation was independently rechecked on the same targets. The old 8,512-target study is not substituted for that population.

The retained root has `selection.json`, `registry.json` and a verified report manifest. Full reports, executables, source inputs and restoration archives are in `/Users/casey.allard/uor-r4-investigations/principal-20260923T214312`. The model/data store is ignored by Git: a fresh clone alone is not the full runnable research state.

## Implemented interfaces
- `learner/lexical_residual.rs`: bounded sparse `IPL1` prior and integer Q12 action-mass-preserving composition, including marginal-relative correction. It allocates; no zero-allocation or whole-binary multiplier certificate is claimed.
- `TlTrainer::from_model_fresh_optimizer`: exact served-model reconstruction with reset Adam state. It is not resumable optimizer checkpoint loading.
- `learner/hamilton_transport.rs`: exact signed Q8 vector actions and query-relative value transport. It does not claim a learned attention model or semantic advantage.
- `bin/support/principal_continuation.rs`: full-Tune explanatory grids, integer realization, depth probes, explicit warm schedules/seeds, per-document rescoring, source-family evaluation, actual grounded rollouts and warm readout/update cost.

The ordinary runner enters these modes only when `UOR_PRINCIPAL_RUN` is explicitly set. Supported modes: `adjudicate`, `count-calibration`, `realize`, `depth`, `warm`, `score`, `fresh`, `grounded`, `cost`. Use a **new** `--state-probe` root, a `.tlx` `--artifact`, the preserved `--docs` inputs and the pinned tokenizer. `fresh`/`grounded`/`cost` also require `UOR_PRINCIPAL_BUNDLE`; fresh requires `UOR_PRINCIPAL_FRESH_DIR`. Ratio adjudication is explicit with `UOR_PRINCIPAL_RATIO=1`. Source report manifests are checked before transferred/fresh evaluation.

`UOR_PRINCIPAL_WARM_SEED` selects a reproducible fresh-optimizer fit-window shuffle. The two tested values are 20260923 and 20260924. The warm plan is 128 updates, four prose and two grounded examples per update, ground weight4, constant0.01 versus linear0.02-to0.002. Do not describe a `.tlx` reload as continuation of the old latent masters/moments.

## Still open
Free continuations remain repetitive; the four external books have poor absolute losses. Native post-copy decisions still follow the source fingerprint rather than isolated emitted-token feedback. The composite's corresponding crossed intervention and the full pre-existing scoped-session campaign were not run. A reversed-history ratio control ties the full-history ratio after tuning, so no unique order/H4/Hamilton advantage is established. Do not replace these qualifications with a generic PASS.

The primary source evaluation used four downloaded Project Gutenberg books only after model/coefficients were frozen. These sources are **now exposed**. The later warm-composite evaluation is a follow-up on that same data, not another fresh test. All recorded generation outputs are published with the evidence, including failures.

## Reproduce a native score without training

From an isolated checkout of the delivered source, use the existing release cache and a new report directory:

```sh
export CARGO_TARGET_DIR="$HOME/uor-r4-kimi/target"
CARGO_BUILD_JOBS=1 cargo build --release --offline -p uor-r4-core --bin ordinary-lexical
UOR_PRINCIPAL_RUN=score RAYON_NUM_THREADS=2 \
  "$CARGO_TARGET_DIR/release/ordinary-lexical" \
  --state-probe "$HOME/uor-r4-investigations/new-score-unique-root" \
  --artifact "$HOME/uor-r4/.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx" \
  --docs "$HOME/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs" \
  --source-rev "$(git rev-parse HEAD)"
```

The report root must not exist. Source-separated evaluation must not use the four now-exposed books as another blind test. The archived experiment-source branch is `archive/principal-evidence-source-20260923`; rebase mapping is recorded because the later owner-direction merge changed source commit identities without changing the tested Rust diff.

## Five independent next workstreams
- Broaden source-separated prose/dialogue/coding training without losing exact owned-memory behavior.
- Repair the isolated emitted-token-feedback failure and test the same loaded composite through the full scoped-session lifecycle.
- Learn query-relative geometric vector actions against equally informed ordinary controls, separating fixed-coordinate changes from useful attention.
- Preserve complete latent optimizer checkpoints and decompose the successful warm recipe into controlled initialization, objective, schedule and data-exposure effects.
- Reduce readout parameter traffic and allocations, then measure whole-task quality-matched latency and energy on the same integrated model.
