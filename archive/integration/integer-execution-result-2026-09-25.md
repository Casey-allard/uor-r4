# Full-context integer attention and transport — executed result

**Decision: retain the numerical implementation at its measured scope.** The
accepted learned-code parents now execute learned attention, recurrent state,
quaternion/Householder transport and vocabulary/copy probabilities with integer
model-value arithmetic. Both arms meet the unchanged numerical bounds and retain
every previously correct source answer against the matched parent backend.
This completes the numerical bridge work card under #973 / #820. It does not
complete useful language, the standalone serving boundary, or the whole rung4.

[Prospective plan](integer-execution-plan-2026-09-25.md) ·
[bound reports](../evidence/integer-execution-result-2026-09-25.json) ·
[all actual text](integer-execution-outputs-2026-09-25.md) ·
[independent comparison](../evidence/integer-execution-comparison-2026-09-25.json) ·
[instruction inspection](../evidence/integer-execution-instruction-audit-2026-09-25.json) ·
[review adjudication](../evidence/integer-execution-review-2026-09-25.json) ·
[resources](../evidence/integer-execution-closeout-2026-09-25.json).

## What changed

The signed-code loader now exposes the same validated parameter codes directly.
`joint_integer.rs`, `joint_integer_math.rs` and `joint_integer_tables.rs` execute
low-bit affine maps, RMS normalization, input-dependent transport, learned
query/key/value scores with actual occurrence age and NoRead, recurrent updates,
copy and normalized vocabulary output. Nonlinear tables are compiled offline in
Rust and loaded by hash. Q48 probabilities sum exactly to `1 << 48`.

Context remains **256 / 256 / 256** for training provenance, evaluation and
sessions. The full reader exposes all255 previous occurrences at position255;
every tested step checks the exact causal count. A zero resulting weight does
not remove an occurrence from the admitted set. Read-vector width64 and state
width256 are representation dimensions. There is no new fit, code/scale change,
candidate restriction, checkpoint selection or new holdout.

The quaternion path retains ordered R4 transport. The ordinary arm uses its
matched Householder pair. These fixed-point transformations approximate their
real-valued operators; they do not establish exact finite-group closure or
Hamiltonian dynamics. The exact event tape retains occurrences; a fixed-width
state alone is not claimed to encode the entire input reversibly.

## Loaded comparison

The existing evaluator supplies32 source variants, five seeded continuations and
Read/NoRead controls per arm. Numerical comparisons use the same observed tokens
in the **first four256-token development windows: 1,024 targets per mode**.
All belong to the exposed calibration prefix; none belong to the comparison
tail. The full976-window likelihood gate was not rerun or declared passed.

| Read-enabled result | Quaternion | Householder pair |
|---|---:|---:|
| Integer / matched-parent complete source answers | 28/32 /28/32 | 24/32 /24/32 |
| First-noun and complete-answer losses / gains | 0 /0 | 0 /0 |
| Integer complete answers with NoRead | 0/32 | 0/32 |
| Maximum state drift; limit0.01 | 0.00732421875 | 0.005859375 |
| Maximum probability drift; limit0.01 | 0.006691748 | 0.008505233 |
| Integer prefix NLL, nats/target | 2.335653773 | 2.348543489 |
| Integer minus matched F32 prefix NLL | +0.000075379 | −0.000065692 |
| Top1 disagreements on identical input | 2/1,024 | 2/1,024 |
| Integer step time,1,024 calls | 16.526 seconds | 16.475 seconds |
| Matched F32 step time,1,024 calls | 0.581 seconds | 0.570 seconds |

NoRead also meets both0.01 numerical bounds. Every tested integer output and
attention row has the required exact total. The source interventions and NoRead
support learned contextual dependence on this authored panel. They do not show
general comprehension or a unique geometric benefit.

