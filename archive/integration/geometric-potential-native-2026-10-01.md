# Compiled signed-relative geometric potentials — October 1

**Measured:** source-bound integer geometric score tables preserve all 512 fixed answer decisions, both heads' source-majority decisions and top-ranked occurrence/NoRead choices for the two saved ordered-span models. There are zero score-bound violations and zero training updates. This replaces the direct reader's floating-point seven-family score evaluation in the existing Rust stack. It does not qualify a complete native serving pipeline.

## Mechanism

Content and retained context remain separate signed H4 channels. Each lane computes the directed relatives `inverse(query) compose(source)` through the pinned exact group table. Seven learned potential families are evaluated offline: content unary, context unary, their cross potential, two radial tables and two presence tables. Their entries are rounded once to signed Q24, nearest with ties away from zero; overflow is rejected. Runtime reads the selected entries and adds them in i64. Absolute offsets are preserved, including both-absent presence scores, because comparison with NoRead depends on them. Missingness is distinct from a present identity, antipodes remain distinct, and radius bins retain the existing physical dyadic policy.

These are four-byte score entries, not four-bit linear-map weights. Each model's two heads with four lanes use 149,440 entries / 597,760 bytes, plus a 15,601-byte pinned H4 payload, source/format metadata and tokenizer identity. The tables factor across heads, lanes and seven families; they do not enumerate combinations of entire model state. Angular tables derive from the actual saved coefficients and signed canonical roots. Loader admission independently recompiles and compares all bytes against the saved source, refusing resealed table tampering or stale live weights.

Input classification still uses the unchanged floating-point encoder. The contextual producer, event controller, F32 reconstruction, age, NoRead, softmax, value/output paths and surrounding trunk remain floating point. Actual predicted events supply the existing exact token dictionary/span register; labels only observe outputs. This is a native scoring substitution, not a transformer runtime conversion hidden inside a lookup or a fully native model.

## Public numerical contract and fixed replay

For L lanes, worst-case table rounding is `Q = 7L / (2 * 2^24)`. The compiler also accounts for F64 evaluation and both final F32 casts using an upward-rounded absolute monomial bound S and declared operation-count gamma bounds. The integer accumulation remains below 2^39; its F64 dyadic reconstruction is exact before the final cast. This is a bound on scores, not a guarantee of ranking margins or downstream answers. No precision tuning, centering or target-dependent quantizer is used.

The unchanged original128 and stress128 panels, both source models and historical outputs are hash-pinned. Execution source: `c69e496502641098588056b1a27dfcfb0600cf99`; executable SHA256: `6b169f57b457a06c5e808068c1cc977f7941793852f7eb68a36323166c58921b`. Controller events and span codes are unchanged; the baseline reproduces all historical answer logits and both heads' source masses bit for bit.

| Saved model | Panel | Retained answers | Retained head0 source majorities | Answer changes | Bound violations |
|---|---|---:|---:|---:|---:|
| GeometricSpan-Ordered-s1 | original | 99/128 | 128/128 | 0 | 0 |
| GeometricSpan-Ordered-s1 | stress | 109/128 | 126/128 | 0 | 0 |
| GeometricSpan-Ordered-s2 | original | 128/128 | 127/128 | 0 | 0 |
| GeometricSpan-Ordered-s2 | stress | 128/128 | 128/128 | 0 | 0 |

All 34,688 padded positions / 28,976 actual positions retain span presence and present lane codes exactly, with zero reconstruction error. Across 5,051,904 head-specific score comparisons (2,525,952 source/query pairs per head), largest observed score error is 9.5367431640625e-7. Per-head declared bounds range from 1.4077207427606457e-6 to 3.6583907695847236e-6. Both heads have zero source-majority changes and zero top-ranked occurrence/NoRead changes across all rows. The argmax observer is diagnostic; the actual reader remains softmax/value mixing, with no new hard selector. Ranking comparison preserves causal admission and NoRead; near-tie differences were diagnostic, not automatic mechanism rejection.

With Q24 rounding, 872,855 of 1,387,520 full-prefix F32 logit values differ bitwise; maximum absolute logit drift is 6.318092346191406e-6. This is not bit-exact model replay. All 512 answer decisions retain their baseline outcomes, including seed1's prior errors; the compiler does not repair answer realization. The preregistered component decision for both models is `BOUNDED_SCORE_AND_FIXED_DECISION_RETENTION`.

The single replay took 198.611832 seconds externally measured, maximum RSS157,728,768 bytes, peak memory footprint212,828,784 bytes. This is evaluation cost, not optimized serving throughput or energy evidence. The root is sealed and verified with all completed chunks, codes, per-query rankings, observations and source artifacts; raw score matrices are not retained in full. The executed binary is preserved with a byte-identical gzip roundtrip. No optimizer update, fresh draw or retry occurred.

Eleven focused checks passed: integer arithmetic/admission4, compiler/binding/bounds4, actual-stack integration1, retained causal/interface1 and evaluator1. Library checks bind implementation `fb10313ab0df20d26c302dc93d9b16ee18a98090`; evaluator/build bind `c69e4965`, whose only additional change is the evaluator's probability guard and assertions. Formatting and diff checks passed. Independent nonauthor review approved source and replay readiness.

Release AArch64 emitted `NativePotentialTables::score` has182 instructions with integer loads, shifts, additions, bit operations, comparisons and branches; no multiply/divide/floating-point arithmetic or allocation call appears. Relative/compose is inlined. Defensive panic callees were not disassembled, so closed-callgraph and whole-model instruction qualification are not claimed.

## Donor recompilation and next causal work

Offline donor weights and donor behavior remain valuable sources for learning these native operators. Preserve interfaces for token-to-geometric actions, contextual transitions/read/write/selection, and output behavior. This continuation compiles our own saved learned compatibility weights, including cross/radial/presence information; it demonstrates neither exact arbitrary-model conversion nor broader donor reasoning transfer. Token IDs are symbols; embeddings contain learned representations, while contextual computation also matters. The [induction-head primary study](https://transformer-circuits.pub/2022/in-context-learning-and-induction-heads/index.html) gives a concrete multi-operation predecessor/matching/retrieval circuit, not evidence that reasoning lives entirely in static tokens or that our model inherits it.

Retain this scorer and the exact span register. Next replace the four-way event controller's float contextual dependency with a small learned native geometric context register and factorized finite relative potentials, using actual token identities/actions and state to choose HOLD/OPEN/APPEND/COMMIT. Keep the reader/value path fixed initially to distinguish event-learning failures from code changes. This is a proposed causal continuation, not an implemented or qualified controller. Then address native current-context/radius/presence production, typed role/scope/span ambiguity, bounded candidate admission and native values/output. Do not expand a lookup over all combined states, silently resume runtime q/k, tune away seed1's preserved errors or discard useful selection because answer realization is incomplete. Keep all prior artifacts and negative results. #1512 remains open for wider language and serving acceptance.

[Machine receipt](geometric-potential-native-2026-10-01.json) binds results, error bounds, source/model identities and retained root. Complete preparation/build/review/evaluation/delivery/cleanup wall accounting is charged once from the preceding cutoff at closeout; overlapping specialist wall and the replay are not separately doublecharged.
