# #1124 — Bound quaternion-cube development fit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T19:54:46Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T19:56:02Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1124
- **Merged:** 2026-09-04T19:56:02Z
- **Merge commit:** e3084eac47b04b540ccccf54d0547e37fa885882
- **Base / Head:** main / UOR-Foundation:codex/issue-973-quaternion-cube-fit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The accepted quaternion-cube path had only a no-fit comparison against weights learned for dense SwiGLU, so #973 needed one fixed development-data dose before any width, corpus, or instruction increase. This change adds that bounded fit path and records its actual result: both authorized launches completed the full 120-token backward gate and eight optimizer updates, but the conservative update-eight projection did not admit the remaining work inside the fixed 840-second wall. The checkpoint therefore closes as `RESOURCE_UNAVAILABLE_FULL_CONTEXT_CUBE_FIT`, a resource-admission result rather than a model negative.

The runner freezes the six retained dense-SwiGLU tensors (36,864 serialized values), optimizes only the 18 cube-forward-active tensors (215,296 values), keeps the accepted sparse reader and immutable dense comparator fixed, and refuses changes to the 128-update/four-thread/840-second contract. Its successful-completion path includes exact fitted-artifact reload, frozen-tensor checks, atomic model/result publication, fixed monitor/anchor scoring, and the two accepted prompt comparisons. Neither stopped launch reached that path, so no fitted artifact was published.

Measured in each admitted launch:
- the exact 120-token recurrent forward/backward completed;
- aggregate gradients for all 18 active tensors were finite and nonzero;
- update-one loss `10.436132` and gradient norm `6.284435` reproduced to six decimals;
- the sole direct resource correction reduced elapsed time to update one from `78.177` to `25.757` seconds;
- observed peak RSS remained below 4 GiB;
- final validation and held-out stores were not opened.

The completed 128-update dose, post-fit NLL/top-1, generation, dense competitiveness, generalization, instruction following, reasoning, coding, H4 advantage, table lowering, and release behavior remain `NOT_RUN`. The roadmap now points to one implementation action: a lean differentiable training forward that omits unused attention-weight outputs and accepts precomputed metadata-only sparse selections while preserving the recurrent computation graph and inference semantics. A future relaunch remains gated by the unchanged update-eight completion projection.

Focused validation:
- `uv run python -m compileall -q src tests`
- `uv run python -m unittest tests.test_fixed_recurrent_kv_binding tests.test_quaternion_cube_fit` (5 tests)
- CLI help for `fit-quaternion-cube-r4-language-path`
- `python3 scripts/check_claim_wording.py`
- `python3 scripts/check_agent_execution_policy.py`
- `git diff --cached --check`

Three independent reviews approved the code, evidence boundary, preserved stop-record hashes, and final delivery wording. No third fit or timing launch was performed.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

