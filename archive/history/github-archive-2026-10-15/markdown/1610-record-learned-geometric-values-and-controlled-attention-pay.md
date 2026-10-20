# #1610 — Record learned geometric values and controlled attention payload diagnosis

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T09:17:37Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-10-02T09:22:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1610
- **Merged:** 2026-10-02T09:22:53Z
- **Merge commit:** ef3d0b1f4a429ceb699666cedc7a24a461a8642b
- **Base / Head:** main / UOR-Foundation:codex/geometric-value-fit-results-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Both fixed joint fits now learn token/state/span-conditioned geometric values, but they do not retain every donor answer. A no-fit payload substitution holds the newly learned query mixtures exactly fixed and rescues 65 of 106 learned failures with both donor and K2 values. This localizes a substantial subset of errors to learned-value/frozen-output compatibility and supports a targeted credit-allocation comparison while retaining the mechanism and all prior rows.

This PR records the completed two-seed 640-update results, artifact/source bindings, independently reloaded integer producers, controlled substitutions and the corresponding next step in the canonical plan. Learned float/compiled answers agree at 78/84 and 122/122 across original/stress panels of 128 rows each. All measured hard states/actions/addresses/packets/Q16/answers agree; logits are not bit-identical. Both substitutions reach 128/128 on both seed-two panels. Learned seed-one successes absent from K2 are retained.

Validation: actual release fits and saved-artifact replays exited zero; independent artifact/row/arithmetic audits are bound in the receipt. Claim-wording and diff checks pass. This results-only PR changes no Rust source and requires no repeated model run. Exact-head review is recorded separately.

The evidence is authored development grammar, not general language, geometric superiority, arbitrary donor conversion, full integer serving or energy qualification. NoRead, trunk and output remain floating. Query-read-weighted auxiliary credit is proposed, not executed; missing optimizer moments require a declared shared restart. Claude's D19/session work remains concurrent and separate.

References #1512, #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5949087101 — 2026-10-02T09:21:58Z — Casey-allard

Exact-head independent results review: APPROVED `550c97d175aeb81f2e4d17b56a080737fa7589e8`.

The nonauthor mathematician independently verified report/manifest/source/trained-artifact/row hashes and recounted all512 learned rows joined to controls. Both substitutions rescue65/106 learned failures; all reverse K2 cases6+4 are retained. Claims remain limited to authored development and learned-value/frozen-output compatibility. Query-weighted teaching remains a hypothesis, not an executed result.

Final correction only aligns wrapper-time rounding with actual execution receipts. Review SHA256 `b357d50c0fb832e4d291d19a276d4ff3cbd618e71ead891e5df8bfa62ccda635`, retained in `geometric-value-joint-fit-20261002/results-independent-review.md`. Claim-wording and diff checks pass. Rust source is unchanged; actual fits/replays and component checks are bound in the receipt rather than repeated. Five historical CI statuses are compatibility acknowledgements; delivery-evidence status remains advisory under D19, and is not substituted for execution validation.

Protected automatic merge authorized by the owner; #1512/#973 remain open.



---
_Comment/review ids in this file: 5949087101_

