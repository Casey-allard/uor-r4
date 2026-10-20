# Exact-token conversation continuity

This integration implements a persistent library adapter over the retained
integer text session. It addresses the history reconstruction and per-request
seed reset described in the [fourth-lab work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5852795123).
The [current state](current-state.md) owns programme status. Five tokenizer and
ten integer-generation checks pass, including five new focused checks. A single
two-turn actual-bundle witness matches all eight prediction records against
direct integer stepping, with 36 total conversation steps and no prefix replay.
The [validation receipt](../evidence/dialogue-continuity-validation-2026-09-27.json)
binds source, executable, exact output, checks and resource cost. The outputs are
capped fragments, not useful chat; the result establishes token/state transport.

## API and retained history

[`generation::DialogueConversation`](../../crates/uor-r4-integer/src/generation/conversation.rs)
is constructed by:

```rust
bundle.dialogue_conversation(protocol, initial_history, seed, read_mode)
```

Each response takes:

```rust
ConversationRequest {
    user,
    max_new_tokens,
    policy,          // SamplePolicy::Greedy or Categorical { top_k }
    first_sentence,
    closure,         // TurnClosure::RequireModelEos or InterruptAssistant
}
```

Construction binds the tokenizer and protocol, requires BOS0/EOS1 and equal
model/tokenizer vocabularies, and validates the encoded initial history. It
executes no model steps. Model state starts only after the complete first
`respond` request passes preflight; `len()` and `step_calls()` are zero before
then. Protocol, initial history/persona and read mode remain immutable.

The [shared encoder](../../crates/uor-r4-tokenizer/src/dialogue.rs) supplies
`encode_open_history` and `encode_user_prefix`. Initial history keeps V1's
normalization, skipped-message rules and completed assistant EOS tokens, without
adding the corpus-only terminator after a non-assistant message. There is no
turn-alternation restriction on supplied history. Each `respond` requires a new
nonempty user message after normalization. Its separator, literal `User: `,
content, separator and literal `Assistant: ` are encoded independently; empty
initial history needs no leading separator. BOS is consumed once at startup.

Generated IDs remain in the same integer session, including its last pending
token. They are never reconstructed from response text. This preserves
training-compatible framing and exact generated history, not equality to a
fresh serialization of arbitrary generated text: alternative BPE segmentation,
edge whitespace, carriage returns, empty output and incomplete UTF-8 need not
match the training serializer's normalized complete content string.

## Turn boundaries, selection and output

`ConversationTurn.boundary` records the boundary **before the current user**:

| Previous state | Boundary and action |
|---|---|
| First response | `InitialHistory`; append the initial history and user prefix. |
| Previous response selected EOS | `ModelEos { token }`; consume that pending generated EOS once, without inserting another. |
| Previous response stopped otherwise, with `InterruptAssistant` | `CallerEos { token, interrupted_stop }`; consume the pending generated token and append one caller-supplied EOS before the user prefix. |
| Previous response stopped otherwise, with `RequireModelEos` | Reject with `AssistantStillOpen`; preserve state. |

The interruption option does not add EOS after an actual model EOS. A token cap,
sentence boundary or short cycle is not model completion. Caller EOS is input,
not generated output. This API starts a new user turn; it does not provide an
assistant-only continuation operation.

`appended_token_ids` contains newly supplied input, including initial BOS/history
on the first call and any caller EOS, but excluding prior generated IDs.
`dialogue.generation` retains exact prompt/generated IDs, prediction hashes,
stopping reason and the existing generation record. `raw_generated_bytes`
decodes the generated IDs without trimming or UTF-8 replacement, including any
generated EOS surface. `response_text` and `raw_decoded` are display strings;
neither is a history store. A partial UTF-8 sequence remains recoverable through
IDs and raw bytes.

Categorical calls carry a persistent sampler cursor across turns; greedy calls
leave that cursor unchanged. Use `DialogueConversation::sampler_state()` for the
conversation cursor. The nested generation's `sampler_state_after` is normalized
to that same authoritative continuation cursor, including after a greedy call.
Existing top-k semantics
remain: zero or at least vocabulary size selects from the full vocabulary;
top-k one is greedy and consumes no random draw. Seed zero keeps the existing
documented replacement. MinP is rejected. There is no new decoder policy or
claim of chunk-invariant sentence/cycle stopping.

Literal special-token text retains V1's unescaped atomic encoding. An EOS inside
user content is observed input; model-selected EOS stops generation. Optional
dedicated turn-end tokens receive no delimiter meaning here. Literal markers
and control-token text do not establish trusted role isolation. The
[existing protocol contract](dialogue-protocol-contract-2026-09-26.md) remains
the serializer authority.

## Rejection, execution failure and integration limits

Preflight checks the supported policy, preceding assistant boundary, normalized
user content, every appended ID and the complete requested input-plus-output
capacity before changing model or sampler state. Zero output budget, empty
normalized user input, unsupported policy, an unclosed assistant or excessive
capacity is a rejection, not a partial turn. The capacity check includes the
previous pending token, any caller EOS and the requested maximum output length,
even if generation might select EOS earlier.

Once execution starts, an error may follow partial state changes. It returns
`Execution`, poisons the conversation and rejects later responses and sampler
cursor access. There is no rollback or automatic retry. `len()` and
`step_calls()` after poisoning are diagnostic only; continuing requires an
explicit new conversation. The adapter never silently evicts, truncates, resets
or replays history. Retained bundles remain limited to full256 capacity.

Google retains `uor-chat` CLI integration ownership. This library supplies a
reusable adapter; it does not change that CLI, its partitioned memory path, model
loaders, a sealed bundle or the running three-arm reader study. The declared
witness uses the unchanged accepted quaternion Dot bundle and tokenizer with
zero training updates. Source `b4c2baa1` produced all recorded checks and the
witness. Build/check work took 112.04 seconds and the probe 2.576 seconds, with
15.81 MB peak child RSS; these debug-build observations do not qualify optimized
serving performance. Both four-token responses stopped at their caps: `“Is there
the` and `1 asked the people`. Exact prior IDs, an explicitly recorded caller
EOS and the next user segments reach the second prediction stream.

Correct token continuity does not establish conversational training, prose,
memory or reasoning quality, geometric advantage, or serving efficiency. The
fourth lab retains the owner's native, multiplier-free serving target; this
adapter adopts no D10 backbone or multiplier exception and does not certify the
complete numerical path by association.
