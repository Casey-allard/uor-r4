# Integer gate and retained identity inside geometric attention

This continues the [hard capture/hold bridge](geometric-attention-hard-bridge-2026-10-01.md) on #1512, preserving all successful soft/hard parents and negatives. The same saved native geometric stack receives an actual integer-produced prior identity. No fitting or changed scorer/candidate admission is introduced.

## Implemented boundary

`uor-r4-integer::identity_latch` implements a validated signed-four-bit gate and preallocated captured/previous vector registers. Each signed i16 vector has its own exponent. Two gate halves have independent dyadic weight scales; bias is a separate signed i64 scalar with its own exponent. Dot products use shift/add arithmetic. Nonzero dot/bias terms align exactly to their finest exponent in checked i128, with no accumulator rounding or saturation; a zero sum captures. Unsupported width/exponent/code or unrepresentable arithmetic returns a focused error. Failure commits neither register.

The current row reads the old identity. CAPTURE copies both current mantissas and exponent for the next row; Held HOLD preserves both, Local noncapture clears to canonical zero. Previous input advances after every successful step. Construction allocates; numerical evaluate/step do not. Width admission1..4096 and input/weight/bias exponents[-64,64] are component limits; unrepresentable alignment still returns overflow. The runtime accepts coefficients[-8,7], while this exporter uses the symmetric[-7,7] subset. Coefficients are stored as i8,64bytes for this64-input gate, not packed half-byte payload. Two32-element i16 registers have128bytes element payload; bias/scales/container overhead are additional. No dense-access or whole-serving efficiency claim follows.

The exporter uses existing public `Grouped4BitRow::encode_f32` with one32-element group per gate half. Its weights-only maxabs scale chooses a power-of-two exponent clamped[-24,16], with nearest ties away from zero. Bias is fixedQ24 with the same tie rule. Input quantization chooses the finest nonclipping i16 dyadic exponent from that token's magnitude only, also nearest ties away. This is explicitly distinct from canonical stack export's `(16+m)` grid and its shift rounding; no codec/native-normalizer parity is claimed. Input is the actual gained RMS vector, so no gainfolding change is introduced.

Each gate is serialized to the new claimed attempt, independently reloaded, validated, and bound to its saved model weight digest before evaluation. The reference float trunk supplies actual gained inputs. The integer latch's OLD mantissas and captured exponent are reconstructed at an explicit floating boundary and enter the existing q/k identity maps. Current-role maps, values, null/age/scorer, output head and preceding trunk remain floating point. This exercises quantized retained content as well as integer action decisions. Event/source/answer labels observe diagnostics only; they do not enter gate/scales/state.

The per-call replay seam is limited to `rra` without pointer processing, rejects nonfinite/mismatched/nonzero-initial prior tensors, and changes no saved model state. Arbitrary caller-supplied tensors are not certified causal; this evaluator constructs them causally from the actual model inputs. Existing integer export/checkpoint/served refusals remain intact.

## Actual saved-model results

| Model | Original answers | Longer-gap answers | Original / longer-gap head-zero majorities | Action disagreements |
|---|---:|---:|---:|---:|
| Held-s1 | 128 | 128 | 128 / 128 | 0 |
| Held-s2 | 128 | 128 | 128 / 128 | 0 |
| Local-s1 | 47 | 59 | 24 / 37 | 0 |
| Local-s2 | 54 | 49 | 29 / 36 | 0 |

All answer/source counts are out of128. Every model/panel has zero prediction changes, zero answer losses/gains and zero majority changes on either head compared with its hard reference. Both Held seeds retain29/29 original and27/27 longer-gap changing-answer query pairs, and correct distinct-source selection in all3/5 equal-answer pairs. Both are perfect in the four longer-gap cohorts45/45,23/23,32/32,28/28. Those cohort group memberships overlap and are not additive.

