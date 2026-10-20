# #1263 — Learn shared read emit stop scheduling for language

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T02:31:21Z
- **Updated:** 2026-10-09T13:17:56Z
- **Closed:** 2026-09-14T02:33:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1263
- **Merged:** 2026-09-14T02:33:37Z
- **Merge commit:** 6e53b430465ad3dd62a54bd5dc9ecef3b4084b81
- **Base / Head:** main / UOR-Foundation:codex/shared-language-scheduling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The language path previously forced a second Read for every dependent prompt. This extends the existing dependent_language module with one learned Read/Emit/Stop policy and the shared recurrent transition for both direct and dependent questions. Actual final-answer/EOS trajectory search trains the table; no example kind, intermediate path or gold read count selects serving actions.

One build and one fit pass PASS_SHARED_LANGUAGE_SCHEDULING: 4,096/4,096 training, 1,536/1,536 development answers and exact paths, all 96 intervention families and six kind/split cells. Suppressing the learned extra-read choice preserves all 768 direct answers and removes all 768 dependent successes. Policy/read/cursor/stop suppression and AlwaysRead produce zero complete answers. ExactIdentity and FeedbackDisabled diagnostics also pass, so no geometric metric or emitted-feedback advantage is claimed.

Validation: optimized Rust build; 2 new plus 34 retained focused tests; all 1,536 Full traces reload and retain adjacent state continuity; seven earlier retained panel totals (13,248 / 512 / 640 / 1,408 / 128 / 9,984 / 9,984) unchanged; cargo fmt --all --check, claim wording and diff checks. All 46 prior sealed roots / 1,183 files and original dirty checkouts preserved. No retries or older fits. Independent review caught helper visibility and prefix exhaustion before compilation; both were corrected.

The corpus losslessly combines the preceding direct/dependent datasets; it is open development, not independent final holdout. One/two punctuation-delimited clauses and actual usable-continuation features remain explicit. All development policy rows were seen in training. Reader, updater, encoder and writer stay frozen; this is bounded scheduling integration, not general prose or joint primitive learning. Normal model 15baec48 is unchanged and unpromoted. No transformer, serving matrix product or external provider is introduced.

Updates the canonical next action to shared final-output credit for reader compatibility and query update inside this loop. Source-bound evidence, results and local original-path receipts are linked from docs/native_geometric_language_scheduling_973.md. Build/preparation/search/fit/controls charge is 183,792 ms within existing limits; no allowance extension, paid compute or cleanup. Queue compatibility acknowledgements are not tests.

References #973. References #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

