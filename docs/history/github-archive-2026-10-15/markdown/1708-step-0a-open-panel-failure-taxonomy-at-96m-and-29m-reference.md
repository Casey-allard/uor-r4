# #1708 — Step 0a: open-panel failure taxonomy at 96M and 29M (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T02:30:36Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-05T03:25:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1708
- **Merged:** 2026-10-05T03:25:10Z
- **Merge commit:** de5de113faf3c88a616a0bf484d73785d5daf3df
- **Base / Head:** main / UOR-Foundation:codex/step0a-panel-taxonomy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Step 0a instrument from `docs/research/barrier-assessment-2026-10-05/final-assessment.md`. Training-free. No model ran and no Rust changed.

## What
- **Annotator:** Claude read and labelled every failing row of two open-panel grade reports (232 requests, qwen2.5:7b judge, 64 new tokens):
  - 96M chat-100m-C: 186 failing rows.
  - 29M chat-29m-B lr5e-4: 189 failing rows.
- **Rubric:** fixed before labelling. Labels are R (recall/anaphora/consistency), K (knowledge-absent), N (incoherent), I (instruction), T (truncation) and O (other).
- **Precedence:** N > R > I > K > T > O. It is deliberately generous to R.
- **Second pass:** a seeded 40-row subset was labelled a second time, without the first labels shown.
- **Report root:** `~/uor-r4-local/step0/a-taxonomy/`, claimed exclusively and sealed with `MANIFEST.sha256`. A copy is in `docs/research/step0/0a-panel-taxonomy/`, with the write-up in `docs/research/step0/0a-panel-taxonomy.md`.

## Results (share of failing rows)

| Category | 96M-C (n=186) | 29M-B lr5e-4 (n=189) |
|---|---|---|
| recall-anaphora-consistency | **4 (2.2%)**, 95% [0.8, 5.4]% | **5 (2.6%)**, [1.1, 6.0]% |
| knowledge-absent | 65 (34.9%) | 50 (26.5%) |
| incoherent | 26 (14.0%) | 48 (25.4%) |
| instruction | 17 (9.1%) | 19 (10.1%) |
| truncation at 64 tokens | 19 (10.2%) | 11 (5.8%) |
| other | 55 (29.6%) | 56 (29.6%) |
| R upper bound (R applies under N) | 5 (2.7%) | 5 (2.6%) |
| any context-dependent failing row | 14 (7.5%) | 11 (5.8%) |
| ill-posed request (material missing from panel) | 47 (25.3%) | 49 (25.9%) |

**Self-agreement:** 39/40 rows agree, **Cohen's kappa = 0.966**. The one disagreement is heldout-178, on the K/N boundary.

## Decision rule
Rule: if R is below 15% of failing rows at 96M, then "retrieval cannot move the open panel". Observed R is 2.2%, with a 95% upper bound of 5.4%.

> **retrieval cannot move the open panel; panel lever = corpus/knowledge (Step 5)**

The outcome does not depend on my R judgments. Only 8 of the 232 requests are multi-turn. All context-dependent failures together are 7.5% at 96M.

## Other observations
- **Ill-posed rows:** about 25% of failing rows are first-line extracts with the referenced material missing (salutations, "the following code:", `[placeholder]`). The panel should be repaired or these rows reported separately before M4 is read.
- **29M to 96M:** incoherent falls from 48 to 26, knowledge-absent rises from 50 to 65, and truncation rises from 11 to 19. Failures shift toward missing knowledge as fluency improves. This rests on one pair of models with one seed each, so it is suggestive only.
- **Empty code fences:** several coding replies open a code fence and stop (heldout-011, -022, -155 and -172 at 96M). Not diagnosed here.

## Caveats
- **Single annotator.** The second pass ran in the same session with row ids visible, so kappa is self-consistency, not inter-rater reliability.
- **Failing set comes from the grader.** The panel judge decides which rows are failing.
- **Scope.** The result covers this open panel only, not the D19 MQAR batteries.

## Checks
- `python3 scripts/check_claim_wording.py`: passed.
- No Rust was added, so `cargo fmt` and tests do not apply.
- The tabulator is small Python that counts fixed TSV labels. Its sha256 is in the JSON.

Do not merge without review.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5987814902 — 2026-10-05T03:55:30Z — Casey-allard

Post-merge note (Claude lab): this PR entered the merge queue without a recorded exact-head review; it was not queued by the Claude lab. **Checks reported by its author agent at the merged head:**
- `cargo fmt --check` passes;
- focused tests pass, or there is no Rust to test (0a is docs and labels);
- the report root is sealed.

The result is accepted as the Step 0 instrument record (see #820). Any defect found later is fixed forward. References #820


---
_Comment/review ids in this file: 5987814902_

