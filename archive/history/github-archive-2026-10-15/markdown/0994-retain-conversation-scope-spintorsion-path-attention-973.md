# #994 — Retain conversation-scope SpinTorsion path attention (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-28T18:25:17Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-08-28T18:26:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/994
- **Merged:** 2026-08-28T18:26:40Z
- **Merge commit:** 5a83535387f0e0efa87da66250908e72e6e278c4
- **Base / Head:** main / UOR-Foundation:issue-973-conversation-spin-path-v1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements the independently frozen #973 conversation-scope contrast and retains `ConversationEntitySpinPathR4V1` at:

`RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL`

The corrected decoded matrix is real 2/2, conversation-disabled 1/2, cross-turn-binding-permuted 0/2, and binding-row-reversed 2/2, with zero support/work mismatches and exact target-free/decoded replay.

The hierarchy contrast keeps current-through-paragraph and global identities/ordered H4 states equal while changing only conversation identity. Candidate support and the accepted #953 table/overlay remain unchanged; the retained paragraph implementation and artifact are untouched.

An independent pre-delivery falsifier rejected a preliminary run because its grammar identity omitted full construction/held-out boundary binding and disabled did not literally repeat entity resolution. Both implementation defects were corrected before this commit and the complete focused evidence test was rerun. Only the corrected identities in the record are accepted.

## Evidence

- operator: `blake3:343c961b06605f6ae9bb6160ac34a98224991715b706156349a8fd544b6dbb35`
- target-free census: `blake3:649d733a194469aa648101a873d9e2ee323266b18872ced412d1da2cc6a56635`
- decoded smoke: `blake3:6930de3c07d30df4420bb68e60ea74531c8076516bcfef1c016240eddf1b9ca2`
- full record: `docs/conversation_entity_spin_path_attention_973.md`

## Activated checks

- PASS: `cargo test -p uor-r4-core --test conversation_entity_spin_path_attention_973 --offline -- --nocapture` — 2 passed, 0 failed, 264.10s
- PASS: `cargo check -p uor-r4-core --all-targets --offline`
- PASS: `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- PASS: `cargo fmt --all -- --check`
- PASS: `python3 scripts/check_claim_wording.py`
- PASS: `git diff --check`

Workspace-wide tests, BDD, teacher/model paths, no-std, Gate C, kappa reproduction, audit, fuzz, conformance, corpus-scale, product, performance, formal, and release suites are `NOT_RUN` under the frozen decision-bearing boundary.

## Claim boundary and tracker

This is one bounded synthetic, construction-bound exact-descriptor cross-turn entity-role stored-spin path selector. It is not a coordinate-specific H4/fiber/torsion result, semantic or natural transfer, a geometric advantage over direct ordered binding lookup, a general entity/conversation model, or general conversation/global attention.

References #973. #973 stays open for one independently frozen bounded-global contrast, then corpus induction; #954 remains blocked.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

