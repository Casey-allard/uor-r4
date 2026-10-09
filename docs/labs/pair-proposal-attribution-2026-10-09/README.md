# Pair proposal attribution at the earliest remaining error — 9 October 2026

**No position-4 correction was offered to the gates in the completed bounded pair run.** Every one of its 13,440 recorded alternatives still chose token 307 instead of target 267. The selected 960 coordinates nevertheless captured **92.7977% of this frame's available initial first-order legal-descent estimate**. This weakens a simple ranking-omission explanation and does not support an automatic larger pair-only sweep.

This is saved evidence attribution following merged #2055, under the [M2 claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6085293771). No model, gradient, proposal, native scoring or new GPU job ran. Accepted whole-answer performance remains **8/512**; the separate pair conditional 6/15 and coupled conditional 9/15 remain unselected results.

## Exact finite observation

The focus is input 245, position 4, target 267, union row 380 / objective slot 4. The original/final conditional prefix is `[617, 2097, 315, 1057]`. The reader joins the saved population map and roles to every recorded alternative at its actual incumbent epoch; it does not compare alternatives from different epochs as if they shared one state.

| Recorded classification | Alternatives | Target-267 winners |
| --- | ---: | ---: |
| Accepted coordinate changes | 931 | 0 |
| Feasible but another code was selected | 3,215 | 0 |
| First protected-guard veto | 210 | 0 |
| Strict combined-CE rejection | 9,084 | 0 |
| Total | 13,440 | 0 |

All 17 reference winners survive every recorded alternative, so this run's objective-ineligible rows are CE-only rejections. The reader derives CE descent and reference retention separately from the saved incumbent CE and objective masses, then checks their conjunction against `objective_gate`. The 210 guard vetoes concern proposals that still predict the wrong focus token; they are not rejected corrections of this error. There is no accepted target-winning change and therefore no accepted-then-lost correction.

Target pooled probability improves from **0.018637775155279073** to **0.020563290376237518**, while the winner stays 307. Relative to their own incumbents, 1,107 alternatives increase this probability, 3,219 decrease it and 9,114 leave it equal. The highest observed probability occurs at order 956, coordinate 16665, incumbent epoch 927, code -7; this change is accepted. Probability improvement and winner correction are separate findings.

## Initial gradient coverage and its limits

Position 4 is physical gradient term 4, already weighted by 1/15. It is present and nonzero; no weight is applied again. The original fractional masters are directly authenticated by `generate-original-master-binding.json`, with all 57,600 values retained. The reader reconstructs the producer's complete frozen ranking, including actual legal displacement, signed-zero ordering and index ties; its first 960 coordinates exactly match the saved order.

For each coordinate, let `h` be the minimum of the focus gradient times `(legal quarter-code − original fractional master)` over all alternative legal codes. The descriptive descent estimate is `B = max(0, −h)`. This uses the original local gradient; it is not a measured native gain or a feasibility test.

| Population | Coordinates | Nonzero focus gradient | Sum of B |
| --- | ---: | ---: | ---: |
| Selected | 960 | 309 | 0.0036832492796889937 |
| Unselected | 56,640 | 16,048 | 0.00028586714556766424 |
| Complete family | 57,600 | 16,357 | 0.003969116425256658 |

Thus 92.79771327067639% of the available initial estimate lies in the examined coordinates. Raw nonzero support, absolute gradient mass and L2 coverage are not equivalent: saturation and legal displacement matter. Direction counters in the original attribution report include nonzero-gradient coordinates with `B=0`; those counters must not be interpreted as suppressed available descent. The separate direction supplement restricts the comparison to `B>0`, distinguishes aggregate utility at the focus-preferred legal destination, and measures the ordered-f32 summation residual. This preserves the original completed report rather than editing its evidence. There are 648 coordinates with available descent: 293 selected and 355 unselected. Only one has aggregate utility opposing the focus-preferred destination, accounting for **0.52525081466297%** of the total available focus descent. The ordered-f32 sum matches the saved aggregate bit-for-bit; its maximum residual from the widened exact sum is 2.867508896997606e-9. These remain initial local-gradient statistics.

These observations do not establish global pair capacity or impossibility, native feasibility for unexamined coordinates, or that the original gradient remains accurate after hundreds of commits. They do show that no winning focus correction reached the gates in this finite pass despite substantial initial credit coverage. Another unchanged pass, expanded coordinate cap, or automatic combination with the coupled candidate is not justified by these findings.

## Evidence, execution and preservation

[Attribution report](report.json) and [direction supplement](direction-report.json) retain source/input/script hashes and measured cost. The source is `generate_episode_learning.rs` at `a66ecef50848cc11160fc0c5a1f188355731ff98`, SHA256 `30ac874b799a41cbfc57893944ac82cf7ac0c4a2681d2d95747da08017b2c769`; its bytes remain unchanged on the refreshed delivery base. The prior independent finite-arithmetic audit is reused, not rerun. This reader authenticates the saved inputs and derives the attribution; it does not reconstruct the encoder, backwards or full native pool.

The complete #2055 outcome was freshly restored from iCloud and all 546 files verified against its SHA256 inventory. Normative Rust complete-file/BLAKE3 verification also passed for the model and prior audit roots. [Recovery receipt](recovery.json) and [report boundary](report-boundary.json) distinguish these checks. A tiny offline CLI adapter calls the repository's unchanged `report_output` module (reexported by core) for exclusive claim, seal and verification; its two existing tests pass. The restored older Mac incidence executable had no standalone report commands, so the adapter build is retained and charged. It is not a new model tool or runtime dependency.

The attribution reader completed in 2.7155 seconds at 190,873,600 bytes peak RSS, using one worker. The report adapter built in 10.00 seconds at 290,013,184 bytes peak RSS. Python files are archival evidence readers only. [Cumulative ledger](resource-ledger.json) includes recovery, preparation, review and delivery, and chains the earlier charges without reset. [Preservation](preservation.json) records the exact sealed roots and verified iCloud copy. No pod was provisioned. Local inputs/builds and the temporary delivery branch/worktree are removed after protected delivery and fresh-main verification.

## Next decision, after merge and cleanup

The next discriminator is an optimistic **legal pair-family pooled-mass bound at this fixed conditional state**: with Source, Prefix, Cue, donor/post-state, Generate unary/bias/prototypes, physical Copy energies and U fixed, can target 267 possibly beat the immutable Copy contribution of rival 307 anywhere in the legal pair box?

The bound must authenticate the current target pair contribution, replace it with its maximum legal contribution, retain frozen U and native clipping, and include all target Copy aliases. Compare that upper target mass with the rival's Copy-only lower mass using the actual native lookup/interpolation and the same pool reference. Cover a conservatively complete set of admissible references, including an off-grid fixed Copy maximum. Raw score gaps or a single current-reference evaluation cannot establish this bound.

If the target upper mass is strictly smaller for every admissible reference, pair-only correction is excluded for this fixed frame and legal box; change the frozen competition or its learned coupling instead of widening the same search. If the bound survives, immutable Copy alone does not exclude correction: coordinated finite updates, other competitors and 380-guard preservation remain unresolved. Equality cannot establish exclusion because native ties prefer the smaller token ID and 267 < 307. Neither outcome admits a new fit automatically, changes serving rules, or qualifies an autoregressive answer. The bound itself is **NOT_RUN** in this deliverable.
