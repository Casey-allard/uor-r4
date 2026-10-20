# KVAR matched relative-energy pilot — result, 2026-09-24

The [pre-registered residual comparison](kvar-relative-energy-plan-2026-09-24.md) is complete. **Decision: reject promotion of the Q8 query/value read-energy residual.** This is a negative on the synthetic KVAR panel, not an adjudication of end-to-end geometric state models, geometric attention, language quality or energy. The original full `(f)` GRU/delta-fastweight and `(h)` end-to-end geometric arms remain `NOT_RUN`.

## What ran

Both arms used the **same frozen quantized `(c)` model per seed**, the same exact addressed overwrite store, the same training episodes and five coordinate sweeps. The ordinary arm used a cyclic C8 relative table; the geometric arm used the exact directed Q8 signed-vector-action table. Each added 66 three-bit action labels, eight signed four-bit kernel weights, a fixed 64-entry relation table and four final-query reads. The learned integer residual altered only the last hard read gate. Transport-off forced Q8 frames to identity; kernel-off replayed the source `(c)` served scores **exactly on every held episode**. The served residual used reads, shift, integer add and compare, with no float or multiply. The base model still inspects **8,771 parameter slots per token**; this pilot does not satisfy terminal D5 sparsity.

The final panel was untouched seed group 4: six `(K, lag)` cells, 34 episodes each, 204 per model seed. Group 1 provided 100 training episodes per cell and group 2 was used only for development. Validity controls on the final panel: order-2 count **2/204** (near 1/64 chance) and hand-coded overwrite **204/204**. Bits/query are measured at the fixed temperature 32; lower is better. Intervals are paired episode bootstraps.

| Frozen base seed | Ordinary C8 bits / accuracy | Q8 bits / accuracy | Q8 minus ordinary bits, 95% interval | Q8 transport-off bits | Kernel-off `(c)` bits / accuracy |
|---|---:|---:|---:|---:|---:|
| 1 | 2.1518 / 0.8039 | 2.0181 / 0.8235 | −0.1337 [−0.3708, +0.0980] | 2.5974 | 1.6240 / 0.8824 |
| 2 | 2.0938 / 0.7647 | 2.0938 / 0.7647 | 0.0000 [0, 0] | 2.0938 | 2.0938 / 0.7647 |

The pre-declared gate required a **≥0.5-bit Q8 advantage with an interval excluding zero and a transport-off degradation on both seeds**. It fails. The seed-1 Q8 read changes 14/204 predictions from kernel-off, while ordinary changes 16/204; both worsen the unmodified source model in bits. Seed 2 learns a zero kernel, so every arm is identical. The development panel also failed the margin (Q8 minus ordinary +0.1475 and 0 bits). Its result was not used to alter the final design.

This panel pairs values randomly with keys and uses an exact key-addressed store. At the read location, the queried key and stored key coincide, so their relative action is always identity. The query/value relation tested here can only learn a residual pattern over arbitrary token labels. The negative therefore supports **retiring this residual for KVAR**; it does not reject Q8/H4 transport in a task with observable frame structure. The positive transport-off difference on seed 1 alone is insufficient, especially because kernel-off is better.

## Retained evidence and provenance

The unique sealed root is `/Users/casey.allard/uor-r4-investigations/kvar-relative-20260924/final1`, with receipt, fitted parameters, per-prediction served scores, the generated panel and seal. Development roots `quick1` and `development1` are also sealed and retained. Source `(c)` parameters came from sealed `final-complete-sg3-v2/parameters.json`, SHA-256 `7aec17b87fa8a005ca87855f8a68a156c094d82fc08b1011b3f3401e4b251401`. The final receipt SHA-256 is `32f038495892a2cc9a314363497fe20c6d7d889486e2bac6bb1c236a2b8faf1b`; fitted parameters SHA-256 `9c00067095a80f72ee013988da97ea9f18dc07fc59ea44cae1b86d8afa801466`. Executed release binary SHA-256 `dbdef492fb00d7eef9517c5a4583e7983f1333e9708173975e8a96cdb37097b0`; the two changed Rust source files had SHA-256 `82560a5625f528839c1e491a8024a8c7b790d4f884596f2528dbeef39661ef93` (`kvar-recall.rs`) and `c6f77656c835fdfe953fb54f01d483c5975ca4d8b2f943dd9188679d134c21f4` (`kvar_relative_energy.rs`) at execution. The source and executable hashes bind this run; they are not a cross-backend training reproducibility claim.

Focused binary tests (`12/12`), a reused-cache release build, the quick and development panels, the fresh final panel and exact kernel-off replay executed locally. The original checkout, frozen `(c)` artifact and all negative roots remain untouched.

## Next dependency

The validated lever remains **addressed overwrite memory**, not this relative-energy residual. The next capability-relevant experiment should integrate that memory into the native language path and require actual loaded-generation changed-source/read-disabled controls with retained lexical and prose checks. Before attributing any gain to Q8 or Hamiltonian transport, use a task with a nontrivial observable frame relation and an information-, parameter-access- and training-matched ordinary arm. The full original `(f)/(h)` comparison remains open until such an end-to-end implementation is specified and run; this pilot must not be silently substituted for it.
