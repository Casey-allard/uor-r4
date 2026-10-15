# #2086 — Docs and workflow: VSA hypervector figure, route-history and SpiralCore facts, N3mesis credit; no /tmp, hygiene sweeps temp

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T20:38:58Z
- **Updated:** 2026-10-09T20:40:45Z
- **Closed:** 2026-10-09T20:40:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2086
- **Merged:** 2026-10-09T20:40:12Z
- **Merge commit:** c4b2d442b011399e4a767b63ef321c7fd9fdd709
- **Base / Head:** main / UOR-Foundation:claude/docs-vsa-spiral
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Docs** (owner request, 9 October):
- **`docs/figures/geometry/vsa-hypervectors.svg`** (from `generate.py`, deterministic) plus a new section in `docs/geometry.md`, *VSA hypervectors and Hamming similarity*:
  - each token is a 4,096-bit bipolar code;
  - XOR plus popcount gives Hamming distance, which gives a bipolar cosine;
  - codes are either the historical fixed hash or the learned icosian-root assignment (trained since #2077).
- **A new section, *What carries route history (and what SpiralCore is)*:**
  - the quaternion recurrence carries route order (trained and served);
  - the Hopf holonomy tracker and Z[φ] chirality signs are line B only;
  - A1 was stopped;
  - SpiralCore is an exact finite control, not on the training or serving path;
  - the route-holonomy read and octonion-signed binding are pre-registered.
- **README:** a link to the VSA section, and the extended N3mesis acknowledgement (octonion/Fano and XOR/Hamming state sketch, ideas only).

**Workflow:**
- **`scripts/storage/uor-hygiene`:** a new stale-temp section. It moves idle /tmp items the user owns to the Trash, skipping anything open by a process, written within 24 h, `claude-*` session dirs or system names. It is a dry run by default.
- **`AGENTS.md` and `CONTRIBUTING.md`:** no project data, builds or downloads in /tmp, and no model training, fine-tuning or grading on the laptop CPU.

Checks: claim wording passes; `bash -n` is clean; the dry run lists only idle items and no session dirs; the figures regenerate identically.

References #2029 #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6088869090 — 2026-10-09T20:39:06Z — Casey-allard

Exact-head self-review at `65ab2feb0261dcbb923a7cecf4ac79d801d614a3`: geometry.md facts match the code-confirmed briefs; hygiene temp section dry-run verified (no claude-* dirs, keeps busy/recent); claim wording passes.


---
_Comment/review ids in this file: 6088869090_

