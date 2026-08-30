# R4 softmax teacher trace and first source-free student — #973

Date: 2026-08-30 (EDT)

Bound implementation revision:
`3cbeab64ba40f4445f878741b87ec6a487fcee00`.

Terminal:
`PASS_SOURCE_FREE_TRACE_STUDENT_ADVANCE_GEOMETRIC_STATE_COMPILER`

This record appends the first completed source-free compilation rung after the
qualified `R4SoftmaxReferenceGeneratorV1` and its native loopback bridge. It
does not revise the earlier source-backed attention result.

## What ran

`R4SoftmaxTeacherTraceV1` is a transparent compiler-side decorator around the
accepted coherent `R4SpinCausalAttentionTransport`. It preserves ordinary
learned Q/K/V, RoPE, scaled dot product, stable causal softmax, weighted-value
aggregation, `W_o`, residual/MLP blocks, and the language-model head. At the
existing transport seam it records:

- complete pre-RoPE projected Q/K/V rows;
- query-gauge Q and current transported K/V;
- bounded top-8 causal attention support with transported K/V;
- weighted query-gauge aggregate and decoded model-frame head output; and
- top-32 logits, full-vocabulary log-sum-exp, target logit, and target NLL.

The construction-only trace compiler then emits one canonical 39,648-byte Q16
suffix artifact with three equal-support arms:

1. teacher-distilled top-32 distributions;
2. observed next-token counts; and
3. a deterministic cyclic document-permuted teacher control.

The source-free prediction and continuation path uses only integer comparison,
addition, and table reads over the artifact and token history. It does not load
or call the source model, execute softmax, allocate teacher state, sample, or
use floating point. The separate evaluation methods, compiler-side trace
capture, and scoring use ordinary floating-point arithmetic; trace capture also
uses the source decoder.

## Frozen split and reachability

The #989/#953 SimpleWiki-derived D3 partition was retained:

- construction IDs `14`, `657`, `4579`, and `5121`: 38 positions;
- held-out ID `13`: 57 positions;
- total source forwards: exactly 95; and
- construction and held-out text CIDs are disjoint.

The pinned tokenizer produced the exact held-out longest-suffix histogram
`[47, 8, 1, 1, 0]` for depths `0..=4`. Ten positions had a nonzero suffix; one
was the shared BOS token, leaving nine context-bearing positions. Two positions
reached depth two or deeper, one reached depth three, and depth four was an
expected-zero control. The result therefore cannot support a broad language or
generation claim.

Preflight CID:
`blake3:f48916d2789d026b88a03a40ec20cb326b12e29debff541dfb9b00a43915b2b7`.

## Construction freeze

Construction and reveal were separate invocations. The first invocation saw
only construction rows, wrote the trace and student, reloaded the artifact,
and sealed a construction-only manifest before any held-out teacher step.

- construction manifest CID:
  `blake3:d3aa26f40171651fb808c2f53c11a493507fecca6591ad2379765627a2fd46f3`;
- trace bundle: 45,205,493 bytes,
  `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`;
- student artifact: 39,648 bytes and 97 rows,
  `blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`;
- rows at depths `0..=4`: `[1, 21, 26, 25, 24]`;
- canonical reload: byte-identical and CID-identical; and
- freeze CID:
  `blake3:bb19fc6f6976aca6dfd8c67c470fd1fb70a1e1e74763800fdcb635f135325df7`.

Construction completed in `145.165836292` seconds. The complete sealed freeze
is tracked as
[`r4_softmax_trace_student_freeze_973.json`](r4_softmax_trace_student_freeze_973.json).

All 38 construction forwards used the requested and effective eight-worker
exact backend; every forward recorded multiworker execution and the measured
maximum was eight active workers. Every document's causal, projection, and R4
audit exactly matched its arithmetic expectation, with zero future reads.

The held-out-derived preflight CID, held-out text identity, and held-out teacher
outputs were not compiler inputs and are not bound into the artifact. Document
`13` plaintext and shifted token targets were intentionally known to the
tokenizer-only reachability preflight; only its teacher outputs were held back
until after artifact freeze. This is construction-disjoint,
teacher-output-held-back transfer, not blind unseen-text generalization.

## Held-out result

The reveal revalidated the freeze, trace, and artifact CIDs before executing
held-out document `13`. All 57 positions selected all 30 decoder layers. The
observed 446,310 K transports, 446,310 V transports, projection census, and
14,528,160 encoded R4 blocks exactly matched their expected values; future
reads were zero.

On the nine non-BOS context-bearing positions, the values below are
cross-entropy conditional on the teacher's renormalized top-32 support and then
restricted to its intersection with the matched student-row support. They are
not full-vocabulary next-token loss:

