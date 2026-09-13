# Addressed geometric attention: first learning pilot — #973

**FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT.** September 13, 2026. The one fixed run completed all **64 SGD updates**, but did not learn useful generated responses. Development response CE decreased slightly while symbol correctness stayed at **1/154**, with one formerly correct symbol lost. All **8 development and 16 construction continuations fail exact answer plus EOS**. Retain `15baec48`; no promotion or second fit.

This executes the [forward integration's recommended pilot](native_geometric_addressed_attention_forward_973.md). The [tracked evidence](evidence/native_geometric_addressed_attention_pilot_973.json) binds the exact source, compiled test executable, frozen data/acceptance, checkpoints, controls, sealed attempts and resources. The [reviewed attention specification](native_geometric_addressed_attention_spec_973.md) and prior UOR/Prism/NEMESIS/Spiralcore research remain the architecture context; this result does not demote that architecture or establish its semantic advantage.

## Implemented learning and preserved boundaries

The new Rust [learner](../crates/uor-r4-core/src/native_geometric/addressed_attention/learner.rs) computes four complete hard trajectories at each frozen parameter vector and applies one plain SGD update. Every gate/head invocation contributes its event score, including effects of emitted symbols on later acknowledgments. Direct conditional CE is added after the actual EMIT context is fixed. The full-document mean objective includes every prompt byte, continuation byte and terminal EOS. There is no pointer/role supervision, output override, optimizer clipping, momentum, auxiliary loss or parameter sweep.

A strict checkpoint binds the full 665,216-element f64 parameter vector bitwise, initialization/event seeds, completed update count, data/configuration digests and old/new implementation identities. Initial and final checkpoints are saved and reloaded. The final compiled primitive payload is independently reloaded and compared with the checkpoint's compilation. A complete learned-parameter interpreter/export trajectory comparison passes. Existing initialized-model loader and serving source remain unchanged except for registering new modules.

The final parameter witness is `96a0faff5a6d439857e800007fa0d5a43d20c00a8c744f3649c503c605abf515`. It is **a trained checkpoint and source-bound compiled witness, not a promoted normal model artifact**. The initialized artifact `ac5cf4ad…` remains the bound geometry/parent reference; its initialized-only envelope is not relabeled as trained. Serving still uses the same integer/table geometric and exact-object route, with no matrix products or transformer backbone. Offline learning uses floating point.

## Frozen experiment

The [Rust data preparation](../crates/uor-r4-core/src/native_geometric/addressed_attention/pilot_data.rs) provides 16 construction and eight development records. They contain short owner/location queries, duplicate payloads, current-value statements, ordered subtraction and changed-operator prompts requiring complete minimal Rust programs. Development uses different names/locations and operand combinations. These are authored family probes, not an independent natural-language corpus or final holdout. Typed intent authors answers once; evaluation does not invent alternatives from generated output.

Each complete document, including EOS, is at most 64 symbols. The single recipe is seed7341, fixed wiring0x973, event seed973, rate0.05, four particles,64 updates cycling construction records in fixed order. The RNG document identity is the unique update index, so revisiting a record does not replay an old random stream. Initialization and step64 are the only parameter checkpoints; no checkpoint-selection sweep ran.

Evaluation scores response bytes plus EOS and separately generates up to64 symbols from a fresh session. Both evaluation and generation prefill the prompt with full predict/discard-offer/observe operations, matching training. The two-phase `observe_input` shortcut is not used here. Records reset independently; this is not a learned multi-turn-session claim.

The pre-fit gate required lower development response CE, more correct symbols, zero initialized-correct symbols lost, at least one exact answer/EOS in each domain, one valid changed-answer pair with both responses exact and different, and all three controls worse in CE and losing a Full-exact answer. Actual generated Rust semantics also had to pass. Passing this narrow pilot would not authorize model promotion.

## Actual evaluation

| Artifact / control | Correct response symbols | Response CE, nats | Exact generated answers + EOS |
|---|---:|---:|---:|
| Initialized, construction | 1/306 | 5.5636669304 | 0/16 |
| Trained, construction | 1/306 | 5.5568810724 | 0/16 |
| Initialized, development | 1/154 | 5.5530254998 | 0/8 |
| Trained Full, development | 1/154 | 5.5516113742 | 0/8 |
| Trained ReadDisabled | 1/154 | 5.5755041012 | 0/8 |
| Trained ExactPayloadMasked | 0/154 | 5.5574766796 | 0/8 |
| Trained StateTransportDisabled | 1/154 | 5.5818403889 | 0/8 |

ReadDisabled returns each query as its Null key; maximal self-score and Null-first ties prevent exact reads while retaining scans. ExactPayloadMasked clears selected/active payload-byte bits776–799 and numeric flags804/805, retaining geometry, lengths, ages and legal-action effects. StateTransportDisabled evaluates then overrides root-delta heads4–7 with identity; zeta channels remain. These are accurately scoped interventions, not removal of all context or a candidate-key shuffle. Their differing actual circuit/scan costs are retained per row.

All control CEs are worse than trained Full, but no Full-exact response exists for any control to lose. Correctness and generation therefore fail the causal gate. Of154 development predictions,153 change between endpoints. The initialized correct `i` at position5 of `dev/operator/0` is lost; `(` at position7 of `dev/sub-8-3/0` is gained. The aggregate1/154 obscures that regression. Raw outputs remain incoherent byte sequences. Six development generations reach64 symbols; two stop with EOS at55/16 symbols, without an exact answer. None of the four generated coding files meets the frozen pure-program admission rule, so compilation/execution is **NOT_EXECUTED**, not a compiler failure or successful program.

## Saved-artifact diagnosis

A descriptive inspection of the saved artifacts performs no model calls or additional fitting. SGD changes the parameter vector by L2 distance0.8948008289 and changes876 compiled bytes. Fixed wiring is identical. Changed compiled decisions include78 of5520 gate truth bits across68 gates,76 of4096 root modes,11 of512 emission modes,253 extent preference rows,11 control preference rows and one Null root. These structural differences are not semantic distances.

The four construction passes have sampled training CE means5.56349,5.56201,5.56319,5.55980. Each uses new trajectories and changed parameters, so these pass means are not a matched objective comparison. Mean per-batch gate-gradient L2 ranges1.61–2.35, while the combined head-gradient norm ranges0.27–0.38. That difference motivates checking gradient repeatability; it does not itself measure noise or establish a defective estimator.

The code executed updates, changed hard decisions and preserved export parity. The remaining possibilities include insufficient dose, noisy credit, and a mismatch between sampled trajectories and deterministic deployment. **None is resolved by this pilot.** A slightly lower CE near the uniform257-symbol reference does not warrant another blind fit or a new mechanism.

## Validation, resources and preservation

One optimized build executes **37 focused tests successfully**; three bounded report drivers are intentionally ignored in ordinary test runs. Separate fit and evaluation drivers each pass their execution checks. Fit completion is distinct from the failed model gate. Tests cover finite SGD/update direction, bitwise checkpoint/rejection, corpus/oracle boundaries, controls, retained-symbol regression rejection and all prior addressed-attention primitive/forward checks. Two existing unused fixture warnings remain.

The fit takes17,126,809µs internally and charges17,740ms including reporting. Evaluation takes1,950,317µs and charges1,962ms. Build/tests charge113,104ms. Total **132,806/240,000ms**. Sampled peak RSS: build/tests2,611,314,688 bytes; fit55,148,544 bytes; evaluation28,655,616 bytes. These are sampled process-tree values, not allocation or energy measurements. Measured step storage before documentation/delivery is29,974,528 bytes within128MiB; final accounting appends locally.

The necessary parent allowance increase of160,000ms was recorded before use:3,200,000→3,360,000ms. Parent consumption is now3,243,906ms; shared ledger122,994,479/132,950,000ms. No shared-ceiling increase, paid compute or cleanup. The128MiB storage margin remains reserved.

Both original checkout heads/dirty status, retained artifact and all **22 prior sealed roots/740 files** verify unchanged. The previous initialized artifact implementation is byte-identical except for module registration; its binary remains preserved. Fit and evaluation claim separate new roots, save the complete outputs and seal/verify their file sets. Notes/navigation remain under established handoff `shared-core-first-step/addressed-attention-pilot-1/`.

## Next action

Run **one zero-update gradient and context-stability diagnostic** on the saved initial/final checkpoints before choosing more dose or a mechanism change. Freeze two existing construction records (`train/memory-a/0` and `train/sub-9-2/0`), eight independent four-particle replicates per endpoint/record, with paired common RNG identities between endpoints:32 batches total. Measure prompt/response objective separately, per-family direct and score-credit means/dispersion, and actual sampled EMIT-context support against the deterministic contexts. Use raw geometric roots/references/contexts for state comparisons; a snapshot hash also binds parameter identity and cannot alone establish state drift. Keep all parameters frozen, reuse the existing data, and introduce no new evaluation-based selection or fit.

This is a targeted diagnostic on two fixtures, not a corpus or generalization result. Proposed complete ceiling180,000ms:140,000 build/checks,20,000 diagnosis,20,000 correction; two build threads/one model process,4GiB RAM,96MiB new storage and128MiB stop margin. Refresh the116,094ms parent balance and record any necessary preauthorized extension before execution. Preserve this negative candidate and all report roots. #973/#964/#820 remain open; broader language and retained-model qualification remain outstanding.
