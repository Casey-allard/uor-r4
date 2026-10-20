# Matched causal-credit addressed-attention pilot — #973

**FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT.** One matched 64-update fit completed, but generation and retention acceptance failed. Retain `15baec48`; no candidate promotion. [Machine evidence](evidence/native_geometric_addressed_attention_causal_pilot_973.json) binds source, complete sealed attempts, logs, projection and independent reviews.

The [preceding causal comparison](native_geometric_addressed_attention_causal_credit_973.md) selected suffix cost-to-go credit after a 47.4% reduction in full-gradient gate covariance. This experiment tests whether that selection improves actual learning. It does not: lower variance at frozen endpoints did not produce useful generation at the unchanged pilot dose. This does not prove that geometry cannot learn language, that the expected gradient is wrong, or that a larger dose could never work.

## Matched recipe and implemented change

The new Rust causal pilot loads the original saved initial parameters, digest `d29bb3f055f015161c782015ae5113872d47effa93744d9363f4d3f5fe72891b`. It preserves 16 training/8 development records and order, initialization seed 7341, event seed 973, four particles, SGD rate 0.05, 64 updates and the 64-position window. The selected causal gradient replaces full-trajectory score credit; the direct emission derivative, full-document loss, policy, SGD and serving runtime remain unchanged. Matching seeds does not imply identical paths after parameter updates diverge.

A separate strict `UORACL01` checkpoint codec binds the actual causal implementation, data, configuration, event seed and update count. Old loaders and artifacts remain intact. The new final parameter digest is `3891c93acd9a3391907a2afb10571ce887e46c2c45c38b7e3480e6aa02455918`; causal checkpoint implementation identity is `d3a672294f3902443ea70aa77dd9d42345e67ce055951f2a4548ba8d05c52626`. Parameters remain finite, L2 change is 0.607128 and 517 compiled bytes change. This is a training checkpoint and compiled primitive witness; a normal trained Model artifact is NOT_EXPORTED.

Original acceptance is unchanged. An additional prospective requirement, frozen before fitting, preserves every correct Full training/development symbol and exact response from both saved predecessor endpoints. It strengthens retention and is reported separately. The candidate fails the original criteria independently. Old fits and evaluations were not replayed.

## Actual generated behavior

| Full panel | Original initial | Old full-trajectory final | Causal final |
| --- | ---: | ---: | ---: |
| Training response CE | 5.563666930 | 5.556881072 | 5.550466976 |
| Training correct symbols | 1/306 | 1/306 | 0/306 |
| Training exact answer + EOS | 0/16 | 0/16 | 0/16 |
| Development response CE | 5.553025500 | 5.551611374 | 5.571116484 |
| Development correct symbols | 1/154 | 1/154 | 1/154 |
| Development exact answer + EOS | 0/8 | 0/8 | 0/8 |

Training CE improves while development CE worsens. Each predecessor/split loses its single formerly correct symbol: initial training memory-a/1 position 7; old-final training sub-6-4/0 position 17; initial development operator/0 position 5; old-final development sub-8-3/0 position 7 (zero-based). The new development match occurs elsewhere and cannot offset retention failure. Every aligned symbol and actual generated output is retained in evaluation-1/retention.json and the Full row files.

ReadDisabled development CE is 5.574535559, ExactPayloadMasked 5.565610266 and StateTransportDisabled 5.547562343. Every control has zero exact answers. No Full-exact answer exists to lose; the weaker-control criterion fails. The latter two controls have lower CE than Full, which is a scoped negative intervention result for this witness, not a general conclusion about exact memory or geometric transport.

All four generated program candidates fall outside the frozen pure-program acceptance grammar and are NOT_EXECUTED. No generated coding semantic success is claimed. General conversation, reasoning, coding and energy savings remain unqualified; this is authored development, not final held-out evaluation.

## Validation, resources and preservation

50 focused release tests pass, zero fail, seven report drivers are intentionally ignored in that ordinary run. Fit and evaluation report drivers each separately pass execution. Checkpoint/export reload parity and complete interpreted/compiled deterministic trace parity pass on all 24 records. Saved initialized evaluation matches all nonfloating fields; historical JSON response CE uses the declared absolute tolerance 2e-14. These implementation checks do not turn the behavior failure into PASS.

The independent executed review verifies 69 gate-bound items, all 64 chained updates and all four retention losses. The final delivery check separately verifies documentation links, expanded evidence bindings and preservation. Both new attempts contain 27 sealed files; 26 predecessor roots/846 files remain preserved. Original dirty checkouts and retained model SHA256 `1454aa69e2e295d0a9970e90326f4664d3c614f666b1a708a16c16b5350a1705` remain unchanged.

Build/tests charge 117229ms, fit 18013ms and evaluation 2300ms: **137542/240000ms**. Internal fit loop is 17.516372s; evaluation 2.279339s. Peak sampled build RSS is 2656436224 bytes, fit 124469248 bytes and evaluation 28983296 bytes. Shared cumulative ledger is 123375513/132950000ms; parent 3624940/3730000ms. Necessary parent extensions of 180000ms and 64MiB storage were recorded before use. Step growth before documentation was 29642752 bytes within 128MiB; the final local resource receipt owns delivery growth, engineering and wall accounting. Two build threads, one model process, 4GiB RAM and 128MiB storage stop margin are retained. No external spending, cleanup or V3–V7 replay occurred.

Evidence remains in the established project-local handoff: `shared-core-first-step/addressed-attention-causal-pilot-1/`, especially fit-1, evaluation-1, gate-result.json, pilot-design.md, pilot-review.md, executed-review.md, delivery-checks.json, delivery-receipt.json and final-resource-receipt.json. Original paths and negative candidates are preserved.

## Next action

Define a deterministic-forward training-credit contract and execute one finite whole-runtime causal test before another fit. State the hard-path full-document CE objective explicitly, separate from the previous expectation over stochastic paths. Change one shared LUT row or legal geometric choice, replay every affected subsequent selection, state update, acknowledgment and publication through the actual runtime, and compare executed discrete loss contrasts against a finite reference. Include repeated shared use, changed-source selection and a late offered-symbol effect; preserve target exclusion before context selection. A discrete loss contrast is not the ordinary logit derivative. Any EFD/straight-through rule must declare its surrogate/bias and recurrent addressed-memory credit; DWN local Boolean differentiation is not a drop-in backward pass. Reuse the scoped UOR/Prism/NEMESIS/Spiralcore and DWN source research. No dose increase, variance panel, isolated emission-head fit or automatic language fit. Proposed complete local ceiling: 180000ms build/test/correction, two build threads, one process, 4GiB RAM, 96MiB new storage and 128MiB stop margin. Refresh cumulative balances and record any necessary preauthorized extension before use.

The unresolved stochastic/deterministic context mismatch was measured in the [frozen diagnostic](native_geometric_addressed_attention_stability_973.md); this run does not establish successful stochastic learning destroyed by export. The [existing specification](native_geometric_addressed_attention_spec_973.md) distinguishes these objectives and explicitly defers recurrent EFD credit. [DWN](https://proceedings.mlr.press/v235/bacellar24a.html) supplies research on approximate differentiation through discrete LUT networks; the [pinned source review](native_geometric_attention_reassessment_973.md) does not establish a ready-made recurrent language learner. The next finite test is a causal check of the proposed contract, not a claim of useful language learning.
