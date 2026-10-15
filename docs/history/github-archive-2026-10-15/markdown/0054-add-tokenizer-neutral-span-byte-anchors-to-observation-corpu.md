# #54 — Add tokenizer-neutral span/byte anchors to observation corpus records

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-22T03:18:07Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-07-22T17:47:41Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/54
- **Merged:** 2026-07-22T17:47:40Z
- **Merge commit:** 9fc1616cfc4d5024244f6c7953dd75592eb837e4
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-uor-r4-29-tokenizer-neutral-span-by
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This change introduces tokenizer-neutral provenance anchors in the observation schema so token boundaries are not treated as semantic concepts. It extends the tokenizer export path and corpus record format to carry stable span/byte location metadata while keeping backward compatibility with existing corpora.

- **Observation schema: add anchor fields**
  - Extended `Corpus` with:
    - `span_start`, `span_end` (token-position spans)
    - `byte_start`, `byte_end` (UTF-8 byte spans)
  - Updated corpus record docs/handling to support:
    - v1: 12-byte legacy records
    - v2: 32-byte records
    - v3: 48-byte records with anchors

- **Corpus IO: backward-compatible read + forward write**
  - `load_corpus_from` now parses v3 anchors when present.
  - For v1/v2 records, anchors are backfilled deterministically:
    - span anchors inferred per story
    - byte anchors marked unknown (`u32::MAX`)
  - Generation path now emits anchor-capable records and preserves resume behavior across legacy/new formats.

- **Tokenizer export: expose byte-length surface**
  - Added `export_hf_bytelevel_tokenizer_with_lengths(...) -> io::Result<Vec<u32>>`.
  - Existing `export_hf_bytelevel_tokenizer(...)` remains as compatibility wrapper.
  - Hugging Face compile flow now uses exported per-token byte lengths to populate observation byte anchors.

- **Tests: anchor coverage + struct updates**
  - Added focused tests for:
    - v3 anchor round-trip
    - v2 backfill semantics
    - tokenizer export byte-length output
  - Updated existing `Corpus` literals in impacted tests for new fields.

```rust
pub struct Corpus {
    pub n: usize,
    pub stories: u64,
    pub story: Vec<u32>,
    pub input: Vec<u32>,
    pub next: Vec<u32>,
    pub t_argmax: Vec<u32>,
    pub top_tokens: Vec<[u32; 3]>,
    pub top_weights: Vec<[u32; 3]>,
    pub span_start: Vec<u32>,
    pub span_end: Vec<u32>,
    pub byte_start: Vec<u32>,
    pub byte_end: Vec<u32>,
}
```

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3632495041 — 2026-07-22T17:47:50Z — Copilot

_on crates/uor-r4-core/src/transformerless/compiler.rs:258_

`record_size` detection reads the entire records file into memory (`std::fs::read(rp)`) just to inspect its length. For large corpora this can be extremely slow and memory-heavy (potential OOM). Use `std::fs::metadata(rp)?.len()` (or `File::open` + `metadata`) to get the size without loading the file contents.

### review 4757136443 — 2026-07-22T17:47:51Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR adds tokenizer-neutral provenance anchors (token-span and UTF-8 byte-span) to transformerless observation corpus records, extends the tokenizer export path to return per-token byte lengths, and updates generation/loading to remain compatible with legacy corpus record formats while enabling anchor-capable records going forward.

**Changes:**
- Extended `Corpus` with `span_start/span_end` and `byte_start/byte_end`, plus loader support for v1/v2/v3 record sizes with deterministic backfill for legacy formats.
- Added `export_hf_bytelevel_tokenizer_with_lengths(...) -> io::Result<Vec<u32>>` and updated the Hugging Face compile flow to use it.
- Added/updated tests to cover v3 anchor round-trip, v2 backfill semantics, and tokenizer export byte-length output.

### Reviewed changes

Copilot reviewed 7 out of 7 changed files in this pull request and generated 3 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/compiler.rs | Adds anchor fields to `Corpus`, parses v3 anchors on load, backfills anchors for v1/v2, and writes anchor-capable records with legacy-size resume behavior. |
| crates/uor-r4-core/src/transformerless/scenarios.rs | Introduces tokenizer export that also returns per-token UTF-8 byte lengths while preserving the old wrapper API. |
| crates/uor-r4-core/src/transformerless/command.rs | Updates HF compile flow to export tokenizer + lengths and pass lengths into corpus generation for byte-anchor population. |
| crates/uor-r4-core/tests/observation_anchors.rs | New tests covering v3 anchor preservation, v2 backfill behavior, and tokenizer byte-length export. |
| crates/uor-r4-core/tests/transitions_test.rs | Updates `Corpus` literals to include new anchor fields; minor formatting changes. |
| crates/uor-r4-core/tests/hierarchical_compiler.rs | Updates `Corpus` literals to include new anchor fields; minor formatting changes. |
| crates/uor-r4-core/tests/deterministic_rebuild_test.rs | Updates `Corpus` literals to include new anchor fields; minor formatting changes. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3632495078 — 2026-07-22T17:47:51Z — Copilot

_on crates/uor-r4-core/src/transformerless/compiler.rs:363_

`record[..record_size]` is written for all `record_size` values, but a 12-byte (v1) record has a different layout (u16 next/argmax + f32 logprob). Slicing the v3 buffer to 12 bytes will silently corrupt v1 corpora when resuming. Either implement proper v1 serialization here or explicitly refuse to write to 12-byte corpora.

### review-comment 3632495105 — 2026-07-22T17:47:51Z — Copilot

_on crates/uor-r4-core/src/transformerless/compiler.rs:352_

When `token_byte_lengths` is provided, falling back to `0` on a missing `next` index (`unwrap_or(0)`) will produce overlapping/incorrect byte anchors without any signal. If byte lengths are expected to cover the full vocabulary, it’s safer to fail fast (or mark anchors as unknown) when an index is missing.


---
_Comment/review ids in this file: 4757136443, 3632495041, 3632495078, 3632495105_

