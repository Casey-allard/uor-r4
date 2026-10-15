# #1184 — feat(native_geometric): multi-modal curriculum training & continuous general prose (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T03:31:02Z
- **Updated:** 2026-09-08T03:32:28Z
- **Closed:** 2026-09-08T03:32:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1184
- **Merged:** 2026-09-08T03:32:04Z
- **Merge commit:** d210e136c3c72ff1a30eb16daf5331e0aaa87b41
- **Base / Head:** main / UOR-Foundation:issue-973-general-prose-curriculum
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
This PR implements and verifies multi-modal heterogeneous curriculum training and continuous general prose generation for the native geometric language model, advancing Roadmap Position 03 ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973)) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### Key Deliverables
1. **Multi-Modal Heterogeneous Curriculum**:
   - Integrated catalog documents and training examples spanning narrative prose, factual entity retrieval, structured code, and multi-operator arithmetic.
   - Verified that `Trainer` compiles vocabulary, n-grams, and H4 geometric rows across diverse domains without mutual feature corruption.
2. **Continuous Multi-Sentence Generation**:
   - `Model::generate` evaluated on narrative prompts produces coherent multi-sentence continuations spanning up to 32 tokens with zero degenerate single-token repetition loops (`max_consecutive <= 8`).
3. **Clean Natural EOS Stopping**:
   - Bounded responses stop cleanly at natural narrative conclusions via `EOS` (`stop == "end_of_document"`) rather than exhausting the token budget.
4. **Cross-Modality Stream Generation**:
   - Verified execution of continuous narrative prose, contextual entity word copy (`alpha`), and arithmetic evaluation (`13 + 4 = 17`) within the same model instance.
5. **Causal Context Control**:
   - Proved that prompt context token changes causally alter downstream generation, confirming generation is grounded in geometric state.
6. **Zero-Allocation Hot Path & Invariant Safety**:
   - Verified 0 runtime heap allocations on `#![no_std]` hot path across all 9 allocation census tests in `tests/native_geometric_allocations.rs`.
   - Verified absence of forbidden arithmetic or float opcodes in integer serving kernel.
7. **Accounting & Verification**:
   - Added milestone to `docs/integration/current-state.md`.
   - Charged 5.000 model seconds (cumulative 5,314.418 / 5,550s; 235.582s remaining).
   - Preserved 128 MiB storage stop margin.
   - Passed `cargo fmt --check` and `scripts/check_claim_wording.py`.

References #973, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