Maximum absolute gate-logit error is0.724079/0.718931 for Held seed1 original/stress and1.090899/1.111756 for Held seed2. The maximum Held error1.111756 is below the minimum observed Held reference margin5.591120, explaining sign preservation on these recorded inputs. This is an observed finite-input comparison, not an unseen-input error guarantee. All4models have zero action disagreements across4,168 original and6,224 stress unpadded positions per model.

Maximum retained-content error is6.103515625e-5 (2^-14). Held inputs all use exponent-13, so this is half a quantization step and no additional decay is introduced by HOLD. These successful model panels do not exercise holding across changing input exponents. Local controls use-14/-13; unequal-scale behavior is separately exercised by focused component tests. The largest Held answer-position vocabulary-logit change is2.19345093e-5; largest Held correct-source mass change2.98023224e-6. Minimum Held source mass remains0.981928. Head1 remains without source majorities in Held; no all-head claim follows.

All newly measured hard-reference answer logits and both-head source masses reproduce the prior hard study exactly. Reference outputs remain identical after integer replay. Input/model/source hashes match the saved parents; standalone gate artifacts match executed parameters. Gate exponents are(-4,-3) for Held seed1 and(-3,-3) for the other three; maximum per-weight quantization error is0.059376..0.062318. No answer-dependent scales were selected.

## Verification, cost and decision

Four integer tests pass: signed4/i16 endpoints and maximumwidth exact reference; separate scales/tie; causal Held/Local registers; transactional admission/overflow. Ten focused stack tests pass, including replay equality with an independently constructed hard reference and rejection of bad prior inputs, plus retained soft/hard gradients, causality, metadata, save/reload, checkpoint and export boundaries. They ran with `--profile dev`: functional/numerical checks, no optimized performance claim. Independent mathematical review recomputed raw rows and artifact binding. Touched Rust rustfmt, diff check and claim-wording are the local checks; compatibility CI is not those tests.

Actual evaluation167.799s internally,172.63s processwall,2Rayonthreads,maximum RSS183,664,640bytes,zero training. Initial unoptimized research build6m18s. Complete preparation/build/review/delivery/cleanup is charged separately and cumulatively. Projection90min,evaluation900s,4GiB RAM,7GiB disposablecache,48MiB retained,128MiB storage stopmargin. No paid or SSD compute. This lab removes its clean worktree/cache after protected delivery and preserves the executable and unique evidence.

A direct disassembly check of14 named numerical latch/dyadic symbols in the measured dev executable finds no integer multiply/divide or floating arithmetic in those symbols, but66 `fmov` bit-transfer instructions generated around wide integer values. Generic callees and optimized complete serving are not certified. This is not a PASS of the project's strict production opcode contract. It motivates inspecting the optimized full boundary when integrated; it does not change the arithmetic results or justify refitting the successful mechanism. Parser corrections and disassembly are retained as scoped audit records.

**Decision: retain the integer gate/state component and advance scale-aware current-role plus identity q/k projections.** Preserve full magnitude/orientation and combine wide contributions before declared final rounding. No new soft/hard/integer-latch campaign or fit is justified by these panels. Natural-language capture/span/commit learning, candidate indexing, complete Lorentz/output lowering and whole-path D11/D5 qualification remain open.

Measured source commit `918c50097f7b88349e01a881fbee95a4828e2cb3`; binary SHA256 `6d7ed3ecb14bb9cc6eb2e30b31f2b018ba633d27a8952f7b38d5d09a9b6dc71b`. Embedded commit is UNAVAILABLE; exact source and executable hashes provide binding separately. Retained root `/Users/casey.allard/.local/share/uor-r4/research/attention-integer-latch-20261001` includes all gate artifacts, sealed row outputs, executable, summaries and scoped reviews/audits. [Compact evidence](../evidence/attention-integer-latch-2026-10-01.json) carries the counts and identities. References #1512; this is authored development integer-component fidelity, not general language, complete serving, geometry superiority or energy qualification.
