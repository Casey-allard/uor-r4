# #1472 — I1: six-view product store and sealed inference checkpoint for the stack (library, not yet wired)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T20:57:59Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-28T21:00:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1472
- **Merged:** 2026-09-28T21:00:38Z
- **Merge commit:** e4a15d1920fd3be44f8f6a012c65643807fab42f
- **Base / Head:** main / UOR-Foundation:lab/claude/i1-stack-store
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1 (Claude main). Class C** (checkpoint format and store interface). References #973 under #820.

**What it adds** (library only; not wired into the model's forward pass or the session):
- **`stack_store`**: a token-level bridge over the existing `scoped_memory::Memory`. It reuses Memory's semantics rather than reimplementing them:
  - six typed distinctions (the owner's charter): Current, PreviousAssertion (the previous record), PreviousDistinctValue and Initial, with Absent, NoHistory and Evicted lookups;
  - same-value reassertion;
  - correction versus conflict;
  - eviction never reported as absence;
  - pinned views;
  - lineage.

  It adds a fixed 16-code re-entry vocabulary (status × view), and a `to_bytes`/`from_bytes` round trip that rejects a foreign lineage and bad encodings.
- **`stack_checkpoint`**: a sealed **inference** checkpoint (`uor-r4.stack-checkpoint/1`). It holds the config, weights, tokenizer and protocol identity, data identities, training provenance and optional store state. `load_checkpoint` verifies the seal and every hash. It is not an optimizer or RNG resume.

**Review:** independent review (summary in the first PR comment): *APPROVE WITH FIXES*.
- Finding 1 is **fixed**: a model in served (QAT) mode is refused before any root is claimed, with a test.
- `main` is merged in.
- Findings 3–5 are low and follow up separately:
  - load takes no expected-lineage or tokenizer argument, so the session layer must check both;
  - the docs overstate "binary search";
  - four rejection paths are untested.

**Tests** (on the merged branch): 38/38 pass (`stack_store` 9, `stack_checkpoint` 7, `geometric_stack` 22). `cargo fmt --check` passes.

**Mapping note:** `Memory`'s relation field is a byte, so the relation token id travels as a length-delimited trailing word inside the entity key, with a fixed tag byte. It stays injective, and there is a test.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5878420435 — 2026-09-28T20:58:19Z — Casey-allard

**Independent review of `lab/claude/i1-stack-store`** (launched by Lab 1; a read-only reviewer pass that did not write the change). **Verdict: APPROVE WITH FIXES.**

**Confirmed:**
- There is no reimplementation: every write, read, save/load and history digest goes through `scoped_memory::Memory`.
- The four views and four lookups map one-to-one.
- Same-value reassertion, correction versus conflict, eviction-never-absence and lineage all come from `Memory`.
- Key injectivity holds: scope and entity are length-prefixed, and the entity field is 4k+4 bytes.
- Load validation runs `Memory`'s checks, then the lineage, tag byte, token encoding and value witness.
- There are no `unwrap`/`expect`/`panic` calls outside tests.
- 15/15 of the author's tests pass.

**Findings:**
1. **Medium: fixed in `7d60fad1`.** Checkpointing a model in served (QAT) mode would reload it in float. It is now refused before any root is claimed, with a test.
2. **Required: done.** Merge `main` in and re-run; 38/38 tests pass.
3. **Low, follow-up.** `load_checkpoint` takes no expected lineage and does not check the tokenizer, so the session layer must check both.
4. **Low, follow-up.** Doc precision: only the chain lookup is a binary search, while `record_ref` is a linear scan; "Evicted" wording for the previous-distinct view under capacity 1; "12 representative histories".
5. **Low, follow-up.** Untested rejection paths: the continuation and parse-score check, a witness mismatch on a released record, a checkpoint memory-record mismatch, and the save failure path that writes `error.json`.

Safe to merge as an unwired library.


---
_Comment/review ids in this file: 5878420435_

