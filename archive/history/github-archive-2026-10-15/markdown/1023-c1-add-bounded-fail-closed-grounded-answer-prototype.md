# #1023 — C1: add bounded fail-closed grounded answer prototype

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T19:00:57Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-08-31T19:02:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1023
- **Merged:** 2026-08-31T19:02:16Z
- **Merge commit:** d5d0c84218450d94514e2bbbcd7dbc0ab690b89a
- **Base / Head:** main / UOR-Foundation:issue-954-grounded-answer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Adds the first bounded #954 source-backed correctness prototype without claiming that it passed.

- adds `r4 answer`, bound to the #954 export by default
- serves only exact source spans or typed `ABSTAIN` / `CONTRADICTION`
- binds and rechecks exact non-symlink UTF-8 source bytes
- rejects audit-output/source aliasing and lossy UTF-8 model output
- adds one fixed 384-step MPS grounding SFT path with no CUDA and no sweep
- records the observed frozen product result: `FAIL_GROUNDING_PRODUCT_TRANSFER_ABSTENTION_COLLAPSE` (`1/3`)
- updates README, research direction, and roadmap: #1019 is closed; the active #954 successor is a learned source-span pointer/copy head over the established R4/Spin states

## Actual M1 evidence

- 384/384 MPS optimizer steps in `883.7735486670863 s`
- final development answer-token loss `0.00014458666336423967`
- weights `blake3:0949d925c1189cdc784a4b93bbb9119da5c1a573fe3385c3b3c8f82a299c9439`
- Rust product inference selected Apple Accelerate CPU BLAS
- all three frozen prompts decoded `ABSTAIN`; only the unsupported prompt passed
- direct Python generation reproduced the same token sequence, ruling out a Rust/Accelerate/file-boundary mismatch for this result

## Verification

- release `r4` build with `local-inference-accelerate`: PASS
- three frozen real Rust product runs: observed and recorded (`1/3`)
- fixed MPS training run: COMPLETE
- Python syntax/data/model/MPS preflight: PASS
- Rust formatting and diff hygiene: PASS
- claim-wording gate: PASS
- broad workspace suite: delegated once to the protected merge queue

References #954

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

