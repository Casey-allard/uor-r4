# Model lifecycle

The full chain from a pinned Hugging Face model to a locally served, evaluated,
CID-addressed bundle. **This is a multi-hour CPU pipeline** — the compile and
score stages dominate, and corpus-scale runs take longer still.

For a zero-setup path that needs none of this, see the
[Quick start](../README.md#quick-start) in the README: the router and dashboard
run with no model at all, and the Gate C harness runs on committed fixtures.

The single-command orchestrator (`./uor-r4-cli`) runs stages 1-3 and then serves,
if you would rather not drive them individually.

**Artifact era.** Compiles emit **TLA7** by default (per-stage i8 centroid
copies, a norm-fold constant, per-stage decode shifts); TLA6 is the fallback and
TLA3/TLA4/TLA5 remain readable through the era-generic parser. `R4_TLESS_TLA7=0`
and `R4_TLESS_TLA6=0` opt a compile back out. Paths below say "TLA" where the era
does not matter.

The local lifecycle has two artifact lanes:

```text
pinned source -> resumable teacher observation -> TLA + TLS1 bundle
                                             |       |
                                             |       +-> local ask/evaluate/import
                                             +-> multiresolution cover
                                                 -> scored R4G1 graph + Gate C report
```

Downloading and compiling are explicit offline operations. Neither `ask` nor
the HTTP server downloads a model or contacts an inference provider.

The top-level `compile` command builds the deployable transformerless bundle
and retains its observation corpus. The graph compiler then consumes those
outputs through `transformerless cover` and `transformerless score`. This
separation makes expensive stages reusable and keeps the deployed runtime
independent of the Hugging Face teacher.

### 1. Download pinned compiler input

```bash
cargo run -- download \
  --repository HuggingFaceTB/SmolLM2-135M-Instruct \
  --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --name smollm2-135m-instruct
```

The default destination is
`.uor-models/sources/smollm2-135m-instruct`. Override it with `--output`:

```bash
cargo run -- download \
  --repository HuggingFaceTB/SmolLM2-135M-Instruct \
  --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --name smollm2-135m-instruct \
  --output /path/to/model-sources/smollm2-135m-instruct
```

The downloader prints the repository and destination immediately, streams the
`hf` process, and emits a heartbeat every two seconds with file count, bytes,
and elapsed time.

#### Source-snapshot manifest (#597)

After a successful download the downloader writes `source_manifest.json`
(schema `uor-r4-source-manifest/1`) into the snapshot directory. It binds the
whole snapshot in one canonical document: repository, immutable revision,
license (SPDX identifier when known; the license *file* is always digested),
compiler version, source-execution mode (`offline-compiler-input`), and every
admitted file's path, byte length, and raw `blake3:<hex>` digest —
`*.safetensors`, `*.json` (including `model.safetensors.index.json`),
`*.model`, `merges.txt`, `LICENSE*`, and `README*`; the manifest excludes
itself. The file list is sorted by path byte order and the serialization is
deterministic, so rebuilding from the same directory reproduces identical
bytes. The manifest's root κ is the canonical-JSON address of those bytes
(`uor_addr::json::address_blake3`), so it uniquely identifies the exact
snapshot; the programmatic surface is `build_source_manifest` /
`write_source_manifest` / `read_source_manifest` / `source_manifest_kappa` in
`src/model.rs`.

The root κ threads into downstream provenance as an opaque string: the cover
stages (`transformerless cover`, `graph-compile`) and the observe driver
accept `--source-manifest-kappa`, recording it as the optional
`source_manifest_kappa` field of the cover/compile report and the observation
manifest; `uor-r4-api`'s `CompileRequest.source_manifest_kappa` forwards it to
the cover stage; and the HTTP server's compile job binds it automatically when
the downloaded snapshot carries a `source_manifest.json`. Legacy inputs
without a manifest compile exactly as before, with the field absent.

**Migration note.** Descriptor κs minted before #597 with
`source_kappa_scope = "model.safetensors"` (e.g. `source_kappa` in
`models/smollm2-135m-instruct.json`) are weight-only identities: they cover
the `model.safetensors` bytes and nothing else. They are NOT relabeled and do
not become snapshot identities; the snapshot-wide identity is only the
`source_manifest.json` root κ described above.

#### Sharded snapshots (#598)

Snapshots whose weights arrive as indexed Safetensors shards
(`model.safetensors.index.json` plus `model-NNNNN-of-NNNNN.safetensors`
files) load through the same teacher adapter as single-file snapshots.
`uor-r4-model-source` resolves and validates the whole snapshot at one
deterministic boundary (`SafetensorsSnapshot::open`) before any model is
constructed: every tensor must resolve to exactly one shard, and missing
shard files or tensors, duplicate or unexpected tensors, shape mismatches
against the `config.json` geometry, byte-length mismatches (tensor spans vs
shape×dtype size, shard file size vs header claim), dtype inconsistencies,
and unsupported dtypes each fail with their own named error. Source tensors
declared BF16, F16, or F32 are widened exactly to f32 (no rounding path);
quantized formats (I8/U8/GPTQ/AWQ-style) are rejected by name, never
silently approximated. A single `model.safetensors` is simply the one-shard
case of the same code path, and its teacher κ (blake3 of the file bytes)
is unchanged. The adapter only checks that every shard file the index
references exists in the snapshot directory; the full per-file digest
cross-check against `source_manifest.json` (#597) remains the root crate's
responsibility.

#### Adapter conformance (#599)

The teacher adapter carries a typed feature declaration
(`uor-r4-model-source::conformance::AdapterFeatures`): exactly which
`config.json` space its executor interprets faithfully — activation (silu),
RMSNorm epsilon (the executor's fixed 1e-5, exact), RoPE mode and theta
range (unscaled only; `rope_scaling` is rejected by name), GQA/MQA head
geometry, projection biases (none), embedding tying (either), scalar
BOS/EOS ids, and a never-interpreted chat-template policy. At oracle
construction the parsed configuration is validated against the declaration
BEFORE any tensor is read or observation generated; anything outside it
fails closed with the focused `SourceIngestKind::UnsupportedConfigFeature`,
so a config the adapter would silently misinterpret can no longer load.

Source-executor parity is pinned by schema-versioned canonical-JSON
fixtures (`uor-r4-adapter-fixture/1`): prompt token ids + byte strings,
bounded per-layer residual captures (declared layer indices only), final
hidden state, logits, top-k, per-check tolerances, and an identity block
(#597 manifest binding read via `source_manifest.json` when present, source
κ, adapter/compiler versions, tokenizer identity). The deterministic runner
(`conformance::run_fixture` / `run_fixture_file`) replays a fixture through
the real executor and returns a three-state canonical report: PASS, FAIL
with per-check numeric deltas, or UNAVAILABLE naming the missing
prerequisite — a missing pinned snapshot or fixture is reported as
UNAVAILABLE evidence, never silently skipped. Replaying the same fixture
twice produces byte-identical reports. The synthetic conformance tests run
everywhere; the real SmolLM2 arm is fixture-gated
(`real_smollm2_fixture_round_trip_passes`, ignored until the pinned 257 MiB
snapshot is downloaded).

### 2. Compile the source

Compile an already downloaded directory:

```bash
cargo run --release -- compile \
  --source .uor-models/sources/smollm2-135m-instruct \
  --output .uor-models/compiled/smollm2-135m-instruct \
  --seconds 300 \
  --target 20000 \
  --sequence-length 128
```

Or let the compiler download an immutable revision through `hf`:

```bash
cargo run --release -- compile \
  --model HuggingFaceTB/SmolLM2-135M-Instruct \
  --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --seconds 300 \
  --target 20000 \
  --sequence-length 128 \
  --r4-attention
```

`--revision` must be a full 40-character commit hash. `--seconds` limits the
teacher-generation work performed by one invocation, while `--target` is the
teacher-token goal. `--r4-attention` selects the experimental teacher
attention variant during generation — the #602 operator
`experimental-r4-source-attention/1`, a 4-wide-chunked dot product with the
same softmax selector as the standard operator (see "Attention operator
identity (#602)" below; omitting this flag runs the standard
`standard-source-attention/1` scaled dot-product operator, the default
everywhere). Hugging Face compilation defaults to
20,000 tokens and 128-token teacher stories. The bounded story length keeps
attention cost and KV memory proportional to the eight-token deployed runtime
window; increase `--target` or `--sequence-length` explicitly for quality
experiments. Repeat the same command to resume an incomplete corpus.

On macOS, offline Hugging Face teacher execution uses Apple Accelerate's
SIMD-optimized CPU BLAS. Linux and Windows use explicit NEON on AArch64 or
runtime-detected AVX2/FMA on x86-64, with a dependency-free scalar fallback.
These compiler accelerators do not add a runtime dependency or change the
allocation-free table-native inference path. Set `TLESS_TEACHER_EXACT=1` to
force the slower, reduction-order-preserving scalar path for diagnostic
comparisons. The pinned legacy proof workflow always uses that exact path.

When compilation completes, the output directory contains:

```text
tless_artifacts.bin       # TLA teacher projection and class tables (TLA7 by default)
tless_store.bin           # TLS1 graded continuation evidence
tokenizer.bin
corpus.meta               # observation-corpus metadata
corpus.records            # deterministic observation records
hamming_calibration.json
hierarchical_codes.json
space_manifest.json
```

Interactive terminals show progress bars. Redirected output receives periodic
`progress:` lines suitable for build logs. Source loading and compilation may
allocate; the allocation-free guarantee applies to the deployed prediction hot
path, which uses fixed and caller-owned buffers.

#### Geometry and projection identity (#600)

The teacher adapter distinguishes two widths: the **source geometry**
(`source_dimension()`, the `hidden_size` declared by `config.json` — 576 for
the pinned SmolLM2-135M) and the **compiled geometry** it presents to the
transformerless compiler (`dim()`, the legacy `D = 288`). The reduction
between them is an explicit, versioned algorithm, not an implementation
detail: `uor-r4-model-source::geometry` names the current one
`bucket-average/1` (output index `i` averages the contiguous source slice
`[floor(i·S/C), floor((i+1)·S/C))`; non-divisible widths spread the
remainder by the floor boundaries so bucket sizes differ by at most one;
each bucket is summed left-to-right in f32 and divided once — the exact
arithmetic the adapter has always used, factored into the free deterministic
function `bucket_average_project`). A versioned registry maps
`(id, version)` to the implementation (`projection_implementation`); an
unknown pair fails closed with the focused
`SourceIngestKind::UnknownGeometryProjection` rather than being guessed.

The typed record — `GeometryProjection { id, version, source_width,
compiled_width, params, implementation_digest }` — threads into provenance
the same way the #597 manifest κ does: the observe drivers record it in the
observation manifest automatically whenever the oracle declares one, the
cover stages (`transformerless cover`, `graph-compile`) accept
`--geometry-projection <json>` and bind it as the optional `geometry` field
of the cover/compile report, and the HTTP server's compile job binds it
opportunistically from the downloaded snapshot's declared `hidden_size`.
The `implementation_digest` is the blake3 of a canonical, byte-stable
serialization of the algorithm's declared parameters plus a stable
algorithm tag — deliberately not a hash of source code text, which would
churn under refactors that leave the arithmetic bit-identical; the unit
tests pin the implementation to the declaration, so a behavioral change
must arrive as a new registry version. The record rides the report/manifest
provenance sidecars; artifact container bytes (TLA/R4G1) are unchanged, so
every previously pinned artifact κ remains valid.

**Migration note.** Reports and observation manifests produced before #600
carry no `geometry` field; absent metadata marks the implicit legacy era.
For teacher inputs produced by the Hugging Face adapter — where
bucket-averaging 576→288 was the sole historical behavior for the pinned
SmolLM2-135M — an absent record may be *interpreted* as `bucket-average/1`
at 576→288. This is an interpretation rule for readers, not a relabeling:
legacy documents are not rewritten, and inputs from the legacy checkpoint
oracle (whose source width is already 288, no projection) carry no such
implication.

#### Versioned tokenizer adapters (#601)

The tokenizer a pipeline segments with is a typed, versioned identity, not
an implementation detail. `uor-r4-core::transformerless::hf_bpe` derives a
`TokenizerAdapter { family, version, tokenizer_cid, policy,
adapter_digest }` record from every parsed `tokenizer.json`
(`HfBpeTokenizer::adapter()`, selected through `TokenizerKind::adapter()`):
family `hf-byte-bpe` version 1 is the post-#242/#253 byte-level BPE, the
`tokenizer_cid` is the blake3 of the raw `tokenizer.json` bytes (the same
CID rule used everywhere today), and the policy block names the applied
normalizer (`none`), the pre-tokenizer steps in order (e.g.
`digits(individual_digits=true)` then
`byte-level(add_prefix_space=false)`), the byte-fallback mechanism
(`byte-level-alphabet` — the GPT-2 byte alphabet makes encoding total), an
added-token count + canonical digest, BOS/EOS insertion (`none`; the
pinned post-processor is null), and the chat-template policy
(`not-interpreted`). The `adapter_digest` is the blake3 of a canonical,
byte-stable serialization of that declared identity — like the #600
geometry digest, deliberately not a hash of source code text; a
behavioral change must arrive as a new registry version, never an
in-place edit.

A versioned registry maps `(family, version)` to the constructor
(`hf_bpe::adapter_constructor`); an unknown pair fails closed with the
focused `SourceIngestKind::UnknownTokenizerAdapter` rather than being
guessed. SentencePiece/Unigram is the recorded follow-up family
(`sentencepiece-unigram`): recognized by name, rejected until a versioned
adapter for it exists, never approximated with the byte-level BPE rule.

The record threads into provenance the same way the #597 manifest κ and
the #600 geometry record do: the observe-text drivers (serial and
batched) record it in the observation manifest automatically whenever the
selected tokenizer declares one (`tokenizer_adapter`, an optional
serde-defaulted field — manifests written through the legacy llama2.c
tokenizer carry no field and stay byte-identical, and every existing
tokenizer CID remains valid). Differential source fixtures
(`crates/uor-r4-core/tests/tokenizer_adapter.rs`) pin the current
verified encode/decode behavior — ASCII, accents, CJK, emoji incl. ZWJ
sequences, byte fallback, added tokens, BOS/EOS handling, digit
boundaries, raw-byte round trips — as the baseline any drift fails
against, and a consumer-agreement test asserts the observation,
evaluation, serving-prompt, and exported-runtime-tokenizer selection
seams resolve the same adapter identity, token ids, and decode bytes.

#### Attention operator identity (#602)

The attention operator the source teacher computes is a typed, versioned
identity, not a boolean. `uor-r4-model-source::attention` defines
`AttentionOperatorSpec { id, version, projections, positional_action,
compatibility_relation, selector_normalization, value_aggregation,
output_projection, runtime_state, tie_breaking,
permitted_operation_class, params }` with the same canonical-bytes +
declared-identity digest discipline as the #600 geometry and #601
tokenizer records (blake3 over a pinned line format of the declared
identity — deliberately not a hash of source code text; a behavioral
change must arrive as a new registry version). A versioned registry maps
`(id, version)` to the spec (`attention::operator_spec`); an unknown
pair fails closed with the focused
`SourceIngestKind::UnknownAttentionOperator` rather than being guessed.

**Truthful operator inventory.** Two operators are registered, and they
are exactly the two branches the teacher's `r4_attention` switch selects
between (`attention::operator_for_r4_switch` is the one boundary mapping
from the legacy boolean to the versioned identity):

- `standard-source-attention/1` (the switch off — the default
  everywhere): dense per-layer f32 `wq`/`wk`/`wv` projections with
  grouped-query key/value sharing (head `h` reads kv head `h / kv_mul`);
  RoPE rotation of q and k before scoring (interleaved vs. split-half
  layout is a source-config property); compatibility relation
  `score(t) = (Σ_{i<H} q[i]·k_t[i]) / sqrt(H)` accumulated as one
  sequential f32 left fold over the full head width `H` (no remainder at
  any width); selector `softmax_with_mode` — subtract the maximum score
  (first maximum on ties, value-identical since only the maximum's value
  enters), `exp` each shifted score (`libm::expf` in D2 canonical mode,
  `f32::exp` otherwise), divide by the sum; value aggregation as a
  position-ascending weighted sum over the full growing-KV-cache prefix;
  dense f32 `wo` output projection. No argmax or selection happens
  inside the operator, so no further tie-break exists.
- `experimental-r4-source-attention/1` (the switch on): identical in
  every respect EXCEPT the compatibility relation, which computes the
  dot product in 4-wide chunks over dimensions `0..4·⌊H/4⌋` (each chunk
  a left-to-right 4-term fold, chunk subtotals then summed).
  **Remainder policy, measured from the code and pinned by unit tests:**
  the trailing `H mod 4` q/k dimensions never enter any score (dropped),
  the scale still divides by `sqrt(H)` over the full head width, and
  value aggregation still uses every head dimension; for `H < 4` no
  chunk exists, every score is 0, and the softmax yields uniform
  weights. The selector is the SAME max-subtracted softmax as the
  standard operator — the historical description of this branch did not
  match its control flow (#515 audit; see the dated correction in
  `docs/deferral_record_2026_08_05.md`), and the registry id names what
  it computes. The branch remains shipped, selectable, default-off, and
  unmeasured; #602 changes no selection and activates nothing.

The reference implementations are free deterministic functions factored
verbatim out of the executor (`standard_head_attention_weights`,
`experimental_r4_head_attention_weights`,
`head_attention_value_aggregate`) — iteration order and arithmetic
unchanged, the same discipline as #600's `bucket_average_project` and
#599's `layer_forward` factoring, with the existing bit-exactness parity
tests (e.g. `forward_batch_matches_serial_forward`) guarding the
executor path and unit tests pinning divisible and non-divisible head
widths.

The record threads into provenance the same way the #597 manifest κ,
the #600 geometry record, and the #601 tokenizer record do: the observe
drivers record it in the observation manifest automatically whenever
the oracle declares one (`attention_operator`, an optional
serde-defaulted field; `TeacherOracle::attention_operator_spec()`
defaults to `None`, and the legacy checkpoint oracle declares none, so
legacy manifest bytes are unchanged); the cover stages
(`transformerless cover`, `graph-compile`) accept
`--attention-operator <json>` and bind it as the optional
`attention_operator` field of the cover/compile report; the typed
compile API derives it from its `r4_attention` option; and the HTTP
server's compile job binds the standard record (its teacher stage never
enables the experimental switch). An absent record marks the implicit
legacy era: documents produced before #602 are not rewritten, and — the
switch having been default-off everywhere — a reader may *interpret* an
absent record on teacher-produced inputs as `standard-source-attention/1`
unless the producing invocation is known to have passed
`--r4-attention`. **Honesty item:** the score/certify report structs in
`uor-r4-graph-certify` carry no attention-operator field — that stage
reads corpus, cover, and artifact bytes with no teacher oracle in reach
(the #597 source κ and #600 geometry records are likewise absent there),
so #602 records the gap instead of inventing plumbing; an
operator-specific experiment should thread the record through the cover
report it already consumes.

**Boundary note.** These specs describe HOST-SIDE source-teacher
computation (f32 dot products, `exp`, division). The deployed inference
operation contract
(`docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`) is a distinct,
unchanged surface: it forbids floating-point arithmetic on the deployed
hot path and explicitly excludes teacher execution. #602 defines no
target (deployed) operator and changes no runtime contract.

#### Teacher trace profiles (#603)

What an observation pass captures — its trace richness, boundedness,
profile identity, absence semantics, and source/provenance dependencies —
is a typed, versioned identity, not an implicit property of the run.
`uor-r4-graph-compiler::trace_profile` defines
`TraceProfile { id, version, top_k, layer_lane, qkv_lane,
attention_support_lane, declared_digest }` (`uor-r4-teacher-trace/1`)
with the same canonical-bytes + declared-identity digest discipline as
the #600 geometry, #601 tokenizer, and #602 attention records (blake3
over a pinned line format of the declared identity, with EXPLICIT
absence markers — an undeclared lane serializes as `<lane>=absent`, so
absence is part of the digest input and distinct from an empty
declaration). A versioned registry maps `(profile, version)` plus the
caller-declared bounds to the record (`trace_profile::profile_spec`);
an unknown pair — and any unbounded declaration (no layer list, a
support cap outside `1..=64`, more than 64 layer indices) — is refused
by name on the sanctioned error surface rather than guessed.

Four profiles are registered, every one an extension of the existing
observation pipeline (no second pipeline exists), each declaring exactly
which lanes it captures and their bounds (layer index list, top-k size,
per-head support cap):

- `minimal/1` — **the default everywhere**: exactly today's surface,
  the v4 88-byte records (bounded token / top-8 / probability rows)
  plus the aligned `.prob` sidecar. A minimal pass writes bytes and a
  manifest byte-identical to a pre-#603 pass: no `trace_profile` field
  is recorded — absence marks the minimal profile, the implicit legacy
  era, exactly like the #597/#600/#601/#602 fields.
- `layer/1` — minimal plus the per-layer residual lane at declared
  layer indices and the final-hidden lane (the post-final-rmsnorm
  hidden state, `TeacherOracle::hidden_state()`). **Measurement record
  (merged #95):** the final-hidden lane was measured NEGATIVE for the
  cover compiler — 2.8% vs 31.7% Gate C top-1 when the hidden state was
  used as the cover observation vector. The lane is preserved here as a
  recorded capture for measurement, not adopted for fitting and not
  deleted; adopting ANY richer profile for fitting requires a separate
  measured issue.
- `attention-support/1` — minimal plus per-head attention support:
  the top-S `(position, weight)` pairs of each head's softmax weights,
  captured ONLY for declared layer indices and within the declared cap
  S (tapped from the #602 factored per-head weight functions through
  the exact executor path).
- `full/1` — minimal plus all richer lanes: per-layer residuals, final
  hidden, current-position q/k/v rows, and attention support.

**Sidecar, not record change.** The v4 88-byte observation record is a
fixed-width era-stable byte format and cannot carry optional per-layer
f32 lanes without a byte-format change, so richer lanes live in a
per-shard side-car file (`shard-NN.bin.trace`, one fixed-width row per
record), following the probability sidecar's existing pattern: the same
deterministic content-addressed partitioning, crash-safe
append/reconcile/resume, per-shard κ registration in the manifest
(`ShardEntry::trace_kappa`), and canonical ascending-shard merge
(`merge_trace_rows`). The primary shard bytes are identical for every
profile. Lane order within a row is fixed (residuals ascending declared
layers, final hidden, q/k/v, attention support), values are
little-endian f32/u32, and the row width — a pure function of (profile,
capture geometry) — is pinned in the manifest (`trace_row_bytes`) at
the first traced write, so a profile or geometry change mid-corpus is
refused. Absence stays absence: an undeclared lane produces no bytes,
an oracle without the bounded capture surface refuses richer profiles
(`SourceUnavailable`, never zero-filled lanes), and an unfilled
attention-support slot (fewer prefix positions than the cap) carries
the explicit `SUPPORT_ABSENT_MARKER` (`0xFFFFFFFF` in both fields),
never a zero-valued entry.

**Bundle identity.** The manifest's identity seam is ONE stable bundle:
`ObservationManifest::identity_bundle_digest()` computes a canonical
blake3 over the five existing identity fields (`input_cid`,
`source_manifest_kappa` #597, `geometry` #600, `tokenizer_adapter`
#601, `attention_operator` #602) plus the #603 `trace_profile`, in a
fixed order with explicit `absent` / `present:<value>` markers per
component — so the digest moves when any component changes, is
independent of the order the fields were recorded in, and an absent
component is never confusable with an empty or zero one.

**Capture plumbing, bounded.** Richer lanes are captured through the
surfaces that already exist: the #599 `forward_capturing` discipline
extended as `TeacherOracle::step_with_trace_capture` (the exact
executor path — a traced step and a plain step produce identical bits,
pinned by unit test), the `hidden_state()` hook for the #95 lane, and
the #602 factored per-head attention functions as the natural tap for
the support lane. Captures copy out only the declared layer indices,
once per step. Determinism is tested by double-run byte comparison:
the same inputs and profile produce byte-identical shard and sidecar
bytes.

**No default change.** The default profile is `minimal` on every path;
richer profiles are strictly opt-in on the generation observe path via
`observe --trace-profile <id[/version]> --trace-layers <csv>
[--trace-support <n>]` (graph-compiler CLI), mirroring how
`--source-manifest-kappa` flows, or programmatically via
`observation::observe_sharded_traced`. The from-text observe drivers
(serial and batched) remain minimal-only: they carry the manifest
seam (`set_trace_profile` exists on the shard writer) but no capture
wiring — richer text-path capture is a recorded follow-up, not
invented plumbing. Existing fixtures, corpora, and manifests are
unchanged.

#### R4RouteAttentionV1 (#604)

The first genuinely R4-native TARGET attention/relation operator:
`r4-route-attention/1`, a versioned reference specification with a
scalar reference implementation, a packed R4G1 lowering, bounded
operation accounting, and an independently replayable witness. It is
DORMANT — registered `open` in `model/ledger.toml` as
`r4-route-attention-dormant` with the pre-declared #604 run contract
(metric: teacher-forced top-1/top-k agreement and bits/token preserving
the runtime operation contract; first verdict: semantic/operation/
witness correctness before any quality interpretation; exit rule: stop
on any runtime-bound violation, witness-replay failure, or
null-indistinguishability; positive hands the operator to the fitting
issue #605 and stays dormant until the activation gate clears; negative
retains the operator and the report). Nothing in the serving path
constructs it, `packed-routing-dormant` is unchanged, no serving
default moved, and no quality claim exists for it.

- **Where it lives.** Reference semantics, witness format
  (`uor-r4-route-attention-witness/1`), and the independent replayer:
  `uor-r4-graph-certify::route_attention`. Packed lowering over
  borrowed bytes and caller-owned bounded state:
  `uor-r4-graph-runtime::route_attention` — a contract-owned module
  covered by the P-4 source scan
  (`uor-r4-core::transformerless`, `p4_contract_owned_graph_runtime_source_scan`).
  Canonical instance wire layout, hard caps, validation, and the op
  census vocabulary: `uor-r4-graph-format::route_attention`. Registry
  record with canonical bytes + declared-identity digest (#600–#603
  discipline): `AttentionOperatorSpec::r4_route_attention_v1()` in
  `uor-r4-model-source::attention`.
- **Semantics (version 1).** Route codes are 288-bit (36-byte) vectors
  — the deployed signature width (`compiler::D = 288`, HEAD
  `signature_bytes`, the ROUT prototype/mask windows), reused rather
  than invented. Per step: the masked XOR+popcount relation
  `d_j = Σ_b popcount((q[b] XOR c_j[b]) AND mask[b])` over every
  declared candidate; bounded top-M selection (`M` declared,
  `1..=min(8, N)`, `N ≤ 64`) by ascending `(distance, index)` with the
  deterministic tie rule "lowest candidate index on equal distance";
  aggregation of the selected candidates' declared ScoreQ contributions
  in selection order with saturating integer adds. No Q/K/V weight is
  reused under the route equation; source-teacher and target routing
  semantics remain separate registry operators, and the legacy
  `r4_attention` switch still selects only between the two #602 source
  operators.
- **Operation accounting.** `RouteOpCensus { adds, xors, popcounts,
  compares, table_reads, bytes_read, candidates_examined }` in the
  `OpKernel` census style; every per-step count is a data-independent
  closed form of `(N, M)`, so the census is verifiable from the
  instance shape alone. Hard caps refuse on the sanctioned surface
  (`NotAProduct` with the observed value and the bound named — R5).
  The packed step is allocation-free in steady state, asserted by
  `crates/uor-r4-core/tests/allocation_census.rs`.
- **Differential + witness evidence.** Reference and packed paths must
  agree bit-for-bit on selections, distances, aggregates, the census,
  and the serialized witness on a pinned deterministic fixture and
  across a shape grid (`crates/uor-r4-graph-certify/tests/route_attention_604.rs`);
  the witness (inputs digest, per-step selected candidates + distances,
  output, census) is verified by an independent replayer that never
  runs the operator; property tests pin mask honoring, top-M bounds,
  tie determinism, ScoreQ saturation, and cap refusal; source-scan
  tests keep both implementations free of float types and of value
  multiply/divide/modulo by construction.
- **Carriage.** Operator instances are separate canonical serialized
  objects (the `RAT1` instance bytes; witnesses serialize via serde),
  loaded by tests/certify only — nothing is emitted into R4G1
  artifacts, so historical artifacts and every existing fixture stay
  byte-identical. If activation later needs in-artifact carriage, the
  optional-section conventions (`SectionId::OPTIONAL_BIT`,
  `EDGE_KIND_OPTIONAL_BIT`) are the designated mechanism.
- **Boundary.** The deployed inference operation contract
  (`docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`) is
  unchanged: this operator's `permitted_operation_class` is the
  deployed integer class (XOR / masked popcount via table / saturating
  add / compare / table read), its packed implementation uses only
  contract-allowed operation classes, and while dormant it is outside
  every contract-bound serving activity.

#### Route-attention fitting + replacement ladder (#605)

The offline harness that fits `R4RouteAttentionV1` instances from
standard-teacher traces, and the pre-registered evaluator that measures
what a fitted selection is worth. Everything is DORMANT
(`route-fit-dormant` in `model/ledger.toml`): no serving path references
it, no artifact byte moves. Three separations are structural, not
aspirational: compilation success is never fit success (kernel-level
runtime checks and fit gates are separate record fields under separate
verdicts), fit success is never model quality (the synthetic arm is a
cheap instrument; the activation gate binds only to the real arm), and
absence is absence (`NOT_RUN` ≠ `UNAVAILABLE` ≠ `FAIL`).

- **Fit method `route-fit/1`**
  (`uor-r4-graph-compiler::route_fit`), a versioned record in the
  #600 discipline: canonical pinned-line bytes + blake3
  declared-identity digest over the parameter DECLARATION, registry
  refusing unknown `(id, version)` by name. Semantics, per
  `(layer, head)`: project each captured query/key head vector to the
  288-bucket route-code width through the registered `bucket-average/1`
  implementation (vectors narrower than 288 are first cyclically tiled
  to the least multiple of their width at or above 288 — a declared
  parameter, since `bucket-average/1` refuses sources narrower than the
  compiled width); binarize at per-bit LOWER-MEDIAN thresholds computed
  over the fit sample in a fixed order (`f32::total_cmp`); pack bits
  LSB-first. Keys become candidate codes (candidate index = position),
  queries become query codes, queries and keys share one threshold
  table. Mask v1 is FULL (recorded, not learned); radii and
  residual/output projection are ABSENT; `top_m = min(8, trace support
  cap)`. The fit manifest carries EIGHT identity fields (source
  snapshot, tokenizer, adapter, trace, geometry, operator, corpus,
  compiler) with typed absence — on the synthetic arm the tokenizer is
  genuinely `None`, never an empty string — plus a provenance label for
  every parameter (v1: route codes + thresholds `compiled`;
  mask/contributions/top_m `declared`; radii, residual projection, and
  source weights `absent`).
- **Fit input boundary.** The fit consumes the PRODUCTION #603 trace
  corpus: the synthetic fixture teacher implements the full
  `TeacherOracle` capture surface and `observe_sharded_traced` writes
  its shards, `.prob` sidecar, and `.trace` sidecar under the
  registered `full/1` profile; the reader comes back through
  `merge_shards` / `merge_probability_metadata` / `merge_trace_rows`.
  No bespoke side channel exists.
- **Pre-registered contract as data**
  (`uor-r4-graph-certify::route_fit_report`,
  `preregistered_route_fit_contract()`), posted to #605 before the run
  and serialized INTO every report: metric (teacher-forced top-1/top-k
  agreement and bits/token per replaced scope; support-overlap Jaccard
  as diagnostic), nulls N1 (seeded-random codes, same shapes) and N2
  (supports deranged by a cyclic one-position shift within each
  sequence), advance gate per synthetic stage (preflight PASS ∧ runtime
  checks PASS ∧ fitted overlap ≥ max(2× best null, 0.5) ∧ top-1 ≥ 0.90
  ∧ replaced bits/token ≤ 1.10× teacher), anti-vacuity (N2 < 0.5×
  fitted at every evaluated scope, else the instrument is VACUOUS and
  the run is invalid regardless of other numbers), replacement
  semantics `support-restrict-renormalize/1`, and the exit rule (stop
  at the FIRST failing stage; later stages `NOT_RUN`).
- **Selection evidence is the deployed kernel's.** Per step, the
  candidate table is the causal prefix of fitted key codes built by
  `build_route_attention_instance`, stepped by the packed
  `route_attention_step` over caller-owned `RouteState`; every step's
  witness is independently replayed (`replay_route_witness`), the
  census is checked against its closed form, the certify-side
  reference runs as a cross-check arm only, and the state's epoch
  discipline is verified per step (the zero-allocation claim itself is
  owned by `crates/uor-r4-core/tests/allocation_census.rs`).
- **Measured result (synthetic cheap instrument; not a model claim).**
  On the deterministic fixture teacher (2 layers, 2 heads, d=32,
  vocab 64, QK-normalized cosine attention, integer-seeded) over a
  1024-token mini-corpus: instrument VALID (N2 0.2353 < 0.5 × fitted
  0.5718 at the reference scope), and all five synthetic stages passed
  the pre-registered gates — fitted support overlap 0.5718–0.5872
  against nulls N1 0.2019–0.2064 / N2 0.2312–0.2537, teacher-forced
  top-1 agreement 0.9902–1.0 (top-8 agreement 1.0), replaced bits/token
  at most 1.0002× the teacher's 4.9709. Instrument-construction record:
  the first fixture iteration (unnormalized dot-product attention)
  measured VACUOUS under the pre-registered anti-vacuity rule (N2
  0.5272 vs fitted 0.7183) because norm-hub keys made supports largely
  query-independent; the fixture teacher was rebuilt with per-head
  QK normalization and the gates/nulls/margins were not touched.
- **Real stages.** `real-teacher` and `real-corpus` are reported
  UNAVAILABLE with their prerequisites named (pinned SmolLM2 snapshot
  absent from the build env; #531 saturation corpus not yet produced —
  compute-bound) — never a vacuous pass, never silently skipped. The
  `route-fit-dormant` activation gate is a positive pre-registered
  ladder result on the pinned real teacher with the #531 corpus,
  witness replay intact, instrument non-vacuous.
- **Where the report lives.** The ladder emits a canonical
  `RouteFitReport` (`uor-r4-route-fit-report/1`, ciborium bytes + κ via
  `route_fit_report_kappa`) carrying the contract, `instrument_valid`,
  per-stage records (scope, fit-manifest κ, #599-typed preflight,
  runtime checks, overlap instrument, embedded Gate C
  [`GateCMetrics`] teacher/replaced parity rows — the existing #307
  surface extended, not duplicated), and the predeclared decision
  record. Tests: `crates/uor-r4-graph-compiler/tests/route_fit_605.rs`
  (registry, manifest, deterministic double-run of the fit) and
  `crates/uor-r4-graph-certify/tests/route_fit_605.rs` (ladder
  determinism, one-head vs nulls under the pre-registered margins,
  broken-fit FAIL, stop-at-first-failure, UNAVAILABLE reasons,
  absence round-trips).

### 3. Compile the holographic graph

The graph compiler turns the retained observation corpus and TLA artifact
into a multiresolution, overlapping semantic graph. First induce and measure
the cover:

```bash
cargo run --release -- transformerless cover \
  --corpus-meta .uor-models/compiled/smollm2-135m-instruct/corpus.meta \
  --corpus-recs .uor-models/compiled/smollm2-135m-instruct/corpus.records \
  --artifacts .uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin \
  --out .uor-models/compiled/smollm2-135m-instruct/graph-cover
```

This writes `cover.r4g1` and `cover_report.json`. The cover report now emits a
versioned objective block (`objective.config.schema`) with separate train and
held-out components for predictive entropy (`H(A|R)`), future-state entropy
proxies (`H(S_future|R)`), teacher-loss proxy, runtime/artifact/bytes/structure
costs, and information-bottleneck proxy terms (`I(Z;X) - βI(Z;Y_future)`),
plus a bounded top-64 between-region distinctiveness term against the global
next-token prior (default weight `0`, preserving the default cover), and
auditable split decisions. Objective versions migrate by appending new fields
under `objective` while keeping Gate C and predictive-sufficiency reports as
separate reproducible artifacts. Then compile semantic
transitions, fixed-point emission residuals, and exact-evidence carryover:

```bash
cargo run --release -- transformerless score \
  --corpus-meta .uor-models/compiled/smollm2-135m-instruct/corpus.meta \
  --corpus-recs .uor-models/compiled/smollm2-135m-instruct/corpus.records \
  --artifacts .uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin \
  --cover .uor-models/compiled/smollm2-135m-instruct/graph-cover/cover.r4g1 \
  --out .uor-models/compiled/smollm2-135m-instruct/graph
```

The result is `graph/score.r4g1`, a stage-validated packed graph containing
regions, refinement/neighbor/forward edges, ScoreQ emission tables, and the
EXCT evidence section. `graph/score_report.json` records artifact and corpus
kappas plus held-out Gate C top-1 agreement, bits/token, and witness replay.

Passing `--cover` reuses the measured cover. It may be omitted to re-induce
the default cover deterministically during scoring. For experiments, `cover`
also accepts `--depths`, `--k0`, `--regions-budget`, and `--memory-budget`.

The public `ask` and `chat` library paths still load the TLA/TLS1 files from
step 2. The native HTTP server auto-loads `graph/score.r4g1` beside
`tless_artifacts.bin` when present (or accepts `--r4g1-artifact`), validates it,
and uses it for the `transformerless` engine before falling back to TLA/TLS1.
When the dashboard is served by that native process, its **Compile / Refresh
R4G1 Graph** button runs the same cover → score pipeline against the bundle's
`corpus.meta` and `corpus.records`, validates the new graph, and hot-swaps it
into the running server. Static WASM deployments cannot run this compiler.
Static deployments still use the geometric WASM fallback because they have no
native filesystem-backed graph loader yet.

The native dashboard also exposes **Download Hugging Face Weights**. Its input
defaults to the pinned `owner/repository@commit` from
`models/smollm2-135m-instruct.json`, but accepts any repository paired with a
full 40-character commit. Downloads go into `.uor-models/sources/`; nothing is
downloaded until the button is pressed. Afterward, run the bundle compiler and
then the R4G1 graph compiler. If the downloaded source is present and the
compiled bundle is not, the native **Compile / Refresh R4G1 Graph** action now
runs the bundle compiler first, then cover and score compilation, as one server
job.

### 4. Ask locally

Compilation produces a directly loadable local bundle. On first use, R⁴
content-addresses the artifact, store, and tokenizer in `.uor-models/objects`:

```bash
cargo run --release -- ask "why is the sky blue?"
```

This direct path verifies container integrity but does not claim that the
compiled approximation has passed an instruction-quality evaluation. The CLI
logs that distinction. Compilation success and answer quality are separate
properties.

### 5. Evaluate instruction quality

Run held-out instruction and grounding evaluation against the compiled bundle
and retain a machine-readable report:

```bash
cargo run --release -- evaluate-report \
  --source .uor-models/sources/smollm2-135m-instruct \
  --compiled .uor-models/compiled/smollm2-135m-instruct \
  --report .uor-models/compiled/smollm2-135m-instruct/instruction-eval.json
```

The report file stores an envelope with the held-out D3 metrics (top-1
accuracy, teacher-argmax agreement, Witten–Bell bits/token vs the teacher
floor), source/artifact/store/tokenizer/corpus CIDs, and
`report_cid_of_report_bytes` for the inner metrics payload. Do not mark an
artifact as passing merely to bypass the chat quality gate.

### 6. Import the evaluated bundle

```bash
cargo run -- import \
  --name my-chat-model \
  --source-model HuggingFaceTB/SmolLM2-135M-Instruct@7e27bd9f95328f0f3b08261d1252705110c806f8 \
  --capability instruction-chat \
  --artifacts .uor-models/compiled/smollm2-135m-instruct/tless_artifacts.bin \
  --store .uor-models/compiled/smollm2-135m-instruct/tless_store.bin \
  --tokenizer .uor-models/compiled/smollm2-135m-instruct/tokenizer.bin \
  --evaluation-report /path/to/instruction-eval.json \
  --instruction-eval-passed \
  --grounded-answer-rate 0.80 \
  --repetition-rate 0.01
```

The model store defaults to `.uor-models`; set `UOR_MODEL_STORE` to relocate
it. Objects are stored once under `objects/blake3/<digest>`. Reads verify both
the declared byte length and UOR CID. The import command prints the manifest
CID.

Continuation-only bundles may be imported for certification and benchmarking,
but `ask` refuses to load them.

### 7. Ask or chat with an imported manifest

One-shot `ask` calls the R⁴ library directly without a server or network hop:

```bash
cargo run --release -- ask \
  --model my-chat-model \
  "why is the sky blue?"
```

Interactive chat retains turn history:

```bash
cargo run --release -- chat --model my-chat-model
```

`--model` is optional. Selection order is `TLESS_MODEL`, the newest JSON
descriptor in `models/`, then `smollm2-135m-instruct`. A descriptor selects a
name; R⁴ first uses an imported manifest and otherwise falls back to a complete
local bundle under `.uor-models/compiled/<name>`.

Library consumers can use the chat example directly:

```rust,no_run
use uor_r4_wasm_router::chat::ChatEngine;

let mut chat = ChatEngine::builder().model("my-chat-model").build()?;
let answer = chat.ask("why is the sky blue?")?;
println!("{}", answer.text);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Chat is an application of transformerless R⁴, not a separate crate or
inference layer.


## Legacy benchmark and certification workflow

The pinned llama2.c TinyStories path remains available for proof reproduction
and same-machine performance comparison.

```bash
cargo run --release -- setup
cargo run --release -- gen 300 150000
# repeat gen until it reports done=1
cargo run --release -- certify
cargo run --release -- compare
cargo run --release -- compare-report
cargo run --release -- scenarios
```

`certify` performs the compile, store, certificate, and census steps
internally. The bare `compile` and `store` subcommands belong to the HF
graph-compiler path — `compile` requires `--model` or `--source`, and `store`
depends on a prior graph compile — and are not part of the legacy chain.

`gen` output is not byte-reproducible across machines or eras: story and
held-out counts can differ slightly from the certified stream (e.g. 754
stories / 30,036 held-out against the certified 757 / 30,192), which bounds
how exactly downstream figures reproduce.

Its default files are:

```text
/tmp/tless_artifacts.bin
/tmp/tless_store.bin
/tmp/ref/tokenizer.bin
```

This is a TinyStories continuation artifact, not an instruction-chat model.
Use `compare-report` for the recorded certificate without loading the source
checkpoint:

```bash
cargo run --release -- compare-report
```

See [COMPARISON.md](transformerless/COMPARISON.md) for the measured quality
and throughput evidence.
