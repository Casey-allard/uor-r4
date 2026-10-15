# #1987 — Pointer route learns to select the value span: 0.108 to 0.239, still rising (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T02:25:32Z
- **Updated:** 2026-10-09T13:21:34Z
- **Closed:** 2026-10-09T02:27:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1987
- **Merged:** 2026-10-09T02:27:13Z
- **Merge commit:** 7e38733ce1fc22fcb9a531e06cd3fdbca51a532a
- **Base / Head:** main / UOR-Foundation:deepseek/pointer-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

After four data routes, three sizes and three objective variants, the pointer route was the remaining candidate: it supplies the capacity to copy an identity rather than the incentive, and every arm before this one was pointerless. Flags were validated with a 20-step smoke test before any full run was spent. The selection rate more than doubled over the run - 0.1079 at step 1,000 to 0.2387 at step 12,207 - and was still climbing at the last measured step, about half of the measured reachability of 0.4729. That is the first mechanistic signal on binding that is still learning rather than at a ceiling. Reachability is measured: about 47 percent of the time the value span is addressable at all, so roughly half the remaining gap is selection and the rest is the addressing scheme. It is NOT a binding result: the panel read was not available when the run was read, so the emitted-value test has not been applied to this checkpoint, and that is recorded explicitly. Next: the panel and the probe seven rows with read mass and DELTA on this checkpoint; if selection keeps climbing, extend the run rather than change the design. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

