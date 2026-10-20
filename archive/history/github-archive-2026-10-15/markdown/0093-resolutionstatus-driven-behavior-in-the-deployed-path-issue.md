# #93 — ResolutionStatus-driven behavior in the deployed path (issue #78)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T05:39:49Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-07-23T05:46:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/93
- **Merged:** 2026-07-23T05:46:46Z
- **Merge commit:** 7c7b8624641d4a0c2c931964d2883036d8e3e529
- **Base / Head:** main / UOR-Foundation:issue-78-status-policy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Wires the D4 manifest policy into the deployed adapter: unsupported inputs are detected and handled explicitly instead of being silently guessed at.

- StatusPolicy as data: ExactContext → Serve, Graph → Serve, Novel → WidenOnce (retry once at 2×TOP_M) then Abstain, Contradictory → Abstain (arm ready); optional config.status_policy override in score_report.json with lenient per-row fallback
- Allocation-free step layer (score_step + StepState): Rule 1/2 semantics recomputed with zero allocations — census shows 0 alloc/0 B across warm-up, predict, abstain, generate, and 8 repeated OOD probes; integer-only source scan on the policy block
- Widen-once is per distinct signature (bounded NovelSeen FIFO, 1024 sigs) — sound because scoring is a pure function of the immutable artifact
- API: POST /api/r4g1/predict + /api/r4g1/generate; responses carry status/widened/abstained; chat abstains with HTTP 200 + generation_mode r4g1-abstained (never silent fall-through); the pathological-output guard stays the last line for served output; typed out-of-vocab error replaces a panic vector
- Probe suite (11/11): OOD → widen → abstain with counters, covered → serve, identical Novel probe → widen skipped (memory), 16 repeats deterministic, step↔reference parity at both widths

Verified by maintainer agent: 49 suites green, clippy -D warnings clean, cargo fmt --check clean, κ-reproduction green (real checkpoint, 24.5s). Live endpoint examples in the issue thread.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

