# #1093 — Measure token-frame exposure and advance the language interface plan

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T04:23:15Z
- **Updated:** 2026-10-09T14:16:52Z
- **Closed:** 2026-09-03T04:25:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1093
- **Merged:** 2026-09-03T04:25:08Z
- **Merge commit:** a7f62b025c707640058e48721ef4971f8be789c5
- **Base / Head:** main / UOR-Foundation:issue-1082-token-exposure
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous token-frame control changed many matrices but did not show how much each learned role actually attended to those changes. This adds the separately frozen construction-only exposure diagnostic, its immutable preparation/result/replay envelopes, independent review, and current-roadmap handoff.

The two retained views show role-selective exposure: view 0 barely exposes fact locations (~0.0035% changed-frame attention), while view 1 barely exposes fact objects (~0.0039%). Strongly exposed roles have D/A close to one; historically retained answers coexist with substantial pooling displacement. These are descriptive results and preserve #1079's `LANGUAGE_R4_PRESERVED_CONTROL_WEAK` verdict. #973 stays open and #954 blocked.

The contract was posted before outcomes in [the native freeze](https://github.com/UOR-Foundation/uor-r4/issues/1082#issuecomment-5520228872). The one run and one fresh-process replay reproduced all eight historical bindings, all 286,720 used-role measurements, both raw metric byte streams and complete summary evidence exactly. Total internal time was 31.159102208 seconds (33.659554750 externally), maximum RSS 0.9890594482 GiB. No new predictions, downstream/head forwards, fitting, development scoring, controls, geometry changes or generation occurred. Raw metrics remain local with their CIDs; public immutable envelopes are committed.

Four focused synthetic checks passed; formatting/lint and claim wording passed. Independent review rehashed 316 bound implementation files, canonical envelopes and raw metrics, and reconciled bounds/summaries without rerunning the model. All 13 claim-ledger source files match their pinned commit. Broad QA remains NOT_RUN; required GitHub statuses are transport acknowledgements.

Current entry points, claim ledger and reusable continuation prompt now select #1085: specify removal of externally supplied clause segmentation, preserving the accepted reader/core, known lexicon/query form and four-fact context. A later empirical comparison needs its own frozen contract. This PR does not implement that successor.

Closes #1082. References #973 and #1085.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5520316194 — 2026-09-03T04:23:34Z — Casey-allard

# #1082 delivery pointer and claim review

Reviewed 2026-09-03 in the `r4-token-exposure` worktree against finalized science commit `c1c45fa098d24f4ceae3c8fc207f0ecd07d62e20`, the diagnostic record, public preparation/result/replay envelopes, and independent scientific review. Scope was the ten changed current documentation/claim-ledger files. No model execution, tests, new scoring, source edits, or historical mechanics recertification.

Verdict: GO. The owner resolved the sole clarification, and the final source was re-read: the claim entry and current map explicitly identify the role means as supported-row means averaged across four fact slots. No blocking findings remain.

Confirmed:

- The diagnostic is descriptive and construction-only: 20,480 reader rows, 286,720 consumed-role measurements, exact fresh-process raw/summary replay, and eight historical reconstruction checks. No new answers, fitting, development scoring, generation, geometry, or replacement control is claimed.
- Reported elapsed time (31.159102208 internal seconds combined) and maximum RSS (1,061,994,496 bytes, approximately 0.9891 GiB) agree with the evidence. Near-zero exposure is not labeled exact zero.
- The text preserves #1079's coherent preservation and `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`. It does not turn pooling displacement/retained-answer association into a downstream causal explanation or geometric advantage.
- #973 remains open and #954 blocked. #1085 is a segmentation-interface specification, keeping the qualified reader/core and existing task restrictions. Any adapter comparison, independent population, or fit needs a separate frozen empirical contract; this delivery does not launch one.
- All 13 claim-ledger source URLs name the full science revision above, every path exists at that commit, and all 13 current source files are byte-identical to that commit. New artifact IDs match the finalized envelopes.
- The adoption receipt and current map preserve the #1092/#1081 historical delivery and distinguish adoption-time issue snapshots from current ownership. CONTINUE refreshes origin/main and current instructions before choosing an eligible task while preserving unrelated local work.

No additional tests or model checks are needed for this documentation review.



---
_Comment/review ids in this file: 5520316194_

