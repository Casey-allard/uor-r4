# Fixed-decoder recurrent/read pair — #973

**Actual result: FAIL_TREE_STATE_PAIR_DEVELOPMENT_GATE. No promotion; retain 15baec48.** The one exhaustive construction experiment selects the unchanged pair [119,119]. There is no improving setting under this fixed decoder and declared two-entry search.

The [previous decoder selection](native_geometric_decoder_validation_973.md) selected 8f8e5c34 but failed preservation and useful generation. This experiment freezes that decoder and all other parameters while varying only indices 1176 and 1930. Index 1176 is a lane-1 transition entry (TRANSITION1024 + lane offset120 + relative root32). Index 1930 is a lane-1 selected-entry read-value update (READ1744 + lane offset120 + stored value root66). Neither is a direct query/key scoring weight. Their effects can propagate to subsequent state, keys and reads.

All **14,400** root pairs are evaluated with the actual hard causal recurrence across all twelve construction documents / 672 byte-and-EOS positions. Each setting starts each document afresh. Saved states are not used as substitute trajectories. Selection uses the global numerical minimum NLL, preferring the unchanged parent within 1e-10 and otherwise lexicographic roots. Opened and disabled-control labels are excluded from selection. The complete attempt takes **7,331 ms internally**; the supervised command takes 7,585 ms. No decoder refit or repeated enumeration runs.

| Setting at indices 1176 / 1930 | Full construction NLL | Correct / 672 |
| --- | ---: | ---: |
| **119 / 119, unchanged** | **2.839534** | **166** |
| 92 / 119, next lowest loss | 3.029702 | 152 |
| 119 / 81 | 3.030433 | 154 |

The unchanged pair is the only setting within the minimum tolerance. Zero settings improve NLL, and none exceeds its construction correctness count. This establishes a complete finite, conditional numerical result for these two entries with this decoder fixed. NLL uses f64/libm; this is not an exact-real transcendental certificate, a global recurrent optimum, a geometric capacity limit or evidence that the broader learner cannot improve.

Witness **blake3:1b33feeb029c00718a283be2c05e7524abb802f09c45d4836bff2327d8c8dbef** binds exact parent 8f8e5c34, runtime/driver, design/corpus and selected pair. It reloads and reproduces the recorded metrics. It changes **zero parameters**; no new normal model artifact is exported. All evidence names the witness separately from its parent. Every decoder, cap, geometry and non-pair field remains unchanged. Production training, serving and artifact source are unchanged; this step adds only a Rust experiment driver and its focused test.

| Panel | Witness correct | Witness NLL | Lost correct vs 8f8 / 1198 / 0f82 / ade9 |
| --- | ---: | ---: | --- |
| Construction Full | 166/672 | 2.839534 | 0 / 74 / 47 / 43 |
| Construction ContextDisabled | 126/672 | 3.356711 | 0 / 38 / 34 / 30 |
| Construction StateDisabled | 69/672 | 3.943612 | 0 / 33 / 48 / 51 |
| Opened Full | 52/381 | 3.653604 | 0 / 20 / 20 / 13 |
| Opened ContextDisabled | 49/381 | 3.737585 | 0 / 21 / 15 / 14 |
| Opened StateDisabled | 49/381 | 3.790113 | 0 / 13 / 31 / 29 |

Construction optimization fails because there is no strict improvement. Preservation fails against the complete prior set: totals **0 / 199 / 195 / 180**, compared separately. These are inherited differences between the unchanged 8f8 decoder and earlier artifacts, not new regressions introduced by this step. Opened Full remains 52/381 at NLL 3.653604, below previous-tree accuracy 57/381; the all-prior opened gate remains failed.

Every state/root and source trace remains identical, and witness prediction Hamming against 8f8 is zero across all six panels. Prediction Hamming totals against 1198 / 0f82 / ade9 are 1,889 / 2,750 / 2,783. Hamming here counts unequal aligned categorical identities; token/hash bit distance is not a semantic metric. All **12,636** prior saved rows (3,159 per artifact) and all **48** prior generated outputs replay exactly.

All four witness Full continuations equal 8f8's incoherent outputs and reach 96 bytes without EOS. The fact prompt does not produce a useful answer and the Rust prompt does not produce a usable program. The attempt preserves all 60 prior/witness prompt/control outputs plus immediate post-reload Full generation. No language, reasoning, Rust coding or fresh qualification is established.

The result closes this fixed-head pair question. It leaves open whether a state change becomes useful when its decoder is relearned on the resulting trajectories. The current head was fitted to the parent's states, so coupled state/decoder adaptation is a specific next hypothesis. The current result does not demonstrate that coadaptation caused failure. A bounded conditional refit profile is more directly connected to that question than repeatedly searching the same fixed-head pair or selecting new parameters without evidence.

**Next action (NOT_RUN):** One bounded coupled state/emitter coordinate profile: the unchanged pair plus 119 alternatives for index 1176 and 119 alternatives for index 1930, changing only one coordinate at a time from [119,119] (239 unique settings). For each setting, recompute complete construction trajectories and fit one conditional angular emitter with fixed depth3/minimum-leaf16 and identical inherited caps. First require the unchanged baseline refit to reproduce exact 8f8e5c34; otherwise stop and diagnose. Freeze the full fit-call, wall/RAM/storage limits and selection/tie rule before execution. Select once using construction NLL, then run one immediate generation and all prior/control comparisons. Bind both state overrides and the actual fitted head in valid provenance or an explicit test-only witness; never identify a changed clone by an inherited CID. This tests state/decoder coadaptation, not a full coupled pair optimum. No new parameter hunt, decoder regularization family, fresh draw, iterative alternating campaign or promotion is authorized by this recommendation.

The source review reuses the [SpiralCore/FBS research](../research/spiralcore-v68/README.md) distinction between intrinsic state/route history and emitted observations, the local knowledge map and the existing [complete-objective driver](../crates/uor-r4-core/src/native_geometric/shared_core/tests/full_objective.rs). No donor mechanism or Bell/IP semantic weights are imported. This step needs no new literature claim; prior methodological references retain their original scope.

Executed validation: offline release compilation and the focused witness-binding/forbidden-field/tie-selection test pass; the actual sealed experiment passes execution checks while the model gate fails. A 512-step witness runtime census observes at most **23** angular comparisons per prediction, below the bound of 27. Allocation measurement is **NOT_RUN in this step**; the earlier unchanged-parent allocation result remains a separate receipt. No blanket suite, complete-path energy measurement or retained-session integration runs. Final formatting, claim-wording, link and source/report identity checks are recorded in the evidence.

[Tracked evidence](evidence/native_geometric_tree_state_pair_973.json) binds exact source/compiler/binary, design, full numerical table, witness, all rows/outputs, reviews and resources. Local originals remain in shared-core-first-step/tree-state-pair-1 under the established handoff; attempt-1 is sealed. Fourteen prior sealed roots / 158 files, the 22 parked dirty paths and retained model verify unchanged. Earlier V3–V7 and model fits are preserved, not replayed for onboarding.

Model/build/test charge 104,429/300,000 ms; parent cycle 2,261,151/2,600,000 ms; shared cumulative ledger 122,011,724/132,950,000 ms. A 200,000 ms local-cycle increase was recorded before use under standing owner authorization; the shared ceiling is unchanged. Added storage 26,853,376 bytes within 128 MiB; parent growth 1,518,923,776 bytes within 2 GiB with a 128 MiB stop margin. Peak sampled RSS 2,367,373,312 bytes. No paid compute or cleanup. Final engineering and delivery receipts append locally.
