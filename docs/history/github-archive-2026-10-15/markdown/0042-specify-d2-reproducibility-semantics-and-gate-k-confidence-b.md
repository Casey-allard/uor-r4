# #42 — Specify D2 reproducibility semantics and Gate K confidence-bounded empirical claims protocol

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-22T00:28:05Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-07-22T05:52:51Z
- **Assignees:** auser, afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/42
- **Merged:** 2026-07-22T05:52:51Z
- **Merge commit:** 027fa967ab49b68018d6879dbf85c9b96beb0278
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-uor-r4-33-define-byte-reproducibili
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR resolves the missing normative spec for behavioral graph equivalence vs byte reproducibility (PDF §15 / D2) and formalizes the statistical protocol required for empirical claims. It closes the gap called out in Plan §9.23 by turning placeholder language into a concrete, referenced specification.

- **Spec addition: BR vs BE + empirical protocol**
  - Added `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md` as the normative source for:
    - **Class BR (byte reproducible)** for certificate-bearing artifacts (identical pinned inputs ⇒ identical bytes/CIDs).
    - **Class BE (behaviorally equivalent)** for non-canonical/platform-accelerated compiles using declared equivalence margins and confidence bounds.
    - Gate K claim requirements: mandatory claim record fields, estimator/CI requirements, and acceptance rules for threshold, equivalence, and non-inferiority claims.

- **D2 integration in wire-format RFC**
  - Updated `docs/transformerless/R4G1.md` §7 to reference the new normative spec and remove “working assumption” ambiguity.
  - Clarifies BR is required for Gate E artifacts; BE is permitted for local accelerated compiles under the protocol.

- **Plan §9.23 linkage**
  - Updated `docs/r4_graph_compiler_implementation_plan.md` backlog item 23 to point directly to the new spec document.

```markdown
- Lower-bounded claim (`metric >= t`): accepted iff CI lower bound `>= t`.
- Upper-bounded claim (`metric <= t`): accepted iff CI upper bound `<= t`.
- Equivalence claim (`|delta| <= epsilon`): accepted iff CI for `delta` is fully contained in `[-epsilon, +epsilon]`.
```

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3627123306 — 2026-07-22T03:02:07Z — Copilot

_on docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md:55_

The phrase “very small n” is undefined in this normative spec, which makes the allowed interval method for rates/proportions ambiguous and hard to implement consistently. Consider allowing either Wilson or exact (Clopper–Pearson) binomial intervals unconditionally, with the chosen method recorded in the claim record.

### review-comment 3627123320 — 2026-07-22T03:02:07Z — Copilot

_on docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md:60_

The spec doesn’t define whether “CI” is one-sided or two-sided. Because the acceptance rules for lower/upper-bounded and non-inferiority claims only check one bound, implementers could reasonably use either a two-sided 95% CI or a one-sided 95% CI, which changes gating outcomes. Please explicitly specify sidedness for intervals used in §2.2.

### review 4750615355 — 2026-07-22T03:02:08Z — copilot-pull-request-reviewer[bot] — COMMENTED

## ⚠️ Not ready to approve

The new normative protocol spec contains ambiguities (e.g., CI sidedness and an undefined “very small n” cutoff) that could lead to inconsistent or conflicting implementations of gating decisions.

<details>
<summary>Pull request overview</summary>

This PR formalizes decision D2 by defining normative reproducibility semantics (byte reproducible vs behaviorally equivalent) and establishing a confidence-bounded protocol for empirical claims (Gate K), then wiring those definitions into the R4G1 RFC and the implementation plan.

**Changes:**
- Adds a new normative specification document for BR/BE and Gate K empirical-claim acceptance rules.
- Updates `R4G1.md` §7 to reference the new spec and remove “working assumption” language.
- Updates the implementation plan backlog item to link directly to the new spec.
</details>

<details>
<summary>File summaries</summary>

| File | Description |
| ---- | ----------- |
| docs/transformerless/R4G1.md | Replaces the prior D2 “working assumption” text with a normative reference to the new BR/BE + Gate K protocol spec. |
| docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md | Introduces the normative BR/BE definitions and a confidence-bounded empirical-claims protocol with acceptance rules. |
| docs/r4_graph_compiler_implementation_plan.md | Links plan item 23 directly to the new normative spec document. |

### Review details

- **Files reviewed:** 3/3 changed files
- **Comments generated:** 2
- **Review effort level:** Low
</details>

> [!NOTE]
> Your feedback helps us improve the quality of this feature.
> Please use 👍 or 👎 to tell us whether this assessment is correct.

---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.


---
_Comment/review ids in this file: 4750615355, 3627123306, 3627123320_

