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
teacher-token goal. `--r4-attention` enables the experimental 4D softmax-free
Spin(4) attention geometry during teacher generation (omitting this flag runs
standard scaled dot-product attention). Hugging Face compilation defaults to
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
