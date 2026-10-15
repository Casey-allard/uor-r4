# #1816 — Record actual checkpoint entry credit and next learning discriminator

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T00:55:16Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-10-07T00:56:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1816
- **Merged:** 2026-10-07T00:56:35Z
- **Merge commit:** 3e1952d9fc7a230d531829d1cec298f14c6b140c
- **Base / Head:** main / UOR-Foundation:codex/entry-credit-result-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Record the measured step-128 entry-credit diagnostic: eight fixed construction rows pass independent integer forward parity, with zero updates and unchanged masters/packets. The old loss already has nonzero potential gradients, so the result supports an alternative local credit path rather than a total learning disconnect or predictive improvement.

Replace the completed diagnostic as the next task with a comparison of old and raw-score token-alias adjoints against the margin, controlling direction scale before admitting a fit. Preserve geometry, negative evidence, alias-mass and Copy-retention limitations. References #820 and #1815.

Documentation-only validation: actual checkpoint report and adversarial review inspected; claim-wording and git diff checks PASS. Cargo/GPU checks are NOT_APPLICABLE to this doc change; #1815 owns the executed checks.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6028502145 — 2026-10-07T00:55:42Z — Casey-allard

Exact-head self-review: 0adc6054e86563edf650463a1c9e2133adda8313. Only current-state documentation changes; branch is based directly on merged main 27c9e84a.

I independently inspected the eight-row source-bound report and its completed GPU log, confirmed zero updates and unchanged masters, and retained the adversarial qualification: nonzero old parameter gradients mean no total disconnection claim; projected surrogate descent and larger norms are not exported/native improvement. The next comparison controls direction scale and keeps alias mass and later Copy retention explicit.

Claim-wording PASS; git diff --check PASS. Cargo, CUDA and Metal execution are NOT_APPLICABLE for this doc PR; measured Rust checks are owned by #1815 at its tested head, not reattributed to this documentation head.



---
_Comment/review ids in this file: 6028502145_

