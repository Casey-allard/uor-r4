# #1237 — Learn bounded angular emission trees and measure transfer failure

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T02:27:02Z
- **Updated:** 2026-10-09T13:17:17Z
- **Closed:** 2026-09-13T02:28:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1237
- **Merged:** 2026-09-13T02:28:46Z
- **Merge commit:** 083cabd8a3e6c1153e50cffae8dd1c0119745731
- **Base / Head:** main / UOR-Foundation:codex/geometric-angular-tree
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The final two-cap emitter improved aggregates while losing retained predictions. This adds an experimentally isolated learned angular tree at each byte/EOS branch to test a richer decoder over fixed signed H4 state. Trees have depth at most three, up to fifteen nodes, integer leaves and construction-only split selection; a tree replaces a cap only when its branch construction loss improves. Schema 3 binds the actual source/data/configuration while preserving exact legacy artifacts and behavior.

**Actual result: FAIL_ANGULAR_TREE_DEVELOPMENT_GATE.** One fit selects 66 trees / 368 nodes. Construction improves 81→206/672 and NLL 3.354320→2.444899. Opened accuracy improves 33→57/381 but NLL worsens 3.659923→3.908623; 188 previously correct rows are lost against 0f82b728 and 181 against ade9a1cf across six panels. All four Full continuations remain incoherent; two now emit EOS. Preserve candidate 1198ef47 as negative evidence; retain 15baec48. No promotion or fresh draw.

Validation: release build and 31 focused shared-core tests; one sealed actual fit/behavior experiment; newly built actual-candidate 512-step zero-allocation and <=27 emission-comparison census; format, claim wording, links and source/evidence hashes. Both old artifacts replay 6,318 sealed rows under the new loader. Recurrence is unchanged. Twelve prior sealed roots / 102 files and all 22 parked dirty paths verify. Queue statuses are compatibility acknowledgements; these local checks carry validation.

Resource charge: 196,939/480,000 ms model/build/test; cumulative ledger 121,809,495/132,950,000 ms. A 300,000 ms local cycle increment was recorded before use within the unchanged shared ceiling. Added storage 30.9 MB within 128 MiB. No paid compute or cleanup.

Current state now recommends one bounded construction-document decoder regularization selection, fitting each fold without its validation labels and retaining both-parent controls. The pretrained parent already saw construction documents, so this is conditional decoder selection, not independent end-to-end qualification. No expanded tree or automatic campaign.

References #973 and #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