| Arm | covered teacher CE, nats | teacher top-1 | actual-next top-1 |
|---|---:|---:|---:|
| Teacher-distilled | **2.660721** | **3/9** | **2/9** |
| Observed count | 9.678894 | 2/9 | **2/9** |
| Document-permuted teacher | 4.342019 | 1/9 | 1/9 |

Across all 57 positions, the teacher-distilled arm also had lower
shared-support covered CE (`2.718054`) than count (`7.473034`) and permuted
control (`3.277220`), with teacher top-1 `8/57` versus `6/57` and `6/57`.
Only 1,327,607 of 3,735,495 bounded teacher Q16 mass was present in the shared
student row support across all positions, so full teacher cross-entropy is
truthfully unavailable. The comparison above is restricted to equal shared
support.

Artifact bytes and CID were unchanged across reveal. Reloaded integer-runtime
evaluation and continuation replayed exactly, and the source execution snapshot
did not change during student prediction.

Result CID:
`blake3:e48b4172e02fc84eef9e00024ac6602b790d8230026a979d2f71c552ddca0cd4`.
Reveal completed in `204.6753085` seconds.
The structured result is
[`r4_softmax_trace_student_973_raw.json`](r4_softmax_trace_student_973_raw.json).

The canonical student artifact is tracked in transport-safe base64 as
[`r4_softmax_trace_student_artifact_973.b64`](r4_softmax_trace_student_artifact_973.b64).
Decode it with `base64 --decode` (GNU) or `base64 -D` (macOS); the reconstructed
39,648 bytes must hash to
`blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`.
The 45,205,493-byte construction teacher trace remains an ignored local
compiler artifact bound by the freeze CID; it is not needed by the source-free
runtime and is not committed merely to make the repository larger.

## Decoded behavior and decision

The source-free prompt `He was born` decoded as:

> ` in , Scotland, Scotland, Scotland, Scotland, Scotland, Scotland, Scotland`

This is deterministic but not coherent generative text. The bounded mechanism
gate is positive because the frozen distilled student beat both declared
controls on covered teacher loss and teacher top-1, did not lose actual-token
top-1, survived the causal and artifact audits, and made zero source calls at
student prediction. The product/generation interpretation is negative: a
suffix table over 38 construction positions collapses after leaving its sparse
support.

The result establishes:

- a complete, causal, canonical R4/Spin softmax teacher-trace seam;
- a frozen construction-before-reveal compiler boundary; and
- deterministic bounded source-free prediction/continuation that transfers
  more of the teacher distribution than matched count and permuted controls.

It does **not** establish geometry advantage in the student, coherent general
generation, reasoning, correctness, softmax replacement, a Transformer-free
architecture, production R4G1/WASM lowering, or release readiness.

## Next authorized rung

Build one `R4SoftmaxTraceStateStudentV1` construction experiment from the
already captured geometric trace. Construction-time fitting may use the
recorded query-gauge Q, transported K/V support, weighted aggregate, and
decoded model-frame output as targets/codebook-induction data—not merely token
suffixes. At held-out and runtime execution, the bounded state transition may
read only prior state, the observed token ID, and independently available
canonical R4/Spin address/frame data. It may not read held-out Q/K/V,
aggregates, decoded heads, source weights, or source traces. The experiment
must include a field-by-field runtime-input provenance audit and compare
equal-budget arms:

- the 39,648-byte suffix student established here;
- a non-geometric recurrent state with the same state/readout budget;
- the geometric trace-state student; and
- a transport/state-permuted control.

Promotion requires shared-support covered teacher CE below `2.660721` and both
matched arms on the frozen nine-position context-bearing support, teacher
top-1 greater than `3/9`, actual-next top-1 greater than `2/9`, a material loss
under the geometry-destroying control, exact causal replay, zero source calls,
and decoded continuation that does not enter the observed period-1/2 short
cycle. Full-vocabulary next-token NLL remains unavailable unless support is
expanded under a newly frozen, leak-free contract.
If it fails, repair the state/transition representation on this split. Do not
resume intrinsic-score variants, resonance replacement, corpus scaling,
reasoning claims, WASM promotion, or release work.

## Hosted-page boundary

GitHub Pages remains a static WASM route-inspection surface. It has no native
source-model backend, and this student has not been lowered into the WASM
runtime. The page's current offline/non-working chat state is therefore not
changed by this result. A truthful static-page wording repair and a local
one-command native launcher are separate product work; hosted generation waits
for a qualified source-free artifact and WASM/runtime integration.

Live inspection on 2026-08-30 returned HTTP `200` for the Pages document but
HTTP `404` for both `/uor-r4/r4_worker.js` and `/uor-r4/api/chat`; the page source
attempts to load that worker and call `/api/chat`. The deploy workflow copies
the generated `pkg/` directory but not `r4_worker.js`, and GitHub Pages cannot
provide the native API. This is a concrete deployment defect, not evidence
against the attention result.
