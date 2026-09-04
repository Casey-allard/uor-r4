# R⁴ — Geometric Intelligence on Local Hardware

**Active architectural-alpha track (2026-09-04):** The old artifact-only
pre-alpha target has been met, but it is a mechanical checkpoint rather than a
useful model or product. The canonical
[project track](docs/integration/project-track.md) now proceeds through fixed
recurrent memory, sparse geometric attention, a nonlinear geometric block,
scale/data/instruction, retrieval/tools, product alpha, Rust/table lowering,
and release proof/evidence/QA.

The fixed recurrent, sparse attention, and first nonlinear R4 stages now
execute mechanically under [#973](https://github.com/UOR-Foundation/uor-r4/issues/973).
The path keeps a constant 9,216-byte f32 K/V ledger, reads at most eight
persistent records plus current, and can replace dense SwiGLU execution with a
finite H4-frame-indexed quaternion-cube residual. Its 120 signed frame indices
contain antipodal pairs that select the same odd map. The no-fit cube runs stayed
bounded but diverged immediately from the fitted dense comparator and produced
visibly degraded text. The next stage is a bounded development-data fit;
useful language, architectural alpha, and final serving remain open.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](rust-toolchain.toml)

R⁴ is an open research project building a **transformerless local AI agent**.
Its goal is to replace transformer attention, mixture-of-experts routing, and
dense learned matrix operations in the serving path with deterministic
geometric routing and lookup.

That is a very real engineering goal, not a claim that the goal has already
been reached. The long-term target is frontier-like capability on ordinary
local hardware. The project is testing whether language context, inference,
and reasoning can emerge from routes through a canonical geometric memory. The
target serving engine uses no Ollama, hosted model, or source-model weights.

