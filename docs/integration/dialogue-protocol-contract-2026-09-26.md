# Shared literal-role dialogue protocol

Status: additive Rust integration seam; existing training and serving adapters
have not adopted it. No model fit, new language result or change to a retained
sealed bundle is part of this delivery.

The fourth lab's source review found incompatible dialogue serialization between
the active learning and chatbot branches. The dialogue branch
`codex/canonical-address-routing-20260925` at `50ac2527` defines corpus v1 in
`crates/uor-r4-core/src/bin/prepare-chat-corpus.rs:15–43,208–321`: literal
`System: `, `User: ` and `Assistant: ` markers, independently encoded separator,
marker and content, with EOS after assistant content. Its `dialogue.rs:757–774`
panel builder renders those literal markers into one string. The chatbot branch
`codex/geometric-chatbot` at `474e9866` plus the inspected dirty
`crates/uor-r4-integer/src/session.rs:93–110,631–641` uses dedicated role tokens,
falling back to BOS for system/assistant, UNK for user, and EOS for turn end.
These source snapshots are distinct active work, not merged model qualification.

For a user message `Hi`, the training-compatible generation prefix is BOS,
`encode("User: ")`, `encode("Hi")`, `encode("\n")`,
`encode("Assistant: ")`. The fallback chatbot prefix is UNK, `encode("Hi")`,
EOS, BOS. Even rendering the same visible literal-marker text into one `encode`
call can differ from training: BPE may merge a marker's trailing space with the
first content token. Matching visible strings is insufficient.

## Contract and API

[`uor_r4_tokenizer::dialogue`](../../crates/uor-r4-tokenizer/src/dialogue.rs)
provides the serializable `DialogueProtocol`. Schema
`uor-r4.literal-role-dialogue/1` binds the format below, the raw tokenizer's
canonical CID, and its resolved BOS/EOS/UNK IDs. `identity()` hashes the canonical
serde representation of that contract. `literal_roles_v1()` constructs it;
`from_json()` parses and validates it; `bind()` validates an already deserialized
contract and returns an encoder. Validation returns focused errors for unknown
schema, tokenizer mismatch, missing atomic specials and incorrect special IDs.
No role resolves to BOS or UNK as a substitute for its literal marker.

The bound encoder provides:

- `encode_document(messages)`: the corpus-compatible token IDs, response mask,
  emitted/skipped counts and count of specials appearing in encoded text.
- `encode_assistant_prefix(messages)`: the same history segments, retaining
  prior assistant EOS tokens, followed by the separator and literal assistant
  marker. It omits a non-assistant document-terminal EOS before that marker.

Each message supplies raw role and content strings; callers map missing fields
to empty strings. Content uses CRLF/CR-to-LF conversion and trimming at the ends;
interior whitespace stays intact. Blank content is skipped before role parsing.
Unknown roles are skipped and counted. There is no added turn-alternation rule.
An all-skipped document yields BOS and zero emitted turns, leaving the existing
corpus row-admission policy with its caller.

Every completed assistant turn appends EOS, including an assistant turn inside
a longer conversation. The final non-assistant turn instead gets a document
terminator with mask zero. Only assistant content and its terminator get mask
one. Literal BOS/EOS/UNK text keeps the original tokenizer's atomic behavior and
is counted; this module does not introduce escaping or reinterpret those tokens
as proof of a trusted role. These choices reproduce corpus v1, not a general
message-security or role-isolation policy.

```rust
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

fn prompt_tokens(tokenizer: &uor_r4_tokenizer::ByteBpeTokenizer)
    -> uor_r4_tokenizer::dialogue::Result<Vec<u32>>
{
    let protocol = DialogueProtocol::literal_roles_v1(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let prompt = encoder.encode_assistant_prefix(&[
        Message { role: "user", content: "Hi" },
    ]);
    // Feed these IDs directly, preserving training segment boundaries.
    Ok(prompt.tokens)
}
```

## Adoption and validation scope

The corpus owner should consume the document encoder and retain its data/drop
receipts. The panel and chatbot owners should consume the prefix encoder through
an explicit token-ID ingestion boundary. Decoding these IDs and retokenizing
their text would undo the boundary guarantee. A future artifact manifest must
bind this protocol before claiming training/serving compatibility. The old
continuation bundles remain unchanged; absence of a protocol means its dialogue
semantics are undeclared, not that this version can be silently inferred.

The focused tests cover independent byte-level reference IDs/masks with UTF-8,
CRLF, multiple turns and skipped messages; a BPE merge that crosses a marker/
content boundary; assistant-prefix agreement with the training document prefix;
literal control tags; and schema/tokenizer/EOS validation failures. They do not
assert broad language, memory, geometry or energy capability. Actual model
quality still requires loading the candidate through the adopted adapter and
reviewing its outputs and applicable controls.

The coordinated offline tokenizer check passed all **7 tests** (4 new protocol
checks and 3 existing tokenizer checks); compilation took 13.15 seconds. The
command was `cargo test -p uor-r4-tokenizer --offline` with two build jobs and
the existing shared target directory. No model was loaded or trained.

A separate Rust witness loaded the accepted quaternion bundle's actual tokenizer
from `/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json`.
Its CID is
`blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`;
the derived protocol identity is
`blake3:0099a613c8fcffc78210ed7b7387841917472d976f307a33526c8424ecf5d327`.
For user content `Hi`, the observed tokenizer results were:

```text
Training-compatible segments:
[0,55,2728,28,223,1094,201,35,560,652,714,28,223]
Concatenated panel text (with session BOS):
[0,55,2728,28,310,75,201,35,560,652,714,28,223]
Role fallback constructed from the inspected chatbot source:
[2,1094,1,0]
```

`Hello` and a whitespace/UTF-8 example also differ under both alternatives.
These are executed tokenizer comparisons; the fallback array is constructed
from inspected source, not a newly executed chatbot session or model-quality
measurement. Witness source and output are retained locally under
`/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-tokenizer-witness.{rs,txt}`.
The API is ready for explicit adapter adoption; this result does not claim that
either existing branch has been integrated or that its language quality is fixed.
