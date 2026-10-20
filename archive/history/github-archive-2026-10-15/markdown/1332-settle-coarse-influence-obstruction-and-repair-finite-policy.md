# #1332 — Settle coarse influence obstruction and repair finite-policy solver

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T14:11:04Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-21T14:46:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1332
- **Merged:** 2026-09-21T14:46:37Z
- **Merge commit:** e0d6296c9d787874ade8d2b705ccf6da85d6cf4b
- **Base / Head:** main / UOR-Foundation:codex/policy-feasibility-settle
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The 32-address influence policy discards information needed to preserve useful answers and abstention. This change collects counterfactual development statistics, settles that policy class with an independent integer certificate, repairs the generic optimizer, and assigns a concrete successor that preserves the retained geometric reader's learned Read–NoRead decision.

On 197 present and 23 absent development queries, the contextual parent emits 149 correct answers and reads 7 absent queries. Under the same absence budget, any current H4 table can emit at most 74 correct answers and categorical at most 71, against 145 required, even with every bucket free. This closes the unchanged coarse-policy search. It is a collision in the five-bit influence observation, not a global H4 capacity result. No fresh final evaluation or model promotion is claimed.

The principal review completes DeepSeek's submitted implementation in this PR:

- Fix fixed-bucket objective offsets and use the same tolerance-relaxed constraints throughout branch-and-bound. Previously a negative fixed cost could incorrectly prune a better feasible solution. The retained obstruction has an independent integer certificate.
- Use the actual saturated integer boost in the counterfactual loss identity; report no tune objective when no policy was selected.
- Preserve both sealed attempts and qualify incomplete source correspondence, fit identity and saved-statistics scope. The expected-manifest loader really checks bytes/config/digest/vocabulary; complete causal dependency binding remains successor work.
- Reconcile the stale live resource JSON and retrospective prior charge/extension, including its 14,500 ms rounding discrepancy. Safely remove inactive incremental compiler intermediates; preserve all unique research/artifacts.
- Update active README/roadmap/state/evidence/mechanism pointers and provide the next comprehensive execution prompt. Artifact inspection identifies an exact parent construction using its existing Read–NoRead advantage; actual prefix parity and useful text integration are next-run requirements.

[Principal review](https://github.com/UOR-Foundation/uor-r4/blob/11e0214ba3c5207c2c6aeec9a6e735f24d7b99e4/docs/integration/policy-obstruction-review-2026-09-21.md) · [Independent audit](https://github.com/UOR-Foundation/uor-r4/blob/11e0214ba3c5207c2c6aeec9a6e735f24d7b99e4/docs/evidence/policy-obstruction-principal-review-2026-09-21.json) · [Next execution prompt](https://github.com/UOR-Foundation/uor-r4/blob/11e0214ba3c5207c2c6aeec9a6e735f24d7b99e4/docs/integration/deepseek-reader-confidence-step-2026-09-21.md)

Validation executed on the corrected source:

- cargo fmt -p uor-r4-core -- --check
- cargo test -p uor-r4-core --lib policy_feasibility --offline: 5/5 pass, including independent exhaustive solver reference, fixed-cost/tolerance cases and saturation arithmetic
- cargo check -p uor-r4-core --bin competitive-reader --offline
- Claim-wording gate, JSON parsing, 907 local links and git diff --check pass
- Independent saved-count dynamic programming, artifact hashes and both eight-member report seals checked. No full model replay for the bounded principal repairs; original harness results remain their original evidence.

Principal build/check debit 272856 ms; cumulative 195884105/198900000 ms. Free space after checks 37.67 GB; cleanup recovered 4.32 GB of immediate observed free space from 5.159 GB allocated inactive incremental intermediates. The 36.766 GB reserve plus 128 MiB margin remains. No paid/external compute. Physical energy and whole-path D0-b qualification remain unavailable.

References #973, #820, #1139, #962, #963, #964. These broad issues remain open. Repository merge-group statuses are compatibility acknowledgements, not executed tests.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

