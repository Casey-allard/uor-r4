# #1409 — Feed literal-role dialogue through exact integer tokens

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T02:03:49Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-27T02:05:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1409
- **Merged:** 2026-09-27T02:05:06Z
- **Merge commit:** f8bee0fff17868e8f467166b3aaf4d2e46ca78ce
- **Base / Head:** main / UOR-Foundation:codex/dialogue-token-session-20260926
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Literal-role dialogue training encodes role markers and content as separate BPE segments. Rendering the same visible prompt and encoding it again can change the IDs; dedicated-role fallbacks differ further. Add an opt-in adapter that binds `DialogueProtocol` to the bundle tokenizer and feeds its exact prefix into the retained full-context integer session, consuming one initial BOS.

`TextSession::append_tokens` validates the complete token list and prospective capacity, including pending output, before mutation. `Bundle::dialogue_session` and `generate_dialogue` use the protocol's EOS semantics and expose its identity. Existing text generation retains an actual optional turn-end token. Both generation and response slicing now preserve its absence; the old fallback to token 6 could stop or truncate ordinary `$` text with the retained tokenizer.

The existing chatbot CLI, partitioned memory session, corpus builder and panel are separate consumers. Existing bundles are unchanged; the explicit protocol is an evaluation binding, not a claim of dialogue training provenance. The roadmap also records the selected shared unit-scale initializer design before a costly radial-reader fit; that initializer and fit are not implemented or run here.

Executed validation on source `79c7c9e0d9cb712af0361cf1dd9d21cfdd8cd9d0`:
- Six focused generation tests pass: four new ingestion/binding/stopping checks and two retained budget/cycle checks.
- The actual accepted quaternion bundle agrees under greedy and seeded top-40 selection across the adapter, exact-ID append and direct `IntegerModel::step` plus `Sampler`. The 13 input IDs and all 16 emitted prediction records match, including full probability-vector hashes, causal slots, NoRead mass and sampler state.
- The actual short responses remain incoherent for chat. The sealed witness is an input/prediction transport result, not a language, performance or energy qualification.
- Independent source review found no remaining blocking issue. Touched Rust formatting, diff and claim-wording checks pass.
- Passing test/build command 13.05 s, witness build 3.56 s, actual-bundle witness 4.32 s. An earlier 1.15 s compile failure used a stale tokenizer dependency in the shared cache; rebuilding that dependency from unchanged source resolved it without cache deletion.

Full source/artifact/output identities, prior failure and cumulative elapsed accounting are in `docs/evidence/dialogue-token-session-validation-2026-09-27.json`. No fit, paid compute or unique-artifact deletion.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

