# #36 — Clarify R4G1 RFC invariants and Phase‑1 width freeze semantics

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-21T22:13:42Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-07-21T22:51:49Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/36
- **Merged:** 2026-07-21T22:51:49Z
- **Merge commit:** 458231458fd132c01b14af150333deae7ef909bb
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-uor-r4-11-draft-r4g1-container-spec
- **Reviews / inline review comments:** 1 / 1
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This updates the R4G1 wire-format draft to fully pin the validation contract described in the implementation plan: sectioned container rules plus Stage‑1/Stage‑2 `GraphView` safety invariants. It also makes the Phase‑1 “freeze packed widths” milestone explicit and normative.

- **RFC contract alignment (Plan §5 Phase 0/1)**
  - Tightened wording in `docs/transformerless/R4G1.md` so packed-record width decisions (`PackedNode`, `PackedEdge`, `RoutingOp`, `EmissionEntry`) are explicitly frozen in Phase 1.
  - Added normative behavior for pre-freeze artifacts: draft readers/writers reject artifacts claiming stable numbered R4G1 versions with unfrozen packed widths.

- **Validation invariant mapping (Plan §6 A1/T8)**
  - Added an explicit phase/obligation mapping after the Stage‑2 invariants:
    - Phase 0: RFC + invariant publication
    - Phase 1: two-stage validator + typed borrowed `GraphView<'a>` implementation + width freeze
    - A1/T8: validated-only `GraphView` and checked-ranges-to-typed-view soundness

- **Illustrative excerpt**
  ```markdown
  Phase mapping (normative intent):
  - plan §5 Phase 0: publish this RFC draft and structural/semantic invariant set.
  - plan §5 Phase 1: implement two-stage validator + typed `GraphView<'a>` and freeze packed widths.
  - plan §6 obligations: this section operationalizes A1 and T8.
  ```

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 4749492790 — 2026-07-21T22:27:00Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR refines the R4G1 wire-format RFC to make validation obligations and Phase-1 packed-width “freeze” semantics explicit and normative, aligning the document’s stated contract with the implementation plan phases and `GraphView` safety invariants.

**Changes:**
- Updates the packed-record width (TBD-w) language to state widths are frozen in Phase 1 and adds a pre-freeze artifact rejection rule.
- Adds a normative phase-to-obligation mapping under the Stage-2 semantic invariants, tying the section to A1/T8.







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626125476 — 2026-07-21T22:27:00Z — Copilot

_on docs/transformerless/R4G1.md:57_

The new rejection rule is hard to implement as written because the spec doesn’t define what constitutes a “numbered stable R4G1 version” vs a draft version. Since the header already has `format_version (major, minor)`, it would be clearer to make the pre-freeze rule explicitly about reserved version numbers (e.g., major>=1 only after packed widths are frozen), so draft readers/writers can deterministically accept/reject artifacts.


---
_Comment/review ids in this file: 4749492790, 3626125476_

