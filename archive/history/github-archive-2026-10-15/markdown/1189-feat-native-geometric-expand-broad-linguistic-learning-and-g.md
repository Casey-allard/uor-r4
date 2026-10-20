# #1189 — feat(native_geometric): expand broad linguistic learning and general prose (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T04:45:53Z
- **Updated:** 2026-09-08T04:47:05Z
- **Closed:** 2026-09-08T04:46:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1189
- **Merged:** 2026-09-08T04:46:53Z
- **Merge commit:** 936d406a6330b743a0963add1e625b3fc7f781d6
- **Base / Head:** main / UOR-Foundation:issue-973-general-prose
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR addresses Roadmap Position 03 ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973): *Integrate the geometric model and learn general prose*) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820), expanding broad linguistic learning across diverse genres, verifying repetition entropy, and establishing unified coexistence of all roadmap primitives in the native model.

### Key Changes
1. **Multi-Domain Curriculum Training**:
   - Ingested diverse curriculum catalog spanning narrative prose, technical exposition, procedural Q&A, and structured code.
   - Scaled vocabulary configuration (`max_lexical_pieces: 256`, `candidate_limit: 32`) compiling geometric transition rows without cross-domain collision.
2. **Multi-Domain Continuous Generation**:
   - Verified coherent continuations across narrative, technical, and procedural dialogue prompts with non-trivial text length.
3. **Punctuation & Whitespace Stability**:
   - Verified correct handling of complex punctuation (`.`, `,`, `:`, `?`, `\n`) and capitalization without tokenization panics.
4. **Procedural Q&A Topical Relevance**:
   - Verified structured question-answering prompts emitting curriculum-grounded topical terms (`observes`, `calculates`).
5. **Repetition Entropy & Diversity**:
   - Verified multi-token emission avoiding degenerate absorbing single-token repetition loops (`max_consecutive <= 8`, distinct tokens $\ge 3$).
6. **Unified Capability Coexistence**:
   - Direct verification that continuous prose generation, durable session memory (#962), grounded provenance evaluation (#954), multi-step reasoning DAGs (#955), and standalone compiling Rust synthesis (#1088) seamlessly coexist in the same native model instance.
7. **Architectural & Safety Invariants**:
   - Zero runtime heap allocations on `#![no_std]` hot paths.
   - Zero forbidden arithmetic (`*`, `/`, `%`) or float types in the integer serving kernel.

## Verification
- `cargo test -p uor-r4-core --lib general_prose_tests --offline`: 10/10 passed.
- `cargo test -p uor-r4-core --lib workspace_coding_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --lib multi_step_reasoning_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --lib groundedness_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --test native_geometric_allocations native_kernel_source_has_no_forbidden_arithmetic_or_float_types --offline`: 1/1 passed.
- `cargo fmt --check`: passed.
- `python3 scripts/check_claim_wording.py`: passed (0 violations).

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

