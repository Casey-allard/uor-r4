# Shared literal-role dialogue protocol

Status: the shared encoder now has an explicit adapter in the retained integer
text-session path. The separate corpus builder, dialogue panel and partitioned
chatbot still need coordinated adoption. No model fit, new language result or
change to a retained sealed bundle is part of this delivery.

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

### Retained integer session adapter

`Bundle::dialogue_session(protocol, messages, read_mode)` binds the protocol,
encodes its segmented assistant prefix and feeds exact IDs into the retained
full-context integer model. The initial BOS is consumed once. Completed
assistant EOS tokens and literal special-token text inside messages are
preserved. `TextSession::append_tokens` accepts an exact continuation without
inserting or re-encoding any tokens. It validates every ID and the complete
remaining capacity, including a pending generated token, before changing
session state. Model execution errors after validation are propagated; this
does not promise transaction rollback for arbitrary arithmetic failures.

`Bundle::generate_dialogue(protocol, messages, max_new_tokens, selection,
read_mode, first_sentence)` also validates the whole input-plus-output budget
before the first model step. It returns `DialogueGeneration`: the explicit
protocol and its identity alongside the existing generation record, including
exact consumed prompt IDs, selected IDs, probability hashes and decoded output.
The text `prompt` field is empty because it cannot safely represent the original
BPE segment boundaries. Dialogue-created sessions expose their protocol binding
and terminate at its EOS. Optional dedicated turn-end tokens receive no new
delimiter meaning under the literal-role protocol.

Ordinary text sessions retain an actual optional `<|turn_end|>` delimiter, when
present. Absence is now `None` in both token selection and response slicing.
Previously both sites substituted ID 6; the retained tokenizer decodes that
ordinary ID as `$`, so the fallback could stop or truncate dollar-sign text.
This correction changes that accidental behavior without changing model weights,
sampling, context access or the partitioned chatbot's memory architecture.

The explicit protocol is an evaluation binding for an old bundle, not evidence
that the bundle was trained for dialogue. The retained width-256 Dot bundles
use the same tokenizer identity as the chat-v0 corpus, but the current width-576
dialogue fit has a separate loading/export problem. Claude's scratch width-128
packet uses a different tokenizer without these specials. Neither can be made
compatible by substituting the retained tokenizer because vocabulary sizes match.

### Corpus and panel adoption

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
That establishment witness preceded the retained-session adapter above. It did
not execute either model path or establish repaired language quality. The later
adapter's executed scope is recorded separately; the partitioned chatbot and
dialogue panel remain independent consumers requiring explicit adoption.

### Executed retained-bundle witness

Source `79c7c9e0d9cb712af0361cf1dd9d21cfdd8cd9d0` passes all six focused generation
checks: four new session/protocol checks plus retained budget and cycle checks.
The source-bound `dialogue-token-witness` example then loaded the unchanged
accepted quaternion bundle and generated at most eight tokens from the exact
13-ID `Hi` prefix above under greedy and seeded top-40 selection. For both
policies the adapter, direct token appending and independent `IntegerModel::step`
plus `Sampler` agree on every selected ID, full probability-vector hash, causal
slot count, NoRead mass and sampler state. The direct replay follows the
adapter's emitted count; the separate unit checks validate stopping semantics.

The actual responses were `10 and the1 of the1` and `“Is there the“ver?” J`.
Both reached the eight-token cap. This is a successful protocol/prediction
transport witness and an unqualified chat model; correct ingestion does not
create conversational training. Context capacity was 256, while this short
witness consumed the stated prompt and output, not a full256 workload.

The [receipt](../evidence/dialogue-token-session-validation-2026-09-27.json)
contains source/executable/bundle/protocol identities, actual decisions/output,
the sealed local report's file hashes, prior build failure and cumulative elapsed
accounting. A stale shared tokenizer dependency initially lacked the new module;
rebuilding that dependency from the unchanged local source resolved the compile
failure without cache deletion. The passing tests took 0.09 seconds, their
build/check command 13.05 seconds, the witness build 3.56 seconds, and the loaded
model witness 4.32 seconds. These debug execution timings are not an optimized
serving or energy result. No new fit or capability promotion occurred.
