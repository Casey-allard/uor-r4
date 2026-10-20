# #1027 — C1-SB4 joint-candidate margin falsifier

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T01:17:42Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-09-01T01:18:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1027
- **Merged:** 2026-09-01T01:18:47Z
- **Merge commit:** 59a667f325d10636ae1caef5a007e94c44afd44b
- **Base / Head:** main / UOR-Foundation:issue-954-joint-candidate-margin
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- implement the independently frozen C1-SB4 full-source candidate-set structured-margin preflight
- record the sole bounded MPS result and its content-addressed evidence
- update the current programme documents to the binding negative result and paired-query conditional-binding successor

## Result

Terminal: `FAIL_JOINT_CANDIDATE_MARGIN_PREFLIGHT`

- fit exact records: `70/126`
- sealed exact records: `35/63`
- all `24/24` targeted attention tensors changed; no non-attention tensors changed
- same-source query relocation was not exact
- Rust parity, full fit, development, checkpoint emission, and product reveal remain `NOT_RUN`
- no retry is authorized by the frozen contract

A question-ignoring affirmative-locative rule reproduces every published aggregate count, so the evidence does not establish subject-question binding.

## Focused verification

- 39 relevant Python unit tests passed
- Python compileall passed
- claim-wording gate passed
- `cargo fmt --check` passed
- staged diff and JSON validation passed

The broad workspace suite was intentionally not run: this is Python research tooling plus evidence/documentation, and the predeclared negative branch forbids downstream Rust implementation or product work.

References #954

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

