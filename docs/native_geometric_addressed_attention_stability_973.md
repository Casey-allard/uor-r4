# Addressed attention: frozen gradient and context diagnostic — #973

**COMPLETE_FROZEN_GRADIENT_CONTEXT_DIAGNOSTIC.** September 13,2026. The [prior learning pilot](native_geometric_addressed_attention_pilot_973.md) remains **FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT**. Retain15baec48; no promotion. This is a completed diagnostic on two construction records, with zero optimizer updates or new fits, not a model acceptance pass.

[Source-bound evidence](evidence/native_geometric_addressed_attention_stability_973.json) binds all30 implementation/contract files, the executed release binary, frozen design, logs and complete40-file sealed attempt. Original reports and artifacts remain at their existing paths under the established handoff. The UOR/Prism/NEMESIS/Spiralcore [research reassessment](native_geometric_attention_reassessment_973.md) and [attention specification](native_geometric_addressed_attention_spec_973.md) remain the design context.

## What was measured

The Rust diagnostic transparently wraps the existing sampled policy and serial leave-one-out estimator. It separates mean direct emission credit from score-function credit and computes each component's repeatability over eight independent four-particle batches. It reports sample covariance trace, standard error of the mean vector, signed bias-corrected squared mean norm and mean pairwise cosine over nonzero vectors. Full/direct/score covariance is paired; independence is not assumed. Six parameter families cover all665,216 entries.

Two preselected construction records, train/memory-a/0 and train/sub-9-2/0, are measured at the saved initial and64-update endpoints: **32 batches /128 trajectories**. The event seed1973 and document/particle identities are paired between endpoints; divergent paths may subsequently use different event keys. Every prompt, response and EOS position uses the original full-document objective. Prompt and response losses are additionally reported with their own denominators. The parameters remain bitwise unchanged. Four deterministic teacher-forced traces use the independently loaded compiled exports; this is not a new autoregressive generation evaluation.

## Actual result

Sampled execution visits about31 distinct emission contexts among32 trajectories at an aligned position. Its matches to deterministic execution are rare:

| Record / endpoint | EMIT context matches | Response matches | Gate pairwise cosine | Gate mean L2 / estimated mean-error L2 |
| --- | ---: | ---: | ---: | ---: |
| Memory / initial | 3/1440 | 1/256 | -0.005155 | 0.6670 / 0.6823 |
| Memory / final64 | 4/1440 | 0/256 | -0.003034 | 0.7282 / 0.7307 |
| Coding / initial | 6/1792 | 3/960 | 0.001629 | 0.5530 / 0.5487 |
| Coding / final64 | 7/1792 | 4/960 | -0.003547 | 0.7255 / 0.7468 |

Gate-gradient directions have little agreement at this sample size. Three of four corrected squared mean-norm estimates are negative; those values are retained, not clamped into a positive signal. The direct emission component has weak positive pairwise agreement (0.0153–0.0317); its score-credit component is near zero (−0.000435–0.000486). Zero direct gate credit is a property of this estimator's decomposition, not evidence that gates cannot learn.

Final-minus-initial sampled response CE is **+0.0009335 (SEM0.0085297)** for memory and **−0.0030312 (SEM0.0031723)** for coding. These eight-replicate differences do not establish consistent improvement. Deterministic response CE improves by0.0402121 on memory and worsens by0.0447785 on coding. All101 deterministic emission contexts change between endpoints. The earlier full generated-answer gate remains failed.

This supports investigating weak credit repeatability and diffuse training paths. It does **not** prove a zero population gradient, an invalid score estimator, insufficient geometric capacity, or successful stochastic learning lost during export. Low context overlap near initialization is descriptive, not a unique causal diagnosis. Raw local occurrence extents/root tuples and categorical offered-symbol Hamming are measured; equal result ordinals across divergent sessions do not prove equal payloads or derivations. No hash/prime identity is treated as semantic distance.

## Validation, resources and preservation

**43 focused optimized tests pass**, with four report drivers intentionally ignored in the ordinary run; the diagnostic report driver separately executes and passes. Tests check exact gradient/CE/event parity to the old batch, component/loss reconstruction, first-target causal invariance and known covariance/cosine cases. One initial compile error in a JSON macro was corrected before execution; its failed command and source freeze are preserved and charged. Existing learner, policy, artifact and serving implementations are unchanged; module registration adds diagnostic modules.

The diagnostic takes **8.178663s internally** (8,440ms charged), peak sampled RSS92,602,368 bytes. Complete build/test/diagnostic cost is **121,732/180,000ms**, including the2,135ms failed compile and111,157ms corrected build/test. Peak sampled build RSS2,549,481,472 bytes. Shared cumulative ledger123,116,211/132,950,000ms; parent3,365,638/3,430,000ms after the pre-recorded70,000ms local extension. Step storage19,001,344 bytes before documentation/delivery within96MiB; final local receipt updates storage/engineering/wall use. The128MiB stop margin remains reserved. No external spending or cleanup.

Both original checkout heads/dirty status, retained model and24 prior sealed roots/766 files verify unchanged. Initial parameter digest d29bb3f055f015161c782015ae5113872d47effa93744d9363f4d3f5fe72891b and final witness96a0faff5a6d439857e800007fa0d5a43d20c00a8c744f3649c503c605abf515 are unchanged before/after. The trained witness is not a promoted normal model artifact. No V3–V7 replay, new fit, fresh holdout, language/coding qualification or energy measurement occurred.

## Next action

Implement exact causal cost-to-go credit in the Rust learner, removing losses that precede each stochastic event while preserving the full-document objective, direct emission derivative and four-particle independent leave-one-out baseline matched to the same downstream cost boundary in the other particles. Events before CE_t receive suffix t..T; offered-symbol and observation events after CE_t receive suffix t+1..T. Count every replay if an implementation uses a second pass. Check analytic expectation and offered-symbol timing on finite causal fixtures, then make one paired frozen-checkpoint comparison against the existing estimator on the same two construction records/endpoints and eight four-particle replicates (32 shared batches). Freeze variance, mean consistency, trajectory equality and cost criteria before execution. No temperature change, corpus expansion, parameter update or new fit in that comparison. This addresses avoidable credit variance; it does not by itself solve sampled/deterministic context mismatch. Proposed complete ceiling 180,000ms (140,000 build/checks,20,000 comparison,20,000 correction), two build threads/one model process,4GiB RAM,96MiB new storage and128MiB margin. Refresh the64,362ms parent balance and record the necessary preauthorized local extension before use; no paid compute.

This uses the downstream-cost option already present in the specification and the stochastic-computation-graph estimator of [Schulman et al.](https://arxiv.org/abs/1506.05254). The paper supplies a credit-accounting framework, not evidence that this UOR implementation learns language or that variance must fall on every finite sample. Preserve the present estimator and checkpoints for an exact paired comparison. #973 remains actively assigned/open; #964/#820 remain open.
