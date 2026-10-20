# #1611 — Continue learned geometric values with matched query-read credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T09:56:44Z
- **Updated:** 2026-10-09T13:17:28Z
- **Closed:** 2026-10-02T10:05:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1611
- **Merged:** 2026-10-02T10:05:57Z
- **Merge commit:** fa47d0c57da769663ba765472a0f668cb5703974
- **Base / Head:** main / UOR-Foundation:codex/geometric-query-credit-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The saved geometric value model could not be continued faithfully: the driver pinned its older context, initialized a new value head and compared donor logits against stale addresses. Passing read probabilities to the existing weighted-mean loss would also divide away NoRead mass. This change adds an opt-in continuation that reloads both learned artifacts and their exact next RNG state, validates the correct learned/new-address initial outputs, and compares query-read credit with uniform matched eligible mass.

Both arms use immutable offline teacher masks and raw integer query weights. Equal mass is enforced separately per example/head/lane/atom/root-category group, with a fixed denominator preserving NoRead/zero exclusions. Fractional loss weights, Boolean targets and raw integer query streams have separate byte records. Fresh optimizer moments are declared; original fresh-fit behavior remains available.

Actual release validation: three unique focused cases pass; the initial zero-affine autodiff failure is preserved and fixed, and the final modified credit case was freshly compiled and passed. The release driver and formatting/claim-wording/diff checks pass. All four actual saved-parent construction checks exit zero with no updates/one backward batch, identical parent parameter bits and no measured hard-choice differences on64 original/stress rows. Paired draws/RNG/teacher masks/query streams match; allocated weights differ. The evidence receipt binds actual sources, executable and reports.

Four640-update continuation fits remain NOT_RUN at this source checkpoint. This is a joint context/value credit-placement experiment, not isolated payload causality or complete integer serving. NoRead/trunk/output remain floating; general language and energy are unqualified. A separate retained-output/source audit identifies the history-dependent NoRead boundary for subsequent native work. Claude's session lane is unchanged.

References #1512, #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5949915316 — 2026-10-02T10:05:05Z — Casey-allard

Exact-head independent source/execution review APPROVED `6d48b350992a6944b95b32cc561b52996b3be5cc`.

Final review SHA256`25a9e19df507864b042bd7e429b315ca6ddca1a3db390bfdd0143c9dd2a15b9b`, retained `geometric-query-weighted-aux-20261002/final-independent-review.md`. Reviewer independently verified source/executable/supporting hashes, all four reports/manifests and file inventories, initial/final actual-parent parameter bits, all13 positive ordinary-answer gradients, no measured hard-choice differences and paired initial draws/RNG/teacher masks/query streams. The64 checked row evaluations repeat episodes across policies; they are not64 independent quality examples.

Actual release unit/build/fmt/claim/diff checks are recorded. Initial connected-zero failure remains preserved; final modified credit case was freshly compiled and passed. No further check is requested. Fits are NOT_RUN at this source checkpoint; source approval is not a learning or complete-serving claim. Five historical CI statuses are compatibility acknowledgements, and delivery-evidence remains advisory under D19. Protected automatic merge is owner-authorized; wider1512/973 remainOPEN.



---
_Comment/review ids in this file: 5949915316_

