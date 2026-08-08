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