Actual stories remain unreliable. For example, one quaternion continuation
introduces “a dog named Max” and then calls Max “a big cat named Bob”; the ordinary
model turns a goose's food, corn, into an apparent participant. Small numerical
changes also branch seeded continuations, so free-output equality with F32 is
not claimed. The retained sampler itself uses host F32 probabilities and F64
log/exp outside the integer kernel. Source answers use integer greedy selection.

## Comparator correction and preserved failure

Attempt1 mistakenly omitted the accepted evaluator's build features
`metal,cpu-accelerate,reference-accelerate`. Its default CPU reference returned29
quaternion source answers rather than the historical28, and the ordinary
maximum state difference was0.01171875, exceeding0.01. That complete attempt and
its failure remain preserved.

Before attempt2, the specific backend correction and prediction were recorded.
Only the build features changed: source, tables, weights, panels, inputs and
tolerances were fixed. The matched F32 source outputs reproduce the historical
parent token sequences row by row. Integer outputs are compared across both
builds in the independent comparison record. This is a comparator repair, not
an integer parameter adjustment or retrospective relaxation of the bound.
The correct conclusion is backend-specific numerical agreement, not agreement
with every floating accumulation implementation.

## Numerical and performance boundary

Independent inspection of41 compiled symbol ranges /9,438 ARM64 instructions
finds no floating arithmetic/conversion and no hardware multiply/divide on
model values. Five multiply/divide instructions remain for row addresses,
chunk shapes and vector descriptor pointers. A SIMD `fmov` transfers an integer
sum without floating arithmetic. **The entire step/executable is not claimed
multiplier-free.** The inspected numerical-value boundary is narrower.

The model implementation still resides in the offline training crate. Loading
legacy metadata, tokenizer/hash operations, allocation, reports, the F32
comparator and seeded sampling are outside that boundary. Dense parameter
access and dynamic allocation remain. The bridge is roughly28–29 times slower
than F32 for these measured batch1 calls, approximately16.1 milliseconds per
integer step. This timing excludes loading, session allocation, generation
orchestration and serialization; it is not a whole-application or physical
energy measurement. D5 parameter sparsity and energy savings remain open.

## Validation, identity and decision

Executed source: `45da66d4a2e77a908411c7084e109f6a17472114`.
Matched-feature binary SHA256:
`554d95eecc552be72bb2923fbbdba7125df1419c1062546d863dd427cac24b6e`.
Parents remain `learned-rounding-20260925/fit-{quaternion,householder_pair}-rounding-3/packed-model`.
Local reports/binaries/tables remain in
`/Users/casey.allard/uor-r4-investigations/full-context-integer-20260925`.

Ten new focused arithmetic/interface checks and two retained codec checks pass;
optimized compilation and both loaded pairs complete. Each pair takes about112
seconds. No model training occurs. RDC runs both Rust workers concurrently on
this M1 and a substantive DeepSeek numerical review alongside implementation;
independent agents handle arithmetic and evaluation. A client named Kimi routes
to DeepSeek; no Kimi model or external training hardware is used. Two failed
client invocations before model work are recorded. No broad test campaign is
added after the declared risks are resolved.

**Next integrated deliverable:** carry this retained computation into a standalone
Rust serving session with integer token selection, full256 access, explicit
artifact/table binding and measured complete-generation cost. Remove the
remaining floating sampler/runtime dependency from that serving path. Profile
the actual integer operators before choosing any exact-output optimization;
retain predictions and report the cost of dense parameter access. An optimized
implementation can use equality to this retained integer path without repeatedly
refitting the model. Do not reopen admission pruning or tune numerical thresholds.

After that usable serving seam, the next capability work remains coherent
language/state learning at meaningful exposure and the same context contract.
Integer conversion preserves existing weaknesses; it does not solve prose
quality. The ordinary model remains a required comparator. Longer context,
geometric advantage, terminal sparsity, useful conversation/code and lower energy
need their own evidence when those capabilities are actually implemented.
