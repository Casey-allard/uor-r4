# #1123 — Implement finite-indexed R4 nonlinear block

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T19:16:21Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T19:20:00Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1123
- **Merged:** 2026-09-04T19:20:00Z
- **Merge commit:** 0a57df305549bab12d2efba589600693982bb6d7
- **Base / Head:** main / UOR-Foundation:codex/issue-973-nonlinear-geometric
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The bounded sparse recurrent language path still executed its dense SwiGLU block, so the nonlinear geometric stage had no concrete implementation. This change adds `R4H4FrameQuaternionCubeResidualV1`: twelve ordered R4 cells execute a parameter-free quaternion cube in the current H4 frame after the accepted sparse attention residual, while the accepted learned artifact and dense-SwiGLU arm remain available as the matched comparator. The project track now advances to a separately bounded development-data fit of this exact architecture.

The fixed no-fit comparison completed on both existing development prompts. Across 53 processed tokens, the candidate executed 1,272 R4 blocks, zero dense-MLP calls, and zero dense-MLP learned-weight products; recurrent storage remained 2,304 f32 values, the read ceiling remained nine sources, and all recorded causal/provenance prohibitions remained zero. The maximum observed f32 block-norm error was `7.152557373046875e-07`, and the maximum residual-bound ratio was `1.0`. Both generated continuations diverged from the fitted dense comparator at the first token and were visibly degraded, so this is a mechanical execution result only: trainability, useful language, H4 benefit, reasoning, coding, table-native lowering, and release behavior remain unverified.

Independent review checked the quaternion law, frame orientation, recurrent seam, audit arithmetic, raw hashes, and evidence boundaries. It corrected the operator-bank description to 120 frame indices with at most 60 distinct maps and narrowed the dense-weight claim to the predictive forward path; all pre-correction raw artifacts remain preserved.

Validation:

- `PYTHONPATH=tools/r4-softmax-trainer/src .../python -m unittest tools/r4-softmax-trainer/tests/test_fixed_recurrent_kv_binding.py -v` (4 tests, pass)
- Python byte-compilation of the four changed/added implementation modules and focused test
- `python3 scripts/check_agent_execution_policy.py`
- `python3 scripts/check_claim_wording.py`
- `python3 -m json.tool docs/integration/agent-execution-policy.json`
- `git diff --check`
- raw artifact SHA-256 and recursive metadata-only correction checks

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