**Current programme:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
owns the model track under the
[build-first policy](docs/integration/agent-execution-policy.md). The next
implementation is a bounded development-data fit of the assembled sparse plus
quaternion-cube path against its retained dense-SwiGLU comparator.
[#1107](docs/r4_workbench_candidate_1107.md)
remains an unbuilt historical workbench source and #1084 is parked until
product-alpha integration. Start a later task with
[CONTINUE.md](docs/integration/CONTINUE.md). Broad proof, evidence
reconciliation, publication, and QA are release-candidate work.

Measured results, negative findings, and retired paths remain in
[docs/RESEARCH.md](docs/RESEARCH.md). The concise
[current map](docs/integration/current-state.md) is the only planning pointer
that should be consulted before implementation.

## Try the project

With Git and a current Rust toolchain installed:

```bash
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
cargo run --bin r4 -- demo
```

Open <http://127.0.0.1:8000>.

The dashboard lets you interact with the existing geometric router and inspect
its state. It is the quickest way to see the project in motion without
downloading or compiling a language model. A first Rust build may take longer
than five minutes on some machines; later launches reuse it.

To inspect one route from the command line instead:

```bash
cargo run --bin r4 -- route "geometry is the route"
```

To run the current working local 7.15M ordinary-softmax generation prototype:

```bash
r4 generate --prompt "Once upon a time in a quiet village"
```

`generate` defaults to `$UOR_MODEL_STORE/research/issue-1017/export` (or
`.uor-models/research/issue-1017/export` when the variable is unset). It requires
that local export and is a bounded #1017 prototype: provider-free at execution,
but still source-backed, floating-point/matmul/softmax, and below the strict
`<1.50` NLL target. #1019 was closed without another capacity run; it is not a
prerequisite for using or productizing this path.

To expose that same frozen checkpoint through its dedicated native reference
seam, build the Apple Silicon CPU-Accelerate binary and opt in on loopback:

```bash
cargo build --release --offline --features local-inference-accelerate --bin r4
target/release/r4 --host 127.0.0.1 --port 8000 serve \
  --enable-r4-softmax-local \
  --r4-softmax-local-model .uor-models/research/issue-1017/export \
  --r4-softmax-local-workers 4
```

The server never downloads the export. The path must contain the immutable
#1017 `export-manifest.json`, `config.json`, `model.safetensors`,
`tokenizer.json`, and `training-result.json`; invalid or missing state fails
closed, and additional loader files such as a Safetensors shard index are
rejected. The exact-executor default is up to four workers, capped by the
process's available parallelism, and the resulting count is fixed at startup.
Apple Accelerate/BLAS remains CPU-native and owns its internal CPU scheduling;
the worker flag is not a BLAS thread cap. This surface has no CUDA or external
GPU path.

In another terminal, request one raw continuation:

```bash
curl --fail-with-body http://127.0.0.1:8000/uor/v1/r4-softmax-local/generate \
  -H 'content-type: application/json' \
  --data '{"prompt":"Mira found a small brass key beneath the old pear tree.","max_tokens":8}'
```

`max_tokens` defaults to 8 and is capped at 32. The
`uor-r4.r4-softmax-local-http/1` response reports text, generated token IDs,
stop reason, model identity/shape, content-addressed decisions and audits, and
backend/timing provenance without reflecting the prompt, prompt token IDs,
local model path, or filenames. `GET /api/sysinfo` reports
`r4_softmax_local.checkpoint_preflight_ready`. Requests are single-flight and
cannot override the model, backend, or exact-executor workers. This dedicated raw single-turn
endpoint is not `/v1/chat/completions`, the default serving engine, or #962's
source-free multi-turn product. The native-served dashboard may offer it as a
raw-completion choice only after preflight passes; it sends no chat history or
chat template. Static/WASM sessions cannot use it.

The bounded `answer` interface now admits only an exact
`Where is the <subject>?` question and punctuation-terminated source spans:

```bash
r4 answer --source-file facts.txt --question "Where is the copper compass?" \
  --head /path/to/qualified-source-relation-head.json \
  --json-output grounded-answer.json
```

For a source-relative relation head, `answer` pairs each of two to eight exact
punctuation-terminated source sentences with the question, captures the #1017
model's normalized causal R4/Spin state at the final question token, and uses
the explicitly supplied qualified `--head` artifact to choose an exact source
span, typed abstention, or typed conflict. The source is content-addressed and
read again after evaluation to detect change. This is a fail-closed extractive
seam, not semantic-entailment or general-correctness evidence.

The first fixed #954 MPS fine-tune completed in 14 minutes 44 seconds on the
project M1, but its frozen Rust product population failed `1/3`: all three
prompts decoded `ABSTAIN`, so only the unsupported question passed. The command
therefore fails safely, but this checkpoint is not a usable answer model. It is
not rerun or tuned. The subsequent `R4SourceSpanPointerV1` preflight passed
12/12, and Python/Rust parity passed with maximum score delta
`1.234420776e-7` and maximum logit delta `1.428717041e-6`, both inside `0.01`.
The sole 256-step fit nevertheless missed every frozen development gate:
answer `89/128` (`69.53125%`), abstain `114/128` (`89.0625%`), conflict
`117/128` (`91.40625%`), and supported pointer `121/128` (`94.53125%`) versus
`>=95%` each. It stopped `FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`
before producing a final pointer artifact; the three reserved product probes
and browser/HTTP wiring are `NOT_RUN`. The implemented
`R4SourceRelativeRelationHeadV1` C1-SB2 successor then fit 12/12 positive
relations, 20/20 negatives, and 6/6 supported copies on its two fit families,
but transferred only 5/12 positives, 14/20 negatives, and 0/6 copies to its two
sealed families. Same-source matched-pair, query-swap, duplicate-agreement, and
distinct-conflict controls were false, so C1-SB2 stopped before Rust parity,
the sole 512-step full fit, development, and product reveal. It emitted no final
head. Consequently the default `r4 answer` surface is unavailable unless an
explicitly qualified relation-head artifact exists. Do not tune or retry either
revealed head. C1-SB3 `R4AttendedRelationAdapterV1` then used rank-eight LoRA
updates on Q/K/V/O in all six attention layers, a fixed yes/no tied-token
verbalizer, and no trainable classifier head. It moved sealed positive recall
from base `0/76` to trained `73/76`, reached negative specificity `234/239`,
changed all 24 attention tensors, and left all non-attention tensors unchanged.
That is bounded mechanistic representation-transfer evidence, but not a
qualified answer mechanism: fit outcomes were `124/126`; sealed outcomes were
`56/63` (answer `19/21`, abstain `19/21`, conflict `18/21`), and supported
copies were `19/21`. The exact gate stopped Rust parity, the sole full fit,
development, and all four committed but unopened product probes as `NOT_RUN`.
Do not tune or retry this independent-candidate BCE adapter. C1-SB4 then ran
the independently frozen full-source, record-level structured-margin
representation. It recovered `126/126` fit and `63/63` sealed positive groups,
but rejected only `394/478` and `197/239` negatives. Exact records were
`70/126` fit and `35/63` sealed; same-source query relocation was not exact.
Rust parity, checkpoint emission, development, and product remained `NOT_RUN`.
Do not tune or retry C1-SB4. C1-SB5 then fit `56/56` paired records but reached
only `14/28` sealed; row-swap equivariance was bit-exact and mean-query plus
attention-off controls were `0/28`. Its products remained unopened, no
checkpoint/head was emitted, and Rust/development were `NOT_RUN`; retire the
rung without retry. #954's final
source-free terminal remains blocked behind #973, and #955 remains blocked
behind #954. See the
[#954 record](docs/r4_grounded_correctness_954.md) and
[C1-SB0 structured result](docs/r4_grounded_correctness_954_raw.json) plus the
[C1-SB1 pointer result](docs/r4_source_span_pointer_954_raw.json) and
[C1-SB2 relation result](docs/r4_source_relation_head_954_raw.json), the
[corrected C1-SB3 result](docs/r4_attended_relation_adapter_954_raw.json), and
[C1-SB4 result](docs/r4_joint_candidate_margin_954_raw.json), followed by the
[C1-SB5 result](docs/r4_paired_query_binding_954_raw.json).

On Apple Silicon, build the opt-in CPU-BLAS version so local inference uses the
machine's Accelerate framework:

```bash
cargo build --release --offline --features local-inference-accelerate --bin r4
target/release/r4 generate --prompt "Once upon a time"
```

On the project M1, the same four-token #1017 prompt produced token IDs
`[14, 403, 285, 261]` and text `, there was a` under both exact `uor-matmul`
and Accelerate. Output and attention-audit CIDs were identical. Accelerate cut
measured generation from `3.060506042 s` to `0.116236875 s` (`26.33x`) and
end-to-end wall time from `3.41 s` to `0.52 s` (`6.56x`). The complete report
still differs intentionally because backend provenance and timing differ.

To run the qualified source-backed R4/Spin softmax reference generator from an
already-local pinned SmolLM2 snapshot:

```bash
cargo run --release --offline --bin r4 -- r4-softmax-generate \
  --source .uor-models/sources/smollm2-135m-instruct \
  --prompt "Explain in three short sentences why plants need sunlight." \
  --max-tokens 32 \
  --workers 4 \
  --json-output /tmp/r4-softmax-reference.json
```

This is the native `R4SoftmaxReferenceGeneratorV1` evidence surface. It has no
provider or network fallback and remains source-weight-backed, `f32`/matmul,
allocating, and Transformer-compatible. It is not the table-native runtime or
the browser-only WASM dashboard. See the
[generation record](docs/r4_softmax_reference_generation_973.md) and
[attempt-01 aggregate](docs/r4_softmax_reference_generation_attempt_01_result_973.json).

To expose the identical source-backed policy through its native research
bridge, opt in explicitly on a loopback address:

```bash
cargo run --release --offline --bin r4 -- \
  --host 127.0.0.1 --port 8000 serve \
  --enable-r4-softmax-reference \
  --r4-softmax-source .uor-models/sources/smollm2-135m-instruct \
  --r4-softmax-workers 8
```

The dashboard reveals the reference option only when the native server reports
the source ready. The dedicated API is
`POST /uor/v1/r4-softmax-reference/generate`; it is not `/api/chat` and does
not replace the default engine. The frozen eight-token sunlight request matched
the CLI's token sequence, decoded `Plants need sunlight to undergo
photosynthesis, a`, decision CID, persistent-state CID, all-30-layer audits,
and zero-future-read audit. The endpoint is disabled by default, loopback-only,
native CPU only, single-flight, and capped at 32 generated tokens. Static/WASM
builds reject it. Dashboard wiring/readiness and isolation checks passed, but
browser interaction/E2E remains `NOT_RUN`. See the
[native bridge result](docs/r4_softmax_reference_http_bridge_973.md).

To run the one fixed canonical-ingestion witness:

```bash
cargo run --bin r4 -- lexical-ingestion-witness
```

To compile and evaluate the established #989 source-free lexical table path:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --json
```

The corpus directory must also contain its pinned `manifest.json`. The command
uses only the D3 construction partition for its vocabulary and integer
unigram/bigram/trigram counts, evaluates held-out next-unit prediction, writes
the deterministic packed artifact, and emits the exact decoded continuation.
It is a statistical lexical baseline command, not an attention, semantic,
correctness, chat, or release surface.

Once the artifact exists, generate from it directly without the corpus:

```bash
cargo run --bin r4 -- source-free-generate \
  --artifact /path/to/source-free-table.bin \
  --prompt "The United States" \
  --continuation-cap 16
```

Add `--json` to include the artifact CID, prompt, continuation, completed text,
stop reason, and source-closure counters. This invocation loads only the packed
table artifact. Its output is deterministic and prompt-conditioned within the
table's lexical statistics; it does not establish semantic understanding or
general language capability.

To run the one frozen #953 comparison against that unchanged table baseline:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --geometric-intervention \
  --geometry-overlay-out /path/to/multiscale-count-radius-r4.bin \
  --json
```

`--geometric-intervention` enables only the frozen
`MultiscaleCountRadiusR4V1` tie-breaking overlay. Both arms retain the table's
first nonempty row, maximum-count tie set, lexical codec, decoder, and shared
declared-work ledger. The report compares held-out choices and both fixed-prompt
continuations; `--geometry-overlay-out` writes the deterministic overlay bound
to the base table artifact. The overlay is a bounded causal geometry experiment. Even a
positive comparison does not establish attention, semantics, correctness,
reasoning, chat quality, performance superiority, formal closure, or release
readiness.

To reproduce the bounded A1R associative ordered-summary decision:

```bash
cargo run --bin r4 -- associative-ordered-summary-a1r-probe
```

To reproduce the corrected A1P paired-H4-derived exact R4-heatmap
identifiability decision:

```bash
cargo run --bin r4 -- candidate-relative-identifiability-a1p-probe
```

To run the #953 decoded loop against a canonical route artifact:

```bash
cargo run --bin r4 -- bounded-geometric-generate \
  --artifact /path/to/canonical-route.json \
  --prompt "active agile athletes run" \
  --continuation-cap 2 --json
```

This research command loads no provider or source weights. It currently accepts
only a canonical artifact whose embedded construction/global input can fully
reconstruct the parent codec registry; subset-observation artifacts fail closed.
Plain output labels both the appendable continuation and typed stop reason;
`--json` emits the full deterministic witness. Trailing prompt whitespace is
also rejected fail closed so the lexical-boundary contract cannot silently
rewrite the prompt. The command is bounded to that reconstructed vocabulary and
the local #969 path; it is not `ask`, `chat`, or a correctness-qualified answer
surface.

The A1R command uses only the frozen construction/evaluation fixture and exact
finite tables. Its frozen report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
The associative state passed the declared scope, independent-global, fold,
incremental, and support invariants. The full arm produced distinct `ll`/`rr`
relative states on all 6 queries, but shortest Cayley distance mapped both to
energy 2 and tied every query. The terminal verdict is `RETAIN_STATE_ONLY`: it
does not generate text or establish full attention.

The A1P command preserves those six queries as regression-only evidence,
prepares construction and sealed-validation geometry/support without labels,
and derives S4 parity from each exact history and the frozen role order before
joining the separate label ledgers. Its paired contract computes
`X=C(H,c)`, `Y=C(P_c,c)`, and `D=X*Y^-1` in the signed `(1,i)` R4 chart. The
exact endpoint rule is `sin=±1, cos=0 -> 1` with chirality retained and
`sin=0, cos=±1 -> 0` with cosine polarity retained; `q0=q1=0` is typed-null
abstention, not a threshold shortcut. `q2` and `q3` remain in the full `D`
witness but are not scorer-key fields.

The target-free structural census covers 120×120 = 14,400 ordered pairs, 120
relative rows, 45 exact heatmap classes, and 480 typed-null pairs. Across 36
fixture decisions, 14 classes were exercised; construction coverage was 12/12
and pure, construction classes covered 10/12 validation decisions, the
no-class-splitting oracle ceiling was 10/12, strict construction transfer was
0/6, and eight heatmap classes were incompatible. The hard gate therefore
stops before scalar search; every downstream selection, control, and placement
row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. Its terminal literal is
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Contract, universe, and report
kappas are
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
This negative is bounded to the paired-H4-derived heatmap readout. Fixed-zeta
phases, ordered n-lets, exact `phi` radial transport, and the typed
`sqrt(2) <-> 2i <-> [0,2]` adapters remain structural under
`STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`; they are not
scorer inputs. It does not establish attention or generation, and #969 becomes
the next stage only after protected #970 merge. #969 has since delivered one
bounded causal path selector. #953 has driven it through real decoded-loop
plumbing and tiered admission on the frozen preflight, but the natural agreement
run made the same full-path choice for both prompts and did not qualify a
natural grammar result.

The ingestion witness maps two turns of text through the pinned lexical codec,
prime/spin route state, canonical hierarchy manifest, strict reload, and exact
lexical reconstruction. It also exercises the declared fail-closed unknown-unit
path. It loads no model and establishes reversible state plumbing only—not
attention, inference, correctness, or reasoning.

The additive serving envelope is
`uor-r4.canonical-lexical-route-manifest/1`; it transitively embeds the frozen
`uor-r4.prime-route-spin-manifest/2` bytes. Its codec identity is
`uor-r4.unicode-lexical-runs/1`: UTF-8 identity normalization, caller-declared
sentence/paragraph/turn boundaries, canonical surface-byte vocabulary order,
and rejection of unknown units before mutation. The parent keeps the complete
codec route-address registry in stable lexical-unit order; the unchanged child
manifest contains only addresses witnessed by its causal sentences. The fixed
input ceiling is 8 turns, 32 paragraphs, 31 sentences, 128 units per sentence,
512 total units, and a 64-unit content-addressed global snapshot.

Downstream code consumes `CanonicalRouteArtifact::decode_canonical`,
`attention_consumer_trace`, `attention_consumer_trace_for_cursor`,
`attention_consumer_trace_with_ordered_h4`,
`incremental_update_trace`, `incremental_cursor`,
`lookup_shared_class_trace`, `scope_ceilings`, and `reconstruct_input`. The
attention handoff is ordered current, previous, last-two, sentence, paragraph,
conversation, then bounded global; the cursor resolver returns those same seven
slots and marks not-yet-established boundaries absent. S0 serializes state and
numeric geometry only: every candidate row ceiling is zero and marked
`NOT_IMPLEMENTED_S0_STATE_ONLY`. #952 established candidate/value reachability
but found its reusable summaries order-erasing. #967 landed the exact ordered
state repair but retained it as state only after the candidate tie. #970's
corrected paired-H4-derived exact R4-heatmap gate stopped at bounded readout
identifiability without searching another scalar. #969 then qualified one local
causal path selector, and #953 implemented the first bounded decoded
library/CLI plumbing. Its relabelled smoke terminated
`REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` then
recovered exact `{still}` then `{run,runs}` primary support while consulting
and truthfully tracing adjacent-spin rows, which remained non-admitting until
the primary tier was empty. The one permitted four-arm run produced `still run`
for both full-path prompts and `still runs`
for both state-disabled prompts, with deterministic replay. The terminal
remains `REVISE_I1_GENERATOR_IN_PLACE`. The first frozen local same-object,
order-sensitive candidate-placement preflight then failed before generation or
replay: real placement selected 0/2 intended candidates while its same-artifact cyclic
placement control selected 2/2. #983's later independent construction-return
classes then transferred to 0/6 held-out decisions. #986's later local
qualification stopped before geometry because neither its exact corpus/codec
population nor a complete lexical Cl(0,6)/SpiralCore frame was available.
#953's historical H4/placement fixtures remain untouched. The later B0 reset
accepted a separate fixed-point R4 table-tie intervention and closed #953 at
its positive terminal. #973 Gate 0 has since retained one bounded prior-prefix
copy mechanism. Its frozen paragraph and conversation slices retained one
exact-descriptor/entity-binding path selector apiece at their respective
   scopes. The first bounded-global exact-spin relation failed target-free; its
   independently frozen V2 noncommuting repair then passed the bounded decoded
   contract. The first natural corpus placement later failed in PR #997, and
   the first bounded gated-delta core trailed plain delta on its sealed smoke.
   #973 now owns the accepted direct transported Q/K/V/O softmax reference.
   Intrinsic/readout alternatives, resonance-based softmax replacement,
   full-model recurrent lowering, and exact deployment are parked. The
   provider-free-at-execution, source-backed `R4SoftmaxReferenceGeneratorV1`
   (`HELM-D-R4`) generation gate and opt-in, loopback-only dedicated native
   HTTP endpoint now pass. Dashboard wiring/readiness and static/WASM-isolation
   checks pass; the hosted Pages surface is static, currently reports WASM
   offline, and has no working chat backend/artifact lowering. The Q16 suffix
   trace student completed with bounded distillation but looping output; its
   recurrent `R4SoftmaxTraceStateStudentV1` successor then failed to produce a
   material or selection-bearing effect. The subsequent construction-only
   observability audit completed at `INSUFFICIENT_SUPPORT_COVERAGE` and cannot
   attribute a boundary. #1014 then directly trained the frozen R4/Spin
   causal-softmax model: its `2.677393`-nat attention-off penalty and two-arm
   Rust parity establish attention, but enabled NLL `2.127407` and
   subject/scene retention `3/5` fail its quality DoD. Close that exact campaign
   without tuning. #1017's separate exposure continuation then passed retention
   `5/5` and all mechanical gates but failed only sealed NLL at
   `1.5727521962806827`. #1019 now freezes that 12-layer parameter-capacity
   contract. Population, smoke, and random-export parity passed, but MPS
   admission stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour
   offline implementation; full training through replay remains `NOT_RUN`.
   The fused-AdamW/deferred-logging fast path was slower (`4.485223` versus
   signed `3.491307 s/step`); #1019 closed without a full run. #954's cosine
   pointer stopped before final artifact or product reveal; its implemented
   C1-SB2 relation successor then failed matched-transfer preflight before Rust
   parity/full fit/development/product and emitted no final head. C1-SB3's
   rank-eight all-layer Q/K/V/O adapter showed bounded transfer (`0/76` to
   `73/76` sealed positive recall) but failed exact outcomes at fit `124/126`
   and sealed `56/63`; parity/full fit/development/product are `NOT_RUN`.
   C1-SB4's full-source structured-margin successor then failed at `70/126` fit
   and `35/63` sealed exact records and stopped before Rust/checkpoint/product;
   do not retry it. C1-SB5 later fit `56/56` pairs but reached `14/28` sealed and
   retired before checkpoint/head/Rust/development, with products unopened.
   CUDA and external GPU execution are out of scope. #954's final source-free terminal
   stays blocked behind #973, and #955 remains blocked behind #954.
See the [append-only #953 record](docs/local_geometric_generation_953.md).
See the [accepted table-tie record](docs/source_free_table_geometric_intervention_953.md).
See the [#973 Gate 0 record](docs/prior_sentence_count_radius_attention_973.md).
See the [#973 paragraph record](docs/paragraph_entity_spin_path_attention_973.md).
See the [#973 conversation record](docs/conversation_entity_spin_path_attention_973.md).
See the [append-only #973 bounded-global record](docs/bounded_global_exact_spin_attention_973.md).
See the [#986 evidence record](docs/corpus_signed_transport_attention_986.md)
for the exact feasibility boundary and deliberately unrun stages.
Stored H4/Hopf/zeta/icosian and related route fields remain
structural state, diagnostics, or controls unless the owning stage qualifies a
specific term.

These commands exercise the no-model research substrate. `demo` does not start
the historical artifact-discovery server, and `route` does not claim to answer
the prompt; it exposes how the current geometry represents it.

The browser-only WASM visualization is published at
[uor-foundation.github.io/uor-r4](https://uor-foundation.github.io/uor-r4/),
but the hosted Pages deployment currently reports WASM offline and cannot run
chat: it has neither the native reference backend nor a lowered compiled student
artifact. With `just` and `wasm-pack` installed, `just wasm-dashboard` builds the
local visualization surface without model weights. Neither surface is evidence
for attention, coherent generation, inference, or reasoning.

## What R⁴ is trying to build

The central hypothesis is simple:

> **The geometry is the route, and the data is the location.**

Text is reversibly assigned to canonical geometric addresses. As a sequence
unfolds, its route carries local and accumulated context. A bounded geometric
query evaluates possible next locations, chooses an admitted least-cost route,
and decodes that location back to text.

```text
text
  → reversible lexical address
  → prime / semiprime route
  → spin, phase, torsion, and radial state
  → current + sentence + conversation + global context
  → bounded next-route selection
  → text
```

The working design brings together:

- primes and semiprimes as addressable atoms and route experts;
- spherical harmonics as the working description of related spin states;
- fixed zeta-zero channels with changing phase and torsion;
- S³/R⁴ transport, Hopf projection, and golden-ratio radial shells;
- a paired-H4/E8 bridge for coupled geometric state; and
- recursive context at route, sentence, paragraph, conversation, and global
  scopes.

Kappa provides canonical identity and serialization. It is not itself the
tokenizer, semantic distance, attention mechanism, or language model. A pinned
lexical codec supplies reversible text boundaries; the intelligence must come
from the geometry.

## What exists now

The current foundation can represent and rebuild prime-route state, preserve
transported trajectory and overlapping context summaries, and perform bounded
deterministic candidate lookup.

It has **not** yet demonstrated:

- prompt-to-answer source-free chat;
- recursive geometric attention that generalizes beyond recall;
- a qualified natural grammatical generation loop;
- correctness and calibrated abstention;
- multi-step reasoning; or
- frontier-class capability or an energy advantage.

Earlier compiler, graph, proof, conformance, and teacher-derived systems remain
in the repository as research evidence and reusable components. They are not
the current product path and are not prerequisites for trying the dashboard.

## Historical roadmap lineage

**Historical roadmap narrative; superseded for sequencing on 2026-09-03.**
The retained lineage below explains the earlier decisions. Its then-active
research step was #1082. The [active project track](docs/integration/project-track.md)
and [current map](docs/integration/current-state.md) own sequencing now.

The programme is deliberately sequential so that infrastructure and testing do
not become substitutes for working intelligence:

1. **Retain the established source-free table baseline (#989)** — 22.261404%
   held-out top-1 versus 5.413561% unigram on 446,342 known targets, exact
   bounded decoding, and byte-identical replay. Preserve its artifact and claim
   boundary as a statistical lexical reference.
2. **Retain the accepted R4 tie intervention (#953)** — 23.211797% held-out
   top-1, +4,242 correct choices over the unchanged table, a distinct bounded
   continuation, matched support and declared-work ledger, and byte-identical
   replay.
3. **Compile from the qualified `R4SoftmaxReferenceGeneratorV1` oracle (#973)** — retain the literal causal Q/K/V/O
   scaffold and V4's positive construction-scale connection-gauge covariance,
   but preserve its terminal held-out negative: H4, alternative, and plain were
   each 13/24 and the destructive controls did not separate. Pin and reproduce
   HELM-D as the bounded architectural reference, then preserve a frozen
   ordinary full decoder's learned Q/K/V,
   ordinary stable softmax, value aggregation, and output projection while
   splitting heads into R4 blocks, binding exact cumulative Spin/H4 frames,
   transporting every causal K/V pair into the query frame, and mapping the
   aggregate back before `W_o`. Require numerical/behavioral parity first on
   frozen real next-token loss, top-1, and decoded output against equal-budget
   plain controls. The first coefficient-only `acosh^2`/centroid intrinsic arm
   stopped unavailable at construction covariance and was diagnostically worse
   than donor and flat R4. The next source-faithful learned-manifold qualifier
   completed validly but failed functional retention and matched parity:
   Lorentz NLL `7.710618`, Euclidean `4.483154`, donor `3.667626`, while all
   geometry-destroying controls were worse than coherent Lorentz. The 8/8
   contract's score/readout attempt stopped at its two-document preflight and
   rejected tangent readout: pooled
   normalized audit-MSE ratio `1.0643688804269025`. Accept ordinary
   dot-product/stable-softmax causal attention in coherent R4/Spin frames as
   the current baseline. Park intrinsic score/readout, resonance,
   softmax-replacement, recurrence, and exact lowering. The smallest
   provider-free-at-execution, source-backed native CPU
   `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) path now passes its native CLI
   gate while retaining the credited HELM
   attention seam and using UOR's pinned
   SmolLM2 `HuggingFaceLlamaOracle` for embeddings, RoPE, residual/RMSNorm,
   MLP, final normalization, and the language-model head. The frozen gate
   recorded 4/5 quality in both passes, 5/5 exact replay after deleting timing,
   exact all-layer audits, zero future reads, and source-donor reproduction.
   Its explicit opt-in, loopback-only dedicated native HTTP endpoint now passes
   the frozen eight-token canary with exact CLI token, text, decision/state CID,
   all-layer-audit, and causal-read parity, without changing the default engine.
   Dashboard wiring/readiness and static/WASM-isolation checks pass; browser
   interaction/E2E is `NOT_RUN`.
   That trace/compiler rung produced `R4SoftmaxTraceStudentV1`, and the next
   recurrent `R4SoftmaxTraceStateStudentV1` rung completed with exact causal
   execution but no material control separation, no changed decision, and the
   same loop. Its #1012 full-trace/signed-reduction/state/readout audit then
   completed at `INSUFFICIENT_SUPPORT_COVERAGE`; it cannot localize signal loss
   and will not be expanded or repeated. #1014 then established load-bearing
   ordinary causal attention with a `2.677393`-nat attention-off penalty and
   exact Rust parity, but failed its complete quality gate at enabled NLL
   `2.127407` and prompt retention `3/5`. Close that campaign without rerun or
   tuning. #1017 then completed the one frozen exposure continuation: NLL
   `1.5727521962806827` failed the strict `<1.50` gate, while retention, parity,
   causal audits, and replay passed. #1019 now freezes an optional 12-layer,
   13,130,784-parameter increase over the same mechanism. Its population, MPS
   overfit smoke, and random-export/all-12-layer Rust preflight parity passed, but MPS
   admission stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour
   offline implementation; the full train/final-qualification/reveal/
   generation/replay path remains `NOT_RUN`, with no further 7.15M exposure or
   LR tuning. The fused-AdamW/deferred-logging fast path was slower (`4.485223`
   versus signed `3.491307 s/step`); #1019 closed without a full run.
   Separately, #973 qualified `R4RetainedLanguagePathV1`, then rejected its
   paired-H4, direct/layerwise readout, and learned-associative capacity seams.
   The independently frozen V5 predictive write/binding successor completed
   `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`: fresh-language and integrity
   passed, but terminal capacity, geometry attribution, and delta-overwrite
   attribution did not. Retire that law and `STOP_WITHOUT_GENERATION`; this
   does not revoke the accepted ordinary-softmax or retained-attention scopes.
   #954's
   cosine pointer stopped before final artifact or product reveal. Its
   implemented C1-SB2 relation successor then failed matched-transfer preflight
   before Rust parity/full fit/development/product and emitted no final head.
   C1-SB3's rank-eight all-layer Q/K/V/O adapter changed only attention tensors
   and transferred most sealed relations, but failed exact fit/sealed outcomes
   (`124/126`, `56/63`); all later stages are `NOT_RUN`. C1-SB4's full-source
   record-margin successor then failed at `70/126` fit and `35/63` sealed exact
   records and stopped before Rust/checkpoint/product; do not retry it. C1-SB5
   later fit `56/56` pairs but reached `14/28` sealed and retired before
   checkpoint/head/Rust/development, with products unopened. CUDA and
   external GPU execution are out of scope.
   Do not resume resonance substitutes. Product development continues through
   `r4 generate`, but no production-readiness or release claim follows yet. This intermediate
   reference is transformer-compatible, `f32`/multiply/alloc and source-weight
   backed—not table-native, multiply-free, or transformerless. No tag, release,
   static web, or browser-WASM claim follows from the result.
4. **Establish correctness** — relevance, contradiction handling, and honest
   abstention.
5. **Establish reasoning** — bounded multi-step route composition.
6. **Connect and ship the accepted engine** — chat integration, measured
   optimization, and only then release QA.

The CLI and WASM dashboard remain usable research surfaces throughout this
sequence so each new mechanism can become visible before the final engine is
complete.

The active dependency chain is tracked in
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820). #989 established
the frozen table reference, #953 established one matched R4 tie intervention
over it, and #973 retained one bounded prior-prefix copy mechanism plus bounded
exact-descriptor/entity-binding path selectors at paragraph and conversation
scope. Its first bounded-global relation remains closed-negative history; the
independently frozen V2 repair passed its bounded contract; and PR #997 rejected
the first natural componentwise placement. A bounded gated-delta core is
structurally implemented but negative against plain delta on its sealed smoke.
Direct-attention V2 is non-promotable; its equal-manifold-budget V3 rejects the
tested mixed-gauge H4 projection/connection/optimizer combination against a
working plain arm. Its `10/12` alternative-connection score is diagnostic only
because that arm was swapped at inference time rather than trained separately.
Connection/gauge Phase I is positive within #973, but its protected Phase-III
held-out reveal is negative: every main arm scored 13/24 and the destructive
controls failed to separate. `HELM-D-R4` source-pinned full-decoder softmax
parity in transported R4/Spin frames now passes. The first intrinsic
distance/centroid V1 attempt is unavailable before D3. The following
source-faithful HELM-D learned-manifold construction qualifier completed as a
valid functional-retention/parity negative despite clear destructive-control
separation. Its 8/8-contract score-by-readout attempt stopped at the
two-document preflight and returned
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`. Ordinary dot-product/stable-
softmax causal attention in coherent R4/Spin frames is therefore the accepted
current baseline; intrinsic/readout alternatives, resonance-based softmax
replacement, full-model recurrent lowering, and exact deployment are parked. The provider-free-at-execution,
source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation gate and
its explicit opt-in, loopback-only dedicated native HTTP endpoint now pass,
with no default-engine change. Dashboard wiring/readiness and
static/WASM-isolation checks pass, but the hosted Pages deployment is static,
currently reports WASM offline, and lacks a working chat backend/artifact
lowering. The source-free Q16 suffix trace student is complete and boundedly
positive but loops; `R4SoftmaxTraceStateStudentV1` also completed and failed
its material, decision, and cycle gates. The bounded construction-only
observability audit then completed at `INSUFFICIENT_SUPPORT_COVERAGE`; no
boundary attribution follows. #1014 subsequently established load-bearing
ordinary causal attention in the trained R4/Spin path through a `2.677393`-nat
attention-off penalty and exact Rust parity. Its full quality DoD is negative:
enabled NLL `2.127407` exceeded `1.50`, and subject/scene retention was `3/5`
versus `4/5`. #1017's separately frozen continuation improved those measurements
to NLL `1.5727521962806827` and retention `5/5`, but its full DoD remains
negative solely on the strict NLL ceiling. #1019's optional frozen 12-layer,
13,130,784-parameter campaign uses the same attention/runtime path.
Its population, MPS overfit smoke, and random-export/all-12-layer Rust preflight parity
passed, but the signed MPS probe stopped `UNAVAILABLE_HARDWARE_BUDGET` for the
frozen eight-hour offline implementation; the full campaign remains `NOT_RUN`.
UOR's deployed architecture/runtime remains CPU-native. Apple Accelerate/BLAS
and MPS are local offline accelerators only. The fused-AdamW/deferred-logging
fast path was slower (`4.485223` versus signed `3.491307 s/step`); #1019 closed
without a full run. #973 subsequently qualified its retained language path,
then closed the paired-H4, direct/layerwise readout, learned-associative, and
predictive block-delta promotion rungs. V5 terminal
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` retires only that write/binding law
at `STOP_WITHOUT_GENERATION`; fresh-language and integrity passed, while
capacity, geometry attribution, and delta-overwrite attribution did not. #954's
cosine pointer stopped before final artifact or
product reveal. Its implemented C1-SB2 relation successor then failed
matched-transfer preflight before Rust parity/full fit/development/product and
emitted no final head. C1-SB3's rank-eight all-layer Q/K/V/O adapter produced
bounded representation transfer but failed exact fit/sealed outcomes at
`124/126` and `56/63`; parity/full fit/development and the unopened product
population are `NOT_RUN`. C1-SB4's independently frozen full-source
structured-margin successor then failed at `70/126` fit and `35/63` sealed
exact records, with perfect positive-group recall but only `82.43%` negative
specificity. Rust/checkpoint/development/product are `NOT_RUN`; do not retry
it. C1-SB5 later fit `56/56` pairs but reached `14/28` sealed and retired before
checkpoint/head/Rust/development, with products unopened.
CUDA and external GPU execution are out of
scope. #954's final source-free terminal remains blocked behind #973, and #955
remains blocked behind #954. The exact contract is
[ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md).

## Find your way around

- `src/` — the `r4` executable, local server, chat shell, and WASM surface.
- `crates/uor-r4-core` — current geometric route/manifest foundation plus
  preserved runtime research.
- `crates/uor-r4-router` — geometric router, memory, and dashboard backend.
- `crates/uor-r4-graph-*` — preserved graph-format/compiler/runtime research.
- `docs/` — current programme, mathematical decisions, evidence, and archive.

Start with the [documentation guide](docs/README.md). The
[R4 Intelligence Completion Plan](docs/r4_intelligence_completion_plan.md) is
the post-v0.1 sequencing authority and readable mirror of programme root #820;
the [Geometric Intelligence Programme](docs/geometric_intelligence_programme.md)
defines its architecture and claim boundaries. Historical records remain
available through the documentation guide without dominating the front door.

## Contributing

This is an obscure and ambitious research problem, and useful contributions
are welcome. The most valuable work advances the first unblocked roadmap stage
and produces an observable user-facing capability. Expensive experiments and
broad QA stay dormant unless a current decision truly requires them.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change.

## License

MIT — see [LICENSE](LICENSE). © 2026 UOR Foundation.
