# Saved-model hard capture/hold bridge

This study continues the [variable-gap learned latch](geometric-attention-latch-2026-10-01.md) on #1512. It replaces the learned soft gate only, using the four saved capture-credit models. No fitting, new data draw, changed source identities, candidate mask, q/k map, radius normalization or metric is introduced.

## Mechanism and experiment

At each token, the learned raw gate logit determines CAPTURE if `z >= 0` (including zero), otherwise HOLD. The Held recurrence copies the complete current gained RMS representation on CAPTURE and retains the complete prior state on HOLD. The Local control copies on CAPTURE and clears on NO_CAPTURE. Both expose the previous identity to q/k before updating it. The previous-input register advances on every token. Values, current-role q/k contributions, age, NoRead and the full causal support remain unchanged.

The public APIs are per-call diagnostic overrides. Ordinary model forward and saved model format remain soft. Hard diagnostic logits, binding masses and actions propagate through the same hard read policy, including earlier read layers. Nonfinite gate logits are refused. This is a floating-point reference to a discrete mechanism; it is not a D11 served artifact or a straight-through training bridge.

The evaluator loads all four saved models from the sealed `run-3-capture` parent. Each receives exactly the original `evaluation.json` and `stress.json` bytes. It records full answer-position logits, predicted answers, exact correct-occurrence masses on both heads, gate logits and hard classifications. It compares ordinary soft outputs before and after each hard intervention, preserving the saved parents. Source labels observe read probabilities; event labels observe action correctness; neither enters model execution.

Each panel contains 128 rows in 32 correlated intervention groups. Existing longer gaps, changed queries, swapped values, reversed ordering and duplicate-value distinctions remain. Both panels are open development evidence. Context cap128, width32, observed original/stress maximum lengths50/76 are distinct quantities. Configuration `rra` puts the latch in the final read, so its pre-read trunk/logits are unchanged by this intervention; a future multi-read stack need not retain that equality.

## Results

| Model | Original soft / hard answers | Stress soft / hard answers | Hard head-zero source majorities, original / stress |
|---|---:|---:|---:|
| Held-s1 | 128 / 128 | 128 / 128 | 128 / 128 |
| Held-s2 | 128 / 128 | 128 / 128 | 128 / 128 |
| Local-s1 | 47 / 47 | 59 / 59 | 24 / 37 |
| Local-s2 | 54 / 54 | 49 / 49 | 29 / 36 |

Every cell is out of128. Hardening changes zero answer predictions in every model/panel and loses zero previously correct answers. Both Held models preserve every head-zero correct-source majority, all29 original and27 stress changing-answer query pairs, and the3/5 equal-answer query pairs with distinct source occurrences. Both are perfect in all disjoint write/query-gap stress cohorts; their group memberships overlap and must not be summed.

The hard Held source mass minima are0.990442/0.981927 for seed1 original/stress and0.997173/0.986671 for seed2. Maximum absolute head-zero mass changes are6.50e-6/1.12e-5 and3.87e-6/1.02e-5. The largest absolute change over all answer-position vocabulary logits in either Held model/panel is9.60231e-5. This is measured floating-point drift, not exact equality or a general tolerance guarantee. Head1 retains zero correct-source majorities in Held; no all-head routing claim follows.

The minimum absolute Held gate logit is5.649386/5.591120 for seed1 original/stress and6.051017/6.340918 for seed2. An integer gate error smaller than this margin preserves the action on these fixed inputs; this does not bound errors from a quantized trunk.

All four models classify every unpadded capture event correctly:4,168/4,168 original positions and6,224/6,224 stress positions per model. Correct event classification alone is insufficient: hard Local controls remain at47/54 original and59/49 stress answers. Local-s1 gains one head-zero original source majority (23→24); its answers remain unchanged. Other source-majority totals remain unchanged.

All ordinary soft predictions and source masses reproduce the saved parent row by row (comparison tolerance1e-6 for source mass; independent review found serialized source masses exactly identical). Full soft answer-position logits and recorded diagnostics are unchanged before/after the hard call. The fixed rra actual hard-action getter equals raw-logit thresholding throughout. The evaluator runs125.779s internally (127.89s process wall), zero training updates,2Rayonthreads, maximum resident set232,390,656bytes. Debug/unoptimized execution is not optimized serving performance. The run is claimed, sealed and verified; all source/data/model/executable identities and row outputs are retained.


## Decision and limitations

**Decision: retain the hard diagnostic bridge and advance integer latch lowering.** No new fit is justified by these panels. The original work card defines the decision: preserve successful soft models regardless of the hard result. If hard actions preserve answer and occurrence decisions, retain the discrete capture/hold mechanism and advance faithful integer latch lowering. If they lose binding, inspect row-level action/state/score differences before a training change. Neither outcome retires geometric attention or qualifies language.

Integer continuation must align current and previous input exponents before the gate accumulator, keep the held mantissa with its capture-time exponent, and align current and retained q/k contributions before rounding. HOLD copies neither the current exponent nor a unit-normalized replacement into retained state. Hard action success does not establish low-bit gate/map fidelity, bounded candidate access, complete integer inference or energy savings.

## Executed verification and artifact binding

Nine focused latch tests passed (`cargo test -p uor-r4-training --lib read_identity_latch_ --locked --offline -- --test-threads=2`), with0failures and458other tests filtered out. The3new tests cover raw-logit threshold/signed-zero/nonfinite semantics and bitwise Held/Local state copying; multi-read hard propagation, source-label isolation, future causality and parameter immutability; unchanged save bytes and soft/hard reload. The6retained tests cover soft recurrence/gradient and gate credit, mode/metadata, checkpoint and export refusal. The focused execution took0.54s after compilation. Touched Rust rustfmt, diff check and claim wording passed; compatibility CI labels are not test results.

Measured executable SHA256:`abc03eb99bd3ae41b97fa210d1b1446b77a73131893df9195406587453272890`. It does not embed a commit (`UNAVAILABLE`); retained source/executable hashes bind the run. Production source SHA256:`6aecb745de3194eed49badbc0fa58326594acba6b4b85d0cf60cc2c786183642`; measured evaluator SHA256:`75ca2f38ab72c85803a52e1f924e3329721cd04e059f72a4fab0818a9c632630`, both delivered in source commit77271969. A subsequent evaluator validation change requires the complete fixed geometric configuration (including vocabulary40 and no pointer/selector/store); it does not alter measured forward computation, inputs or results. The original executable/evaluator/diff remain preserved, and the final evaluator is compiled separately.

The initial research build took4m57s. The focused test profile rebuilt optimized dependencies; compilation cost is included in complete elapsed accounting, not model execution. Initial complete projection90min, evaluation900s,2modelthreads/2buildjobs,4GiB RAM,32MiB new retained and128MiB storage stop margin. Disposable cache allowance was prospectively extended6→7GiB under standing local authorization before final test linking, because the optimized test profile adds a second dependency build. All cache is removed after protected delivery; all unique models/evidence remain. No paid or SSD compute.
