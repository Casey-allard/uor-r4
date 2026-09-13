# Conditional geometric decoder validation — #973

**Actual result: FAIL_DECODER_VALIDATION_DEVELOPMENT_GATE. Retain 15baec48; no promotion.** This executes the next action from the angular-tree checkpoint once. Regularization improves opened likelihood, but preservation and the complete opened gate fail; actual generation remains incoherent.

Four fixed depth/minimum-leaf families were evaluated with three deterministic document folds (document index modulo three), eight fitting and four validation documents per fold. Each fold excludes its validation documents from incremental tree topology, leaf fitting and cap/tree selection. The recurrent and cap parent already saw all construction documents. This is conditional incremental decoder selection, not independent end-to-end cross-validation or fresh qualification. Opened and control labels were excluded from selection.

| Maximum depth | Minimum leaf examples | Pooled validation NLL | Correct / 672 | Sum of fitted fold nodes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 4 | 3.635385 | 67 | 380 |
| 2 | 4 | 3.873000 | 61 | 649 |
| **3** | **16** | **3.616284** | **62** | **379** |
| 3 | 4 | 4.079226 | 57 | 871 |

Selection uses position-weighted pooled validation NLL, with ties within 1e-10 of the global minimum resolved by fewer summed nodes and then family order. All four validation losses exceed the fixed cap comparator's 3.354320; that parent has pretraining exposure to these documents and is an informational comparator, not a fifth selectable configuration. Twelve fold fits and one final refit ran; no additional configuration or retry ran.

The selected depth-three/minimum-16 configuration produces candidate **blake3:8f8e5c342c83b062274744033a96668e9c079dc47a9c4d1b138baa7ea4889b6a**, with 36 selected trees / 184 nodes across 94 occupied branches. Final fitting examines 443,520 split candidates in 40 ms internally; the complete experiment takes 894 ms internally. The serialized candidate is 179,006 bytes, 11,103 above the cap parent and 8,447 below the previous tree. Production training, runtime and artifact source are unchanged; this step adds a bounded Rust experiment driver and focused selection test. Signed H4 geometry, recurrence, context routing and inherited cap parameters stay fixed.

| Panel | Candidate correct | Candidate NLL | Lost correct vs 0f82 / ade9 / 1198 | Prediction Hamming vs 0f82 / ade9 / 1198 |
| --- | ---: | ---: | --- | --- |
| Construction Full, 672 positions | 166 | 2.839534 | 47 / 43 / 74 | 561 / 575 / 330 |
| Construction ContextDisabled | 126 | 3.356711 | 34 / 30 / 38 | 578 / 592 / 354 |
| Construction StateDisabled | 69 | 3.943612 | 48 / 51 / 33 | 611 / 610 / 499 |
| Opened Full, 381 positions | 52 | 3.653604 | 20 / 13 / 20 | 317 / 320 / 213 |
| Opened ContextDisabled | 49 | 3.737585 | 15 / 14 / 21 | 328 / 330 / 215 |
| Opened StateDisabled | 49 | 3.790113 | 31 / 29 / 13 | 355 / 356 / 278 |

Construction optimization against the immediate cap parent passes: 81 → 166 correct and NLL 3.354320 → 2.839534. The opened Full candidate improves both cap parents' NLL and accuracy, and improves previous-tree NLL 3.908623 → 3.653604. However, opened accuracy falls from the previous tree's 57 to 52/381, so the all-prior opened gate fails. Preservation also fails, with **195 / 180 / 199** correct-to-wrong rows against 0f82b728 / ade9a1cf / 1198ef47, respectively. These are separate comparisons, not additive unique errors.

Every teacher-forced root/source trace matches. Prediction Hamming counts unequal categorical predictions at aligned positions; it is not semantic distance between hash or token IDs. All 9,477 earlier saved rows and all 36 prior generated outputs replay exactly. Immediate post-reload Full generation precedes broader comparisons and reproduces later. All four new Full continuations remain incoherent, reach the 96-byte cap and emit no EOS. The Rust continuation is malformed text; no useful prose, factual answer or Rust generation is established. The sealed report retains all 48 outputs and all six panels for every model.

Regularization reduced tree size and improved opened likelihood in this authored development population. It did not establish reliable generation, preservation, generalization or a global representation limitation. The result supports moving the next bounded causal intervention upstream rather than another decoder-only sweep.

**Next action:** One bounded complete-construction recurrent/read-pair intervention with selected decoder `8f8e5c34` fixed. Project a single 120 × 120 enumeration of parameter indices 1176 and 1930, scoring complete causal sequences over all 672 construction positions; freeze all other parameters and exclude opened/control labels from selection. Prior exhaustion under the cap emitter does not establish exhaustion under this tree emitter. Bind a changed artifact with valid tree-aware provenance or use an explicitly source-bound test-only parameter witness. Compare against 8f8e5c34, 1198ef47, 0f82b728 and ade9a1cf separately, measure state/source drift, retain all preservation and generation gates, and record a negative result if the block has no improving setting. No decoder refit, larger corpus, automatic joint campaign or promotion follows from this recommendation.

Research reused the [SpiralCore/FBS review](../research/spiralcore-v68/README.md), prior source audit of geometric partitions and the local knowledge map. [Cawley and Talbot (2010)](https://www.jmlr.org/papers/v11/cawley10a.html) informs separation of model selection from performance evaluation and the finite-selection limitation; it does not establish a UOR-R4 result. No external mechanism was imported.

Executed validation: offline release compilation, the focused document-fold/weighted-selection/tie test, the single actual sealed experiment and an actual-candidate 512-step allocation census pass their execution checks. The source-identical allocation binary was reused with a pinned hash; it reports zero allocations and at most 27 emission angular comparisons per prediction. This is bounded integer/table serving, with no new transformer or runtime matrix products. No blanket suite, fresh qualification, recurrent fit or complete-path energy claim was run. The model gate remains failed.

[Tracked evidence](evidence/native_geometric_decoder_validation_973.json) binds source, compiler, binaries, design, all folds, candidate, rows/outputs, independent reviews, research and resources. Local originals remain in shared-core-first-step/decoder-validation-1 under the established project-local handoff; attempt-1 is sealed. Thirteen prior sealed roots / 111 files, 22 parked dirty paths and retained model verify unchanged.

Execution-close resources: **97,800 / 300,000 ms** model/build/test; parent cycle **2,156,722 / 2,400,000 ms**; cumulative ledger **121,907,295 / 132,950,000 ms**. Added storage 26,787,840 bytes within 128 MiB; parent growth 1,491,619,840 bytes within 2 GiB with the 128 MiB stop margin. Peak sampled RSS 2,357,968,896 bytes. No allowance extension, paid compute or destructive cleanup. Final engineering and delivery receipts append locally.
