# UOR-R4 Geometric Language Model: integer serving

> **Selected API, 13 October — D29:** Cargo selects `src/native_geometry.rs`, exposing fixed H4 geometry, checked integer primitives and report/config metadata. The historical model/stack/sampling executors and generation binaries described below remain source evidence and are excluded; `historical-runtime` fails compilation. The composed native CUDA graph at `1f1547bfd` passes 31 finite controls on RTX4090, pending protected delivery. No compliant D11 language-serving model was qualified by this facade or those controls at that 13 October boundary; training holds then in force are historical. Current runs use the lab's D67 pre-run review, with D41's donor pass the named owner-at-dispatch exception. [Boundary record](../../archive/labs/d29-vendor-boundary-2026-10-13/README.md).

## Status through 31 Oct 2026

This crate serves the saved discrete M1 pair and the separate float-trained FBS trunk as D11 integer artifacts. Pre-alpha: 1 of 8 milestones is complete, and no useful conversation is established. The model's two paths and their state are in [the pipeline map](../../docs/model/pipeline-map.md); the plan is [PLAN.md](../../PLAN.md).

- **Discrete M1 pair.** `uor-r4.lut-discrete-trunk/1` reproduces the reference on all 14,287 DEV rows and passes B4. It scores 7,002 / 7,001 on DEV, 7,215 / 7,228 on Fresh V2 and 7,850 / 7,872 on p-f, closing all six bars at two seeds. Apple M1 decode is 10,876 / 10,566 tok/s under the bar-f protocol ([M1 close](../../docs/labs/m1-close-saved-model/README.md), [Fresh V2](../../docs/labs/m1-fresh-panel-transfer/README.md), [bar-f](../../docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).
- **D80 and D81 (28 Oct).** Served integer code may use the hardware integer multiply and divide. Floats, transcendentals and dense matmul stay banned in serving.
- **Separate FBS `lut-stack/5`.** It keeps the 120-root rotations exact, putting the export gap inside its bar at DEV scope ([claude-182](../../docs/loop/tasks/d11-exporter-float-trunk/claude-182/REPORT.md)). It serves at 4,217–4,320 tok/s, then 4,491/4,504 after the integer square root ([claude-186](../../docs/loop/tasks/fbs-serving-vector-kernel/claude-186/REPORT.md)), on the M1 laptop. `/6` carries additional Hamming-read hooks; the joined session remains open.
- **Limit.** p-f has 62.5% verbatim TRAIN reference replies, so M1 is not useful-chat qualification. Claude-217's teacher-forced p-s reply openings are only 17.4%, 13.7% and 18.6% at three seeds, against about 80% on later tokens.

## Historical implementation and evidence

This crate serves the retained learned recurrent model with full causal context,
integer model arithmetic, and integer token selection. It supports the quaternion
model and the matched Householder-pair control, with the retained dot-product
read or the hyperbolic (Lorentz) read, at state width 128 or 256 (read width 64,
context up to 256). Serving bundles keep the retained width-256, context-256
shape. The weights retain the model's existing
limitations: general prose remains weak, and useful general reasoning and
frontier capability are not established.

A Lorentz model is served under its own packed contract
(`config::packed_numerical_contract`), which differs from the retained one only
in its read declaration. Its two learned scalars are signed 16-bit codes. At load,
`exp(read.lorentz_log_beta)` is computed once with integer arithmetic. For each
key, `z - 1` of the lifted query and key comes from their Q8 codes as a
difference of exact integers, with one square root and one rounded division
(see [`src/lorentz.rs`](src/lorentz.rs)). The distance `arcosh(z)` is read from
a sealed Q24 table (`arcosh.json`/`arcosh.bin`), which the training tool now
exports next to the retained tables; the retained table files are unchanged.
A Lorentz model refuses a table root without it.

The crate depends on the shared byte-level tokenizer, Serde/JSON, and hashing
libraries. It has no dependency on Candle, the training crate, the core crate,
model-source, a transformer implementation, or an external model provider.
Training evaluation re-exports the same integer computation used here.

The [packed coefficient candidate](../../archive/integration/packed-integer-preparation-2026-09-27.md)
keeps signed4 arrays in shared `Arc<[u8]>` storage after loading and consumes
packed rows directly. Signed16 arrays are shared separately.
`IntegerModel::coefficient_storage()` reports unique coefficient payload bytes,
excluding metadata, loading temporaries and session state. Artifact formats and
supported dimensions are unchanged. The [completed comparison](../../archive/integration/packed-integer-comparison-2026-09-27.md)
preserves every declared stable field across eight paired workloads on the two
retained width-256 artifacts. Logical retained coefficient payload falls from
6,702,340 to 847,876 bytes; observed peak child RSS is 13.04–16.83% lower.
Timing changes are small and mixed in this single pass. Dense access and the
artifacts' language limitations remain; no general speed or energy gain is established.

The separate [width-576 dialogue development profile](../../archive/integration/dialogue-child-native-observation-2026-09-27.md)
now accepts a verified complete-prefix child through its own conversion schema.
Use the training example `dialogue-child-integer-bridge` to construct its packed
model and `pack-dialogue576` to package it. This keeps the selected child's
actual arrays and local clocks separate from its historical R1d ancestor.
The observer's `--child` mode compares this child's saved reference and actual
FF/QQ/integer replies. The nearest-hard child loses useful relationships and
remains diagnostic; it is not an accepted bundle or useful chat qualification.
The linked result supplies complete CLI arguments, artifact identity and costs.

## Build and package

**Current build clarification:** The [manifest](Cargo.toml) requires `serving-wasm` for this binary; the implementation and qualification evidence above remain historical.

From the repository root:

```sh
cargo build --release --offline -p uor-r4-integer --features serving-wasm --bin uor-r4-integer
target/release/uor-r4-integer pack PACKED TABLES TOKENIZER_JSON NEW_BUNDLE
```

`PACKED` is a sealed accepted packed-model directory; `TABLES` is its sealed
integer table directory; `TOKENIZER_JSON` is the original tokenizer file. The
tokenizer's SHA-256 must match the parent's evaluator provenance. Packaging
copies the existing parameter codes and tables; it performs no fitting or table
construction. `NEW_BUNDLE` must not exist.

For a model packed by a training tool rather than an accepted campaign (for
example the `joint-integer-parity` example of `uor-r4-training`), use
`pack-development PACKED TABLES TOKENIZER_JSON NEW_BUNDLE`. It produces the
same sealed layout without evaluator provenance: the tokenizer is the caller's,
and `bundle.json` records the bundle as a development bundle with no accepted
parent. A Lorentz model's table root must carry the arcosh table.

The resulting sealed bundle binds the model, table hashes, tokenizer bytes and
identity, full context policy, and parent provenance. Loading uses the files
inside the bundle; recorded source paths do not cause source lookups. Vocabulary
size must be 4096, `<|bos|>` must encode as token 0, and `<|eos|>` as token 1.
The tokenizer applies its existing byte-level BPE rules without inserting BOS or
EOS. The session inserts one BOS explicitly.

## Generate from a request array

```sh
target/release/uor-r4-integer generate BUNDLE requests.json NEW_REPORT
```

`requests.json` is an array of 1–256 independent requests. Each starts a fresh
model session. The complete request shape is:

```json
[
  {
    "prompt": "The red key is in the box. Where is the red key?",
    "max_new_tokens": 24,
    "selection": {"kind": "greedy"},
    "read_mode": "enabled",
    "first_sentence": true
  },
  {
    "prompt": "Once there was a little bird",
    "max_new_tokens": 48,
    "selection": {"kind": "categorical", "top_k": 40, "seed": 17},
    "read_mode": "enabled",
    "first_sentence": false
  }
]
```

`prompt` and `max_new_tokens` are required. Omitted fields default to greedy
selection, reading enabled, and `first_sentence: false`. Unknown fields are
rejected. `read_mode: "no_read"` is an explicit diagnostic intervention.

Greedy ties choose the lowest token ID. Categorical selection samples the Q48
masses at temperature 1 using the declared xorshift64 generator. `top_k: 0` or
`top_k >= 4096` includes all tokens; `top_k: 1` is greedy and consumes no random
value. This is a new policy, not seeded parity with the historical floating
temperature sampler. No temperature parameter is accepted.

The full requested budget must satisfy:

```text
1 BOS + encoded prompt length + max_new_tokens <= 256
```

Empty prompts, zero continuation budgets, and oversized requests fail. The
runtime does not truncate, evict, reduce attention access, or reset implicitly.
Every prediction exposes every previous occurrence, up to 255 positions. A
64-coordinate read vector does not impose a 64-token access limit.

Generation stops at EOS, the requested maximum, or three repeats of a token cycle
of period 1–4. With `first_sentence: true`, any decoded ASCII period also stops
generation; this is a byte-level stop rule, not a linguistic sentence parser.

`NEW_REPORT` must not exist. Successful output is sealed and contains
`generations.jsonl` and `summary.json`, with token IDs, rendered text, stop reason,
Q48 distribution hashes, causal-slot counts, identities, and timings. A failed
attempt may leave partial output; retain it and choose a new directory for a
retry. `raw_decoded` is a lossy UTF-8 rendering including emitted EOS content;
token IDs preserve the exact output and `utf8_decodable` reports whether the
response bytes were valid UTF-8. `response_text` omits EOS and trims whitespace.

## Retain a session in Rust

```rust
use std::path::Path;
use uor_r4_integer::{bundle::Bundle, generation::Selection, ReadMode, Result};

fn continue_session(bundle_path: &Path) -> Result<()> {
    let bundle = Bundle::load(bundle_path)?;
    let mut session = bundle.text_session(ReadMode::Enabled)?;
    session.append("Once there was a little bird")?;
    let first = session.generate(
        16,
        Selection::Categorical { top_k: 40, seed: 17 },
        false,
    )?;
    let second = session.generate(
        16,
        Selection::Categorical {
            top_k: 40,
            seed: first.sampler_state_after,
        },
        false,
    )?;
    println!("{}{}", first.raw_decoded, second.raw_decoded);
    Ok(())
}
```

`TextSession` retains recurrent state and exact token/occurrence history. Its last
emitted token is pending until the next input or prediction, and counts toward
`len()` and the 256-token limit. `append` accepts finalized text chunks: each
chunk is tokenized independently, so arbitrary byte-stream chunking can change
BPE token boundaries. It is not an incremental UTF-8/BPE stream decoder.

The random seed is explicit for every `generate` call. Pass
`sampler_state_after` as the next categorical seed to continue its random stream;
reusing the original seed restarts that stream. Stops apply per call. Resuming
after a returned EOS is an explicit caller decision. Session state is in memory;
the generated report is not a serialized session checkpoint.

The separate `session::ChatSession`/`ChatTokenStream` API used by `uor-chat` has
its own session lifecycle. After draining a stream, callers must inspect
`error()` and `stop_reason()`; iterator exhaustion alone is not success. If the
model write that commits a sampled EOS, turn-end or custom stop, or an inserted
cap/cycle turn-end, fails, the stream retains the error and reports `ModelError`
while flushing already buffered text. `uor-chat` exits with status 1 before
success telemetry on a terminal error and reports the actual stop on success.
Existing stream-construction errors still print and return to the prompt, so
exit status zero alone does not establish that all earlier commands succeeded.
A failed model step may have changed state; this reporting provides no rollback
or recovery. The [focused validation](../../archive/evidence/chat-stream-error-validation-2026-09-27.json)
uses synthetic library and CLI-helper checks plus a release build, with no
learned-model execution, spawned CLI exit-status test or chat-quality claim.


`uor-chat` serves three different containers, and they hold different models. `--bundle <PATH>` loads
the sealed recurrent bundle described above. `--stack <model.lut> --tokenizer <TOKENIZER.json>
[--content-readout <SIDECAR>] [--prompt-reply <SIDECAR>] [--protocol 1|2] [--threads N] [--say TEXT] [--max-tokens N]` instead serves a `UORLUT01` geometric
stack artifact directly, through `stack::IntegerStackModel` under the literal-role protocol
(`stack::StackChat`; default version 2, the pinned corpus encoding; version 1 remains selectable for
old artifacts): the engine's integers and the float stack's stop rules, greedy `stack_argmax`
over `next_token_scores()` with pointer mixtures included, exact token history, and an over-context
turn refused rather than truncated. The stack path is **greedy only** — a nonzero `--temperature` is
refused because this engine has no integer categorical sampler. `--say` prints a JSON record.
`--content-readout` attaches the positive modulo-256 nearest-example table from a separately hashed
`uor-r4.stack-content-readout/1` sidecar. The sidecar must bind the exact artifact SHA-256; it
overrides only `serve_step`'s native/point fallback when a training row is within its learned Hamming
radius. Raw `step` and `step_native` remain unchanged. The one-shot, REPL, prompt-ID and OpenAI
loopback paths all use the attached model. Session snapshots written with a readout use the
`UR4SESS2` codec and bind its canonical hash; restore refuses a missing or different readout before
changing state. Legacy `UR4SESS1` bytes remain unchanged when no readout is attached.
`--prompt-reply` attaches an `uor-r4.stack-prompt-reply-readout/1` sidecar on
the loopback serving path. The sidecar binds the exact artifact and tokenizer
hashes and returns one complete training reply by exact prompt-token-set
Jaccard ordering. A hit emits the stored reply followed by EOS; a miss or a
response that exceeds `max_tokens`/context falls back to the normal greedy
path. The sidecar never changes `UORLUT01`, raw `step`, `step_native` or the
content-readout path.
`--memory <VALUE>` adds one `System: Memory: <VALUE>.` turn before the next assistant reply, using the
same literal-role encoder as training. In the REPL, `/memory <VALUE>` queues the same one-turn line,
`/memory` displays it and `/memory clear` removes it. The value is explicit in this unit; statement
extraction is opt-in through `--memory-sieve` or `/memory auto`. The deterministic sieve searches
only previous user turns, ranks clauses by shared normalized content words and recency, and extracts
a bounded name, time, location, age or predicate value. `/memory manual` disables it and `/reset`
clears both the fact log and any queued explicit value. Explicit memory overrides the sieve for one
turn. Generated assistant text is never treated as fact data.

### Native OpenAI loopback (3 November 2026)

The same non-discrete `--stack` artifact can be served directly, without a CLI subprocess or the retired router:

```sh
uor-chat --stack model.lut --tokenizer tokenizer.json \
  --serve 127.0.0.1:8000 --studio-root web/studio
```

The loopback service implements `GET /healthz`, `GET /v1/models`, `GET /v1/models/uor-chat`, `POST /v1/chat/completions`, `POST /v1/completions`, `POST /v1/responses`, and `GET /v1/capabilities`. Text messages and text parts work with multi-turn history, `max_tokens`, `max_completion_tokens` (and `max_output_tokens` for Responses), `stream`, `stream_options.include_usage`, `temperature: 0`, and one choice. JSON and SSE return exact prompt/completion/total token usage plus additive `uor` metrics: integer elapsed nanoseconds, first-token nanoseconds, completion IDs, and tokens-per-second in thousandths (null if the measured elapsed value is zero). When `--content-readout` is active, metadata and generation records also carry `content_readout` and `content_readout_sha256`, the sidecar path and raw sidecar SHA-256. EOS and short cycles map to `stop`; the token cap maps to `length`. Unsupported models, roles, fields, content parts, and OpenAI families receive explicit errors rather than placeholder success; `/v1/embeddings` and image input return 501 in this unit.

Only `127.0.0.1` binds. Host and Origin are checked, bearer/`X-UOR-Studio-Token` authentication is required for `/v1` (except CORS preflight, which is validated by Host/Origin alone), headers are capped at 32 KiB and bodies at 1 MiB by default, and one generation runs at a time (a concurrent request receives 429 before any SSE bytes). Context overflow returns 413 before a stream starts; a second in-flight generation is refused. One completion ID/created timestamp and Responses event/item identity stay stable across a stream. The per-launch token is never printed or persisted. When `--studio-root` is present, the same-origin HTML receives an in-memory `window.__UOR_NATIVE_BOOTSTRAP__` for the next Studio unit; traversal and symlink paths are refused. Headless serving without that root requires `--token-stdin`, which reads 1-256 token bytes once without logging them; launching without either token channel fails immediately. `--origin` optionally adds one exact cross-origin value; cross-origin access is otherwise disabled. This is a local protocol/serving path, not production publication, M4/M7 qualification, or a useful-conversation claim.

`--discrete <model.lut> --prompt-ids <CSV> [--max-tokens N]` serves a `uor-r4.lut-discrete/1`
artifact directly through `discrete_d11::IntegerDiscreteModel`. Each step appends the evaluator's
integer top-1 prediction; an initial prompt must fit the declared context, which then slides by one
token as generation proceeds. This raw-token path needs no tokenizer, is greedy only, and prints a
`uor-r4.uor-chat-discrete-parity/1` JSON record. Text chat, REPL and `UR4SESS1` discrete sessions are
not supported. No conversion exists between the three containers, and none can be written without
training a model to another container's parameter inventory.
## Arithmetic and cost boundaries

Model values use the retained signed parameter codes, fixed-point state, integer
normalization, bound tables, and exactly normalized Q48 output masses. Both
greedy and categorical token selection use integer arithmetic. Offline floating
table construction remains in the training crate.

This numerical claim does not cover legacy JSON parsing, tokenization, hashing,
allocation, shape/address calculations, or host clock/reporting operations. It
does not claim that every instruction in the executable is free of multiplication
or floating operations. The runtime still reads dense parameters, allocates,
and has a finite context. Packaging alone establishes no speed or energy benefit.

CLI summary wall time includes bundle loading and request generation through
JSONL output, but excludes the final summary write and sealing. Model step time
is nested within that total and must not be added to it. Consult the project
current-state and experiment records for measured results and their scope.
