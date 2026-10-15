# #322 — Record pinned SmolLM2 135M teacher baseline provenance

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T00:54:38Z
- **Updated:** 2026-08-01T01:06:58Z
- **Closed:** 2026-08-01T01:06:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/322
- **Merged:** 2026-08-01T01:06:58Z
- **Merge commit:** 6dce9be26433b31bbce9273bf36cbb086b292937
- **Base / Head:** main / UOR-Foundation:issue-320-smollm2-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

P1 of #320: pin and rehearse the SmolLM2-135M-Instruct teacher path without changing the stories15M fixtures.

- Records the immutable Hugging Face revision in the model descriptor.
- Records the loaded `model.safetensors` source κ and byte size.
- Documents the reproducible download and compile commands.
- Records a bounded 1,000-token/11-story pipeline rehearsal and its 114.1 teacher tokens/s throughput.
- Clearly marks the rehearsal as non-quotable; P2 still needs a complete corpus and row-matrix comparison.

## Validation

- `jq empty models/smollm2-135m-instruct.json`
- `cargo fmt --all -- --check`
- `cargo test -p uor-r4-graph-cli --lib --offline`
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`

The workspace gates were run on the unchanged code path; this PR only adds metadata and documentation.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

