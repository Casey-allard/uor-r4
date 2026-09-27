# Historical dialogue artifact: native integer observation, September 27, 2026

The explicit width-576 profile now executes the retained R1d model through conversion, packed reload and persistent native integer dialogue. **Keep this as a diagnostic artifact, not a promoted language model.** The largest observed numerical change comes from parameter discretization. Quantized float and integer outputs agree on 55 of 58 turns, yet lose some of the weak parent's retained memory payloads. The integer implementation also supplies no observed speed advantage in this run.

This closes the actual-artifact gap left by the [profile implementation](native-dialogue576-preparation-2026-09-27.md). The [portable evidence packet](../evidence/native-dialogue576-observation-2026-09-27.json) includes all 174 actual replies with selected IDs and stops, numerical summaries, source/binary/input identities, independent review bindings and bindings to complete execution receipts. This observation uses the historical R1d parent, **not** the newly fitted [complete-prefix candidate](dialogue-prefix-paired-result-2026-09-27.md).

The parent has 5,429,826 parameter coordinates, Quaternion transport, Dot reading, Full causal admission, width 576, read width 64, vocabulary 4,096 and context 256. The bridge and pack binary are frozen at source `e75acd94281a4bfff99ece8e5228ea76a1647265`; the observer is `b9bea957df86c27f8a94bca966c15e27eb1b854d`, release with `cpu-accelerate`. Conversion calibrates parameter scales and exports the actual hard values with zero new optimizer/model updates. The historical step-2,237 preparation clock does not turn calibration into quantization-aware training.

Four forms process exactly the same 4,331 saved input positions, resetting state per turn:

- **FF:** the verified continuous parent, with continuous parameters and interfaces.
- **QF:** actual decoded packed parameters with continuous interfaces; no recovered shadow weights, recalibration or second rounding.
- **QQ:** the same loaded hard parameters with the declared quantized interfaces in the offline float evaluator.
- **Integer:** the explicit development bundle, using the native multiplier-free numerical path.

Each form has 1,766 assistant targets and 2,565 other next-token targets. The last generated token is scored but not fed back. The 1,766 assistant targets are the weak parent's saved continuations; the other 2,565 targets come from prompt/history replay. These losses are numerical diagnostics, not an answer oracle, corpus likelihood estimate or language-quality score. Historical held-out files are hashed for import integrity but are not scored here.

| Ordered comparison | Mean half-L1 probability difference | Greedy changes / 4,331 positions | Changes / 1,766 assistant targets |
|---|---:|---:|---:|
| FF → QF | 0.138392909 | 746 | 247 |
| QF → QQ | 0.000975525 | 6 | 3 |
| QQ → integer | 0.000803991 | 8 | 2 |

The parameter change is already visible at initial BOS, before a memory read: half-L1 0.140714, maximum probability difference 0.0486459, and NoRead mass 1 in both forms. Its subsequent recurrent interactions are included; this does not identify a particular tensor or a unique additive fraction of the final error. First assistant-target divergence is the second greeting selection, with FF token 915 versus 745 in the other forms. The first QQ/integer argmax difference instead has only about 2.37e-9 maximum probability difference and equal state, so an argmax difference alone is not proof of a large kernel defect.

All integer distributions have exact Q48 total mass. No saved target has zero probability. Float mass drift is retained without renormalization; its observed maximum is about 2.39e-6. State and gate comparisons decode the actual Q11 and Q15 grids. Saved-target assistant NLL is 1.0710394 FF, 1.2386796 QF, 1.2387833 QQ and 1.2388022 integer, with the weak-target limitation above. No empirical acceptance epsilon was invented.

Actual generation uses the existing 38 requests and 58 turns for each of FF, QQ and integer: 174 replies total, greedy selection, genuine EOS, the same 32-token cap and short-cycle controller. FF reproduces all 58 historical responses exactly in selected IDs, text, stops and prompt IDs. QQ and integer have identical current prompt IDs and stop reasons on all 58 turns; selected IDs/text differ on greeting 3, the music sentence request, and Tokyo final recall. Equal aggregate selection counts are not output parity.

The integer conversation retains its state and pending token. Float generation replays each complete prefix. Twenty raw history-receipt differences are expected: float records caller EOS after a nonfinal capped/cycled turn, while integer records it at the next request. Exact next-request prompts match. This is not a context-loss finding.

Principal and independent review read every output. The converted model remains weak:

- Momo is lost across all three cat turns, which become “Yes, there are named 1000”. The final blue-car reply also loses the requested color.
- Alex and pizza appear in looping final replies where historical FF omitted them. These local gains coexist with losses; they are not reliable conversation improvement.
- Integer alone mentions Tokyo in a malformed final reply relative to QQ. The earlier sister replies omit it, and the added visiting fact is not handled.
- All eight factual requests remain unanswered. Greetings, sentence instructions and ambiguity requests remain poor; repeated topic words are not fulfilled requests.
- FF ends with 11 EOS, 43 caps and 4 cycles. QQ and integer each have 3 EOS, 47 caps and 8 cycles. Stop category remains separate from usefulness.

The run performs 17,324 common-input trace steps and 11,207 generation steps, totaling 28,531 model steps and 5,050 selections including EOS. No learning, backpropagation or observer recalibration occurs.

| Cost component | Observed seconds | Scope |
|---|---:|---|
| Conversion command | 2.062 | Includes supervisor polling; parameter calibration, export and reload |
| Pack command | 1.031 | Includes supervisor polling and bundle construction |
| Observer command | 57.822 | Includes preparation, numerical traces, generation and sealing |
| Complete three-command supervisor | 61.469 | Includes verification and closeout; do not add the rows above again |
| Observer checks/build, both attempts | 218.198 | Includes the preserved first metadata-macro compile failure and focused repair |

Peak actual child RSS is 161,349,632 bytes, and new report material is 29,784,043 bytes under the 64 MiB projection. All four focused checks pass: packed parameter preservation/guards, trace target alignment, finite/zero-probability denominators and explicit caller closure. The first attempt's library test remains valid; only the affected example checks/build were repeated after parenthesizing a metadata iterator expression. No unrelated warnings were repaired.

For the same 4,331 trace inputs, recorded model stepping is 3.173 seconds FF, 3.057 QF, 4.032 QQ and 13.983 integer. This is a single fixed-order descriptive run with diagnostic instrumentation, not a controlled performance benchmark. Generation also performs different work: FF/QQ replay history, while integer persists it. The current implementation does not demonstrate a speed advantage; energy was not measured. Dense access to the model's parameter arrays remains, so packing and avoiding multiplication do not establish D5 selected access or consumer-device efficiency.

Preserve all sealed artifacts and exact negative outputs. The next numerical question is parameter discretization; first reconcile the existing precision/learned-rounding evidence and this retained parent's actual scales instead of launching a new rounding or training sweep. The already prepared packed-coefficient workload comparison can advance independently and measure its exact loaded-output/cost tradeoff. No general chat, geometric advantage, fresh-held-out generalization or complete-path energy result follows from this observation. Owner-confirmed native, multiplier-free serving remains the target.
