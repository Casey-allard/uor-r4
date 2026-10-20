# Active geometric parameter sensitivity — #973

**Actual gate: FAIL_ACTIVE_SENSITIVITY_DEVELOPMENT_GATE. No model promotion; retain 15baec48.** The complete baseline-active single-coordinate scan finds 213 construction-loss improvements across 49 entries. The selected minimum changes READ index 1937 from root 119 to 107, improves construction loss by 0.114739%, and gains two correct predictions. It fails preservation and opened-development gates; free generation remains incoherent and unterminated.

## What was changed and measured

The preceding coupled profile exhausted two particular coordinate directions. This test-only Rust driver traces all baseline accesses to TRANSITION, QUERY, KEY and READ and evaluates every alternative root for each active entry, holding the selected 8f8e5c34 decoder, all other parameters and geometry fixed. It addresses whether useful coordinate directions exist outside the historical pair. It does not fit a new decoder, add a runtime mechanism or train a new normal model artifact.

The test-side observe mirror calls native geometric primitives and checks exact prediction, target likelihood, complete state (including ring, phase and source) and work equality against the actual kernel at every byte. A focused test varies parameter values across all four families, exercises all five interventions and ring overwrite, validates family visit counts and inactive-entry invariance. Every alternative is evaluated using the actual kernel from each document's initial state through every byte/EOS position; cached baseline states do not substitute for recurrence.

| Family | Domain entries | Active entries | Visits | Alternative settings | Lower-loss settings |
| --- | ---: | ---: | ---: | ---: | ---: |
| TRANSITION | 480 | 477 | 2640 | 56763 | 4 |
| QUERY | 120 | 119 | 660 | 14161 | 10 |
| KEY | 120 | 118 | 660 | 14042 | 58 |
| READ | 480 | 436 | 2248 | 51884 | 141 |

The 1,150 active entries yield 136,851 settings including the baseline. Exactly 136,612 new full-construction proposal evaluations run; 238 settings reuse the two sealed old coordinate slices after exact parent/artifact/source/driver/corpus/loss bindings and baseline reproduction. The 50 inactive entries are excluded by the single-intervention argument: before its first read, changing an unread entry cannot alter state to create that read. This argument does not cover simultaneous changes or another parent. Visit counts measure coverage, not causal credit.

Coverage, exact call count, source/data identities and limits are written exclusively and synced before alternatives. Selection uses complete construction NLL, tolerance 1e-10, unchanged-parent preference and then index/root ordering. No opened or disabled-control labels select the candidate. Only one chosen setting receives generation and the full gate; the other 212 improving alternatives have not received full qualification.

## Actual selected behavior

Witness blake3:aaa1abe1ce2c4b2b1af65c73f43e2b69e410622dd897395c8a4527abb5fb8337 binds source, driver, parent, corpus, frozen design, coverage and the single parameter override. It is an explicit test-only witness; no changed clone is exported under an inherited candidate CID. Replay checks all other artifact fields, including the decoder, remain exact.

| Panel and control | Parent correct | Selected correct | Parent NLL | Selected NLL | Newly lost correct rows |
| --- | ---: | ---: | ---: | ---: | ---: |
| complete_construction / Full | 166/672 | 168/672 | 2.839534 | 2.836276 | 0 |
| complete_construction / ContextDisabled | 126/672 | 126/672 | 3.356711 | 3.356711 | 0 |
| complete_construction / StateDisabled | 69/672 | 69/672 | 3.943612 | 3.943612 | 0 |
| opened_development / Full | 52/381 | 52/381 | 3.653604 | 3.662621 | 7 |
| opened_development / ContextDisabled | 49/381 | 49/381 | 3.737585 | 3.737585 | 0 |
| opened_development / StateDisabled | 49/381 | 49/381 | 3.790113 | 3.790113 | 0 |

Index 1937 is READ base 1744 + lane-1 offset 120 + stored root 73. Its baseline value is 119 and it is used once in Full construction. All construction differences occur in document 11, the lake conversation, at its final five targets (positions 64–68 including EOS). The two new correct predictions are period and newline at positions 66/67. All other construction documents and both disabled-control panels remain exact. This is a narrow tail fit.

