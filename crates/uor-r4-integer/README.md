# UOR-R4 Geometric Language Model: integer serving

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

The [packed coefficient candidate](../../docs/integration/packed-integer-preparation-2026-09-27.md)
keeps signed4 arrays in shared `Arc<[u8]>` storage after loading and consumes
packed rows directly. Signed16 arrays are shared separately.
`IntegerModel::coefficient_storage()` reports unique coefficient payload bytes,
excluding metadata, loading temporaries and session state. Artifact formats and
supported dimensions are unchanged. The [completed comparison](../../docs/integration/packed-integer-comparison-2026-09-27.md)
preserves every declared stable field across eight paired workloads on the two
retained width-256 artifacts. Logical retained coefficient payload falls from
6,702,340 to 847,876 bytes; observed peak child RSS is 13.04–16.83% lower.
Timing changes are small and mixed in this single pass. Dense access and the
artifacts' language limitations remain; no general speed or energy gain is established.

The separate [width-576 dialogue development profile](../../docs/integration/dialogue-child-native-observation-2026-09-27.md)
now accepts a verified complete-prefix child through its own conversion schema.
Use the training example `dialogue-child-integer-bridge` to construct its packed
model and `pack-dialogue576` to package it. This keeps the selected child's
actual arrays and local clocks separate from its historical R1d ancestor.
The observer's `--child` mode compares this child's saved reference and actual
FF/QQ/integer replies. The nearest-hard child loses useful relationships and
remains diagnostic; it is not an accepted bundle or useful chat qualification.
The linked result supplies complete CLI arguments, artifact identity and costs.

## Build and package

From the repository root:

```sh
cargo build --release --offline -p uor-r4-integer --bin uor-r4-integer
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
or recovery. The [focused validation](../../docs/evidence/chat-stream-error-validation-2026-09-27.json)
uses synthetic library and CLI-helper checks plus a release build, with no
learned-model execution, spawned CLI exit-status test or chat-quality claim.

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
