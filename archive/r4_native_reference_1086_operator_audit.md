# #1086 native reference: independent operator and artifact audit

This is a source audit at repository revision
`eade29f4b78435e9857936786426bb34e596b301`, supporting a specification decision.
It adds no implementation or inference result. The audit read the original
Python/Rust source, the committed #1094 binding and comparison receipts, and
installed Torch source documentation. It imported no model/runtime package,
read no model or population payload, deserialized no tensor, changed no
permissions, and ran no fit, evaluation or replay. Source byte identities and
precise locators are in the [source manifest](r4_native_reference_1086_operator_sources.json).

The accepted reference is `R4LanguageInterfaceInference(execution="r4")` around
`LanguageInterfaceModel`, with the learned reader and frozen compound core.
`text_clause_adapter/worker.py:332-383` chooses this path explicitly. Porting
only the plain learned model would omit both accepted transport stages.

## Exact state and file identities

The following tensor names and shapes are derived from the original module
constructors and loader contract. Accepted #1094 receipts bind loaded state
CIDs, f32 parameter validation and parameter counts. This audit does not
independently extract shapes or values from the model files; a future exporter
must check the actual serialized inventory before producing an artifact.

| State | Exact serialized tensor name | Shape | Dtype |
|---|---|---|---|
| Reader | `embedding.weight` | `[4096,32]` | f32 |
| Reader | `context.weight` | `[64,32,5]` | f32 |
| Reader | `context.bias` | `[64]` | f32 |
| Reader | `role_projection.weight` | `[3,64]` | f32 |
| Reader | `role_projection.bias` | `[3]` | f32 |
| Core | `embedding.weight` | `[4096,64]` | f32 |
| Core | `query_projection.weight` | `[64,128]` | f32 |
| Core | `key_projection.weight` | `[64,128]` | f32 |
| Core | `value_projection.weight` | `[64,64]` | f32 |
| Core | `output_projection.weight` | `[64,64]` | f32 |
| Core | `null_key` | `[64]` | f32 |
| Core | `null_value` | `[64]` | f32 |
| Core | `output_norm.weight` | `[64]` | f32 |
| Core | `output_norm.bias` | `[64]` | f32 |

Reader: 141,571 scalars. Core: 286,976 unique scalars. The fourteen tensors
contain 428,547 f32 scalars, or 1,714,188 raw bytes. `lm_head.weight` is omitted
from the serialized core and its state CID. The loader requires exactly this
missing key and no unexpected keys, then checks that `core.lm_head.weight is
core.embedding.weight`. The input layer norms have no affine parameters; no
extra biases, query residual, positional embeddings or dropout exist.

The separate source file and tensor-state identities are:

| Asset | Bytes | SHA-256 recorded in the accepted binding |
|---|---:|---|
| Reader safetensors | 566,692 | `912ac1d8a3dfb80a04755557576fbb87d518e78f04163085111fccfd329e5250` |
| Core safetensors | 1,148,672 | `43ffff3c24f8030701e340cab802b985f7c0b7e4e12e270ec1107d141d65b079` |
| Vocabulary JSON | 130,245 | `01d70796333a5c94c87a45d012a04038a9c79da2127792f5acd0132fd0255a82` |
| H4 frames JSON | 81,255 | `ea9ea1de2f666aff24761991e16cb3d7ab21f3b36e38992e04b2376927c18b65` |
| Token frames JSON | 32,189 | `427f20c223886131910ebc3a16dcc4d7898c732b1654a37e483b5494b0b83fc0` |

