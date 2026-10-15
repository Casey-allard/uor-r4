# #1186 — feat(native_geometric): qualify grounded correctness, conflict handling & abstention (#954)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T04:05:08Z
- **Updated:** 2026-09-08T04:06:31Z
- **Closed:** 2026-09-08T04:06:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1186
- **Merged:** 2026-09-08T04:06:09Z
- **Merge commit:** f71dc675aeb8442e0c4794e108bec66d45625684
- **Base / Head:** main / UOR-Foundation:issue-954-grounded-correctness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
This PR implements and qualifies grounded correctness, conflict handling policy, and calibrated abstention for the native geometric language model, fulfilling Roadmap Position 05 ([#954](https://github.com/UOR-Foundation/uor-r4/issues/954)) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### Key Deliverables
1. **Typed Serving Outcomes & Causal Provenance**:
   - Implemented `GroundedOutcome` (`Answer`, `Conflict`, `Clarify`, `Abstain`) in `groundedness.rs`.
   - Attributed answers to exact verifiable ground-truth sources via `GroundedProvenance` (`DurableRelation`, `ContextSpan`, `Computed`, or `DirectLexical`) with confidence margins.
2. **Contradiction Detection & Revision Policy**:
   - Flagged unannounced contradictory assertions against existing ground truth as explicit `Conflict` outcomes (`ConflictStatus::PendingRevision`), surfacing `existing_value`, `conflicting_value`, and the relation ID.
   - Verified that explicit revisions (`Action 2`) resolve contradictions and restore grounded answer generation.
3. **Four-Case Micro-Population Benchmark Suite**:
   - Evaluated a 4-probe suite spanning supported global facts, conflicting assertions, answerable local facts, and unsupported out-of-scope queries.
   - Verified dual-denominator metrics: 100% whole-population accuracy (4/4), 100% answered-conditional accuracy (2/2), 50% coverage (2/4 answered), 100% abstention accuracy (1/1), and 100% conflict detection accuracy (1/1).
4. **Causal Context Intervention**:
   - Proved that actively removing a supporting fact (`forget`) causally converts a previously answered query into a calibrated abstention (`NoAdmissibleSource`), confirming causal load-bearing dependence on geometric memory.
5. **Preserved Refusal Distinctions**:
   - Verified distinct handling for lexical non-read (`NoReadSelected` / `NoAdmissibleSource`), numeric operator bounds failure (`NoOperationPrecondition`), and ambiguous entity queries (`Clarify`).
6. **Zero-Allocation Hot Path & Invariant Safety**:
   - In-tree allocation census tests pass with 0 runtime heap allocations on `#![no_std]` hot path.
   - Integer serving kernel retains zero float types and zero forbidden arithmetic opcodes (`*`, `/`, `%`).
7. **Resource Accounting & Compliance**:
   - Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,323.418 / 5,550s; 226.582s remaining).
   - Preserved 128 MiB storage stop margin.
   - Clean `cargo fmt --check` and `scripts/check_claim_wording.py` (0 violations).

References #954, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

