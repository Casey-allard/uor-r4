# #1397 — Deliver full256 standalone integer serving and exact generation speedup

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T20:15:37Z
- **Updated:** 2026-10-09T13:17:34Z
- **Closed:** 2026-09-25T20:16:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1397
- **Merged:** 2026-09-25T20:16:46Z
- **Merge commit:** 413a32fcf55f13a267862fd823109f8b459cafa9
- **Base / Head:** main / UOR-Foundation:codex/integer-serving-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Delivered

- Extract the accepted learned-code integer model into `uor-r4-integer`, sharing the same numerical source with offline training evaluation. Add a shared byte-BPE crate while preserving core compatibility.
- Package sealed, tokenizer-bound model/table bundles; provide a standalone generation CLI and persistent in-memory text sessions, with integer greedy/categorical selection and explicit full256 context errors.
- Use one native profile to target the actual arithmetic hotspot. Reuse exact signed4 input multiples across dense rows; no model, scale, table, admission or threshold changes.
- Preserve 256/256/256 and all causal occurrences. No fitting or new evaluation population.

## Executed evidence

- Both extraction and optimized implementations match all4,096 retained Q48 distribution hashes and greedy predictions, all128 source rows and341 source decisions.
- Complete source answers remain quaternion28/32 and ordinary24/32; NoRead0/32 each, zero lost correct rows. Actual multi-call session versus raw occurrence replay passes for both artifacts.
- All10 new integer categorical continuations are exactly unchanged by optimization, excluding timing only.
- Whole CLI through JSONL: source batches52.97/53.15s ->8.02/8.01s; continuation batches11.93/10.76s ->2.62/2.14s. Single before/after observations, two model arms concurrent; not an energy comparison.
- Final compiled numerical audit:55 ranges/12,756 ARM64 instructions; no floating arithmetic/conversion or model/probability/PRNG value multiply/divide. Shape/address/clock exceptions remain explicit.

## Validation and limits

24 integer and3 tokenizer unit checks PASS; extended signed4 arithmetic check PASS; standalone optimized bins build; training compatibility with original metal,cpu-accelerate,reference-accelerate features PASS; loaded comparisons PASS; formatting and claim wording PASS. Two core integration targets unexpectedly triggered unrelated binary builds and were canceled before tests: NOT_RUN. Compatibility CI acknowledgements are not tests.

Dense parameter access, allocation, finite context, weak prose and full-path arithmetic/energy obligations remain. The new categorical policy is not seeded parity with the old floating sampler. No geometric superiority, Hamiltonian dynamics, useful general language, coding or D5 sparsity is claimed.

RDC ran a substantive DeepSeek review and concurrent local Rust workers. Independent agents reviewed extraction, tokenizer, arithmetic, compiled scope and row comparisons. Existing artifacts and negative results are preserved. Current state, README, canonical plan and evidence now point to one integrated coherent-language continuation through this serving path, with meaningful learning exposure and explicit output-quality decisions before launch.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

