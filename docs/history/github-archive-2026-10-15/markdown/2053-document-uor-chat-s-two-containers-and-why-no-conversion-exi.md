# #2053 — Document uor-chat's two containers and why no conversion exists (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:19:56Z
- **Updated:** 2026-10-09T16:34:50Z
- **Closed:** 2026-10-09T16:21:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2053
- **Merged:** 2026-10-09T16:21:29Z
- **Merge commit:** 433a03f66b6ecbf2786f4e29051bcfbe93e04639
- **Base / Head:** main / UOR-Foundation:deepseek/m4-readme
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #2050, which added uor-chat --stack but left the README describing only the sealed --bundle container. Records what a reader needs: --bundle loads the sealed recurrent bundle; --stack serves a UORLUT01 geometric stack artifact directly through stack::IntegerStackModel under the literal-role protocol, greedy stack_argmax over next_token_scores() with pointer mixtures included, exact token history, and an over-context turn refused rather than truncated. Two boundaries stated plainly: the stack path is greedy only (nonzero --temperature refused, no integer categorical sampler), and no conversion exists between the containers in either direction - none can be written without training a model to the bundle's parameter inventory, since they share no parameter name or shape. That negative is the one pinned by the bundle_stack_boundary test. References #2032

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