Reader state CID:
`blake3:7c659422df2e65a0ce24c08738dc9f08dca99775de1702251097a0fc6483404e`.
Core state CID:
`blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59`.
Frame bundle tree CID:
`blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.
The manifest retains all five file CIDs and original source locators from
`docs/r4_retained_assembly_1094_evidence/bindings.json`. These are retained
identities, not a claim that the external files were freshly read here.

**Definition — tensor-state identity.**
`zoology_release/development.py:173-183` iterates tensor names in sorted order.
For each tensor, append canonical UTF-8 JSON of
`{"name":name,"shape":[...],"dtype":"torch.float32"}` with keys sorted,
compact separators, no NaN, and one trailing LF; append its contiguous C-order
raw bytes immediately afterward. BLAKE3 of the resulting concatenation is the
state CID. There is no outer array, tensor-count prefix or additional domain
prefix. Accepted execution is little endian. Exclude `lm_head.weight` from the
core map. File CIDs hash complete safetensors bytes and therefore identify a
different object. These codecs must not be conflated.

## Input, lexical and frame boundary

`policy.json` is the byte-bound `R4TextToClausesV1` policy, SHA-256
`91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0`.
One raw byte buffer becomes `inputs[1,5,13]` and `lengths[1,5]`, both i64:
four facts followed by the question. Padding ID is 57. The caller batches
these unchanged arrays; accepted workers used batches up to 128. Raw input is
at most 4,096 bytes. Grammar recognition accepts four identical fact forms
within a request, from the four declared forms, plus the single question form.
It returns no roles or semantic assignments. The original `_parse_clause`
continues to be evaluator logic outside the entry path.

Reader IDs 0–51 share the core lexical prefix. Reader aliases 52–57 are
`not`, `but`, `,`, `owned`, `by`, `<pad>`; the corresponding core embeddings
and core output names remain the existing `<unused-0052>` through
`<unused-0057>`. IDs 58–4095 retain the core unused-token names. Input rejects
IDs 0, 7, 11 and 57 as lexical words and does not admit the unused names.
Output decoding uses all 4,096 entries of **the core vocabulary**. In
particular, `unknown` is a possible learned output, not an input escape token.

The exact byte lexer, spans, typed refusal precedence and input-hash framing
must follow `adapter.py` and the policy. Lowercase ASCII words and literal
punctuation are recognized without normalization; space, tab, LF and CRLF are
the only separators. Bare CR and non-ASCII bytes are encoding refusals.
Schema/startup refusals precede request parsing; all refusals cause zero model
forwards. Native API representation must preserve this byte-level distinction
without inventing a permissive string conversion.

`zoology_r4_inference/frames.py:138-252` validates the canonical native sidecar
and token map, then reconstructs **f64 directly from the original 120×4×4
u64 bit patterns**. The sidecar helper also exposes f32 matrices, but the
accepted language path deliberately does not upcast those rounded values.
Token leaves have 8,192 entries; multiplication is a 120×120 integer table.
The sidecar and token map must have the same `identity_index`, and token zero
maps to that identity. Use the bound field; this audit does not infer an
integer identity index from observed reachability or assume it is zero.

**Definition — frame fold.** Start each request at identity. For each valid
token, in fact0, fact1, fact2, fact3, query order, update
`current = multiplication[current, token_leaf[token_id]]`. Record that
post-token index; record each clause's last current index as its frame.
There is no reset between clauses, BOS, separator token, label or hidden role
input. Punctuation participates. Padded positions get the identity sentinel
but are never folded, encoded or transported.

## Operator order and numerical seams

These equations are **Definitions of the source operations**, not proof of
floating-point equivalence between Torch and an unimplemented native path.
`Linear(out,in)` stores `[out,in]` weights and applies their transpose to
row vectors. All learned tensors and learned-module outputs are CPU f32.

1. **Reader.** Mask by `position < length`; replace invalid IDs with zero
   before embedding and multiply their embeddings by zero. Shared Conv1d is
   a cross-correlation over the clause, stride 1, dilation 1, groups 1,
   kernel 5 and zero padding 2. In coordinates,
   `h[t,o] = bias[o] + sum(i=0..31,k=0..4) W[o,i,k] x[t+k-2,i]`,
   with out-of-clause values zero. Apply GELU with default
   `approximate="none"`, then the affine `[3,64]` role projection. Mask
   invalid-position scores to negative infinity **after** the projection;
   softmax over the valid token positions separately for owner, object and
   location. No entity mask or position argmax enters model routing.

2. **Token-to-role R4 mixture.** Split each frozen core embedding into
   sixteen consecutive four-lane blocks and convert to f64. For token frame
   `F_t` and clause-end frame `F_c`, encode `F_t^T v`, transport using
   `F_c^T F_t`, and weight all valid transported values with the reader's
   f32 attention coefficients converted to f64. Decode with `F_c`. Cast the
   complete 64-coordinate role vector to f32 before any core layer norm.
   The code processes each clause's equal-length groups in ascending length
   order, never transporting padded zero values. It computes fifteen role
   vectors per row; the query-location vector is computed but unused by the
   binding operation and its fourteen supervised role decisions.

3. **Learned Q/K/V.** Concatenate query owner and object in that order;
   apply non-affine LN over 128 coordinates then query projection. Apply the
   same LN to each fact's owner/object concatenation then key projection.
   Apply non-affine LN over each fact's 64-coordinate location then value
   projection. Every LN uses biased variance, with epsilon `1e-5` inside
   the square root. Append the learned null key/value as slot four after
   the four fact slots. Null uses the atlas identity frame; query uses the
   query-clause end frame; fact sources use their respective clause ends.

4. **Compound R4 mixture.** Convert Q/K/V into f64 sixteen-by-four blocks.
   Encode Q in query frame and K/V in their true source frames. Construct
   source-to-query matrices `F_q^T F_s`, transport all five K/V entries,
   and compute the full dot over sixteen blocks/four lanes in f64. **Cast
   the full dot to f32 before dividing by f32 8.0.** Softmax the five scores
   in f32. Convert those weights to f64 for weighted transported values,
   decode with `F_q`, then cast the completed 64-vector to f32. There is no
   fact equality mask, sparsification, winner selection or query residual.

5. **Output.** Apply the bias-free 64-to-64 output projection, affine
   64-coordinate LN with epsilon `1e-5`, then the tied core embedding
   transpose for all 4,096 logits. Worker answer and diagnostic role
   positions use `argmax` along their final dimensions. Torch specifies
   the first index on exact ties; therefore vocabulary ties select the
   lowest ID, and role ties select the earliest position. Argmax is only
   output/diagnostic handling; soft mixtures remain intact throughout.

All f64 data beyond the frame matrices are these transient block values,
connections, transported values, dot products, weighted sums and decoded
vectors. There are no learned f64 parameters, fitted thresholds or extra
geometry parameters. Exporting only the model safetensors would omit the
lexical policy and both bound native frame files.

The reference delegates convolution, matrix/einsum reductions, GELU, layer
norm and softmax to Torch 2.7.1 on CPU with Apple Accelerate, four intra-op
threads, one inter-op thread and deterministic algorithms enabled. It uses
Python 3.12.14 on little-endian arm64 and complete eval/frozen model state.
The inspected installed Torch sources confirm default Gaussian-CDF GELU,
biased-variance LN, cross-correlation convention and first-index argmax;
they do **not** expose a portable instruction-by-instruction reduction order
or a guarantee for another math library. A native scalar f32/f64 contract
can represent these operations coherently, but its cross-runtime numerical
qualification remains unmeasured. A predeclared tolerance would be an
empirical criterion, not a predicted result or permission to tune after a miss.

## Retained evidence and native implementation boundary

The public execution/replay receipts record the following per-batch seams:
`inputs[B,5,13]` i64, `lengths[B,5]` i64,
`role_attention[B,5,3,13]` f32, `role_vectors[B,5,3,64]` f32,
`binding_attention[B,5]` f32, `logits[B,4096]` f32,
`predictions[B]` i64 and `role_positions[B,5,3]` i64.
Both phases bind the unchanged model states. #1094's exact result concerns
two paths running the **same Python numerical implementation**, then fresh
processes repeating it. The full transient tensor streams were removed
under that run's frozen policy after their phase evidence was persisted.
The committed receipts retain shapes, types, hashes and comparisons; they
cannot supply missing raw tensor values for a future tolerance calculation.
New reference outputs require the separately frozen successor check.

The existing Rust `h4_spin_frame_sidecar.rs` can decode and validate its
canonical frame sidecar; `r4-zoology-frame-export.rs` obtains token leaves
from the native `R4SpinFrameAtlas` and includes three exact prefix witnesses.
Those are useful existing components and provenance. They do not constitute
a Rust learned-role/raw-text implementation. A focused source search found
no native implementations under the learned reader, compound model, raw-text
policy or language-wrapper names. This is a bounded search finding, not
exhaustive proof that no equivalent code could exist elsewhere.

An explicit native container for the fourteen raw f32 tensors, tied-head
relationship, unrounded f64 frame bits, integer maps, two vocabulary views and
exact policy is feasible to specify without fitting or quantization. Its
exporter, strict portable loader, local reader, two-stage inference and
matched behavior check remain unimplemented by this task. No export, native
forward, HTTP/browser wiring, integer/table lowering or final deployed-kernel
qualification follows from this source audit. #1079's weak token control and
#1082's descriptive noncausal findings remain unchanged.

## Review of the proposed native contract

The source-audit reviewer inspected sections 2, 3, 5, 6 and the numerical
comparison rules in section 7 of `r4_native_reference_1086.md`. The prose
preserves the accepted state inventory, tied head, lexical views, raw-text
boundary, continuous frame fold, two transport stages and f32/f64 cast seams.
Review identified the need to say **concatenate owner then object, then
normalize over 128 coordinates**, which the specification now states. Export
must decode safetensors and verify state directly without calling constructors
that would consume RNG before overwriting weights.

The proposed scalar profile intentionally specifies a new numerical reduction
order and a separate empirical threshold. Its f32 GELU formula, stable softmax,
biased-variance layer norm and ordered f64 transport are coherent definitions
of that candidate; no source evidence predicts a tolerance pass. Exact constants,
primitive sequencing, round-to-nearest ties-to-even, preserved subnormals and a
bound math backend must also appear in the machine contract and later release.
For the full binding dot, order the sixteen blocks then the four lanes. The
new B=1 comparison does not inherit #1094's batch-128 timing or byte-equality
result. This review ran zero additional model work; it does not accept a future
export, native binary or execution release.

**Machine-contract review accepted (2026-09-03).** The reviewer cross-checked
contract snapshot SHA-256
`432b737607303e8d31f7b6e7b3afd0938ca26be29c99072bb442157889ec3800`
against this audit's retained identity and source-derived tensor inventory.
All fourteen parameter names, shapes and f32 types match; their byte lengths
sum to 1,714,188. All twenty-one tightly packed component offsets and shape
products reconcile to the declared 2,160,742-byte payload. The five original
asset bindings, both state CIDs and single tied-head alias agree with the
accepted source metadata. All ten declared f32 constant bit patterns match
their stated values, including `epsilon=3727c5ac` and `sqrt_two=3fb504f3`.

The libm 0.2.16 checksum matches baseline `Cargo.lock:930-933`. The profile
explicitly pins its three math calls, `arch` feature, generic
`aarch64-apple-darwin` first target, rounding/subnormal rules, GELU sequence,
ordered reductions, cast-before-division binding score and argmax rule. This
defines one future candidate coherently alongside the unchanged source
equations. The remaining exact compiler, selected architecture path and binary
identities are explicitly required in a separate execution release. Static
JSON/integer/bit-framing checks consumed only specification and audit records;
no export, runtime import, tensor deserialization or numerical inference ran.
There is no operator/schema blocker from this bounded review. Native behavior
and resource feasibility remain `NOT_RUN`.

**Final contract acknowledgement.** The current contract SHA-256 is
`e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`.
The subsequent changes concern explicit trusted-field equality checks and
loader-gate attempt/model-state-load accounting. Model-state loads count the
reader and core separately; they do not count the fourteen wire tensor
components. The reviewed operator, constants, aliases and component clauses
remain unchanged. This operator acceptance therefore
continues to apply to the current contract. The separate boundary review owns
the revised gate accounting. This acknowledgement adds no execution or broader
audit, and does not qualify a native implementation.
