# UOR-R4 Geometric Language Model: integer serving

This crate serves the retained learned recurrent model with full causal context,
integer model arithmetic, and integer token selection. It supports the quaternion
model and the matched Householder-pair control. The weights retain the model's
existing limitations: general prose remains weak, and useful general reasoning
and frontier capability are not established.

The crate depends on the shared byte-level tokenizer, Serde/JSON, and hashing
libraries. It has no dependency on Candle, the training crate, the core crate,
model-source, a transformer implementation, or an external model provider.
Training evaluation re-exports the same integer computation used here.

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
