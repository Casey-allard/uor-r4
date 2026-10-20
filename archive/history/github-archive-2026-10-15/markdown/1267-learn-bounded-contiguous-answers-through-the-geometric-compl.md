# #1267 — Learn bounded contiguous answers through the geometric completion loop

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T04:50:08Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-14T04:51:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1267
- **Merged:** 2026-09-14T04:51:20Z
- **Merge commit:** 32d81d5264222409d75a703b091f39739324448a
- **Base / Head:** main / UOR-Foundation:codex/language-span
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained occurrence reader could emit only one word. Add bounded contiguous span candidates with exact occurrence and byte identity, an output-trained selector, and context-boundary credit from actual answer/EOS-compatible source spans. Reuse the existing geometric completion loop, updater and byte/EOS writer.

**PASS_TYPED_LANGUAGE_SPAN:** 2,624/2,624 training and 1,728/1,728 development answers, paths and boundaries; all 48 intervention families and 1,728 reloads. The new Full path preserves 3,712 prior answer/unresolved outcomes. All prior legacy controls remain equal. Disabled/truncated/extent controls establish dependence on the selected full span; ExactIdentity equals Full, so no new metric advantage is claimed.

Preserve two negative attempts: the first exposed an inseparable boundary representation without fitting; the second fit training but failed development and retention because unused records supplied context credit. The final correction preserves the fitted rule and all parent parameters, with no second rule search. All 51 earlier sealed roots plus three new roots, frozen sources/binaries and original dirty checkouts are preserved.

Validation: optimized offline Rust build, 44 focused tests, three explicitly executed/sealed reports, format, claim wording and diff checks. One compile failure and all work are charged. Model/build/fit/controls 993,360/2,100,000 ms; standing-authorized parent extensions recorded before work; no paid compute or cleanup. CI status names are compatibility acknowledgements, not local test evidence.

This is finite authored terminal phrase copying, with corpus-derived word roles and familiar grammar. Intermediate updates remain one word; independent final holdout and general prose remain unqualified. Retain normal model 15baec48, no promotion. Current state records whole-phrase intermediate query updates as the next action.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