Opened Full changes two documents: bird/lake prose from position 16 and Rust text from position 13. Their summed NLL changes are -1.253196 and +4.688888 respectively; all seven new correct-to-wrong changes occur here, accompanied by seven gains. Overall opened accuracy therefore remains 52/381 while NLL worsens from 3.653604 to 3.662621. State and source effects propagate well beyond the changed read.

Across all six panels, lost-correct counts versus 8f8/1198/0f82/ade9 are 7/205/198/183; gains are 9/154/416/408. Prediction Hamming is 73/1919/2756/2791, root-identity Hamming 317 against each prior and source-identity Hamming 81 against each prior. These are aligned categorical disagreements, not semantic distances. Existing differences from older heads remain distinct from seven new losses versus 8f8.

The four immediate Full continuations and their final replay all reach 96 bytes without EOS and remain incoherent. The Rust prompt produces no usable program. No useful language or coding behavior is established. The 11-file sealed attempt contains coverage, the full 136,851-setting profile, witness, actual rows, 60 final outputs, four early outputs, runtime census and source/manifest.

## Validation, preservation and resources

Release compilation and the focused trace/lineage/coverage/selection/reuse test pass, as does execution of the actual experiment. The model gate fails. All 12,636 prior saved rows and 48 outputs replay exactly. The 512-step runtime census observes at most 22 prediction angular comparisons against bound 27; allocation measurement is NOT_RUN this step. Production training/runtime/artifact sources and exact geometry are unchanged. No fresh holdout, full suite, general-language/Rust qualification or complete-path energy measurement ran. Queue status names are compatibility acknowledgements only.

[Tracked evidence](evidence/native_geometric_active_sensitivity_973.json) binds 137 source files, compiler and pinned binary, all reports, research/review, actual deltas and resource receipts. Sixteen older sealed roots / 657 files, all 22 parked dirty paths and retained model hash verify unchanged. The active worktree is /Users/casey.allard/uor-r4-worktrees/shared-geometric-core on codex/geometric-active-sensitivity; original artifacts and task navigation remain in the established handoff. The accumulated repair and V3–V7 stay parked.

Model/build/test charge is 166852/420,000 ms: 100,193 ms release build/focused test and 66,659 ms actual command (66,397 ms internal experiment). Parent cycle 2539118/2,800,000 ms; shared ledger 122289691/132,950,000 ms. Added storage at execution close 50954240 bytes within 128 MiB; parent growth 1604321280 bytes within 2 GiB with a 128 MiB stop margin. Peak sampled RSS 2464940032 bytes. No allowance increase, paid compute or cleanup. Final engineering and delivery receipts append locally.

## Research and next decision

The local knowledge map and scoped credit-assignment/tied-learning records support complete tied trajectories and separating access coverage from credit. Existing SpiralCore/FBS research distinguishes intrinsic state/route effects from emitted results; no Bell/IP semantic weights or donor runtime are imported. [JEPA research](native_geometric_jepa_research_973.md) remains a candidate auxiliary objective, not implemented or run here.

The broader scan disproves absence of local improving directions, while demonstrating that its best update has only one construction occurrence supporting it. Among the 49 improving entries, 26 have one baseline visit and 36 have at most two. This supports ending isolated coordinate harvesting and addressing predictive support across related contexts; it does not prove a global optimum, a geometric capacity limit or that JEPA is necessary.

**Next action (NOT_RUN):** One bounded matched construction-context augmentation experiment, before changing the learning objective. Use Rust to prepare three distinct length-matched construction variants per original document, preserving the declared relation/program structure, for 48 documents total. Compare two state-learning arms from exact 8f8e5c34 with its decoder and geometry fixed: varied contexts versus four copies of the original 12 documents. Match byte/EOS exposure, proposal stream, complete-causal objective, full TRANSITION/QUERY/KEY/READ domain and evaluation-call/resource caps; do not shortlist the 213 observed improvements. Freeze the variant rules, source-separated composition probes, acceptance criteria and both complete budgets before either fit. Use valid tree-aware training provenance or explicit source-bound witnesses; record document support descriptively without adding a new penalty. Select each final arm on construction only, then run immediate generation and all prior/control comparisons plus the untouched probes once. This tests broader predictive support under the same objective. No decoder refit, JEPA loss, automatic training campaign or promotion is included; JEPA remains a separate candidate if the matched result warrants an objective change.
