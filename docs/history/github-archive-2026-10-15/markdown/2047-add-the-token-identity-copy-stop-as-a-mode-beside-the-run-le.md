# #2047 — Add the token-identity copy stop as a mode beside the run-length rule (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:47:17Z
- **Updated:** 2026-10-09T15:51:29Z
- **Closed:** 2026-10-09T15:49:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2047
- **Merged:** 2026-10-09T15:49:05Z
- **Merge commit:** 645a218d7f6b9890e8c2e9dda5682c468374174f
- **Base / Head:** main / UOR-Foundation:deepseek/identity-stop
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Add the token-identity copy stop as a mode beside the run-length rule (References #820)

Measured motivation: the run-length copy stop recovers +40 cells at floor 1.0 but adds EXACTLY ZERO
at floor 0.5, and the cause was measured - the matched run lies inside the VALUE'S OWN SPAN on only 20
of 79 cells, the rest matching the PROMPT TEMPLATE, because values tokenize to ~5-7 ids. A stop keyed
on TOKEN IDENTITY against the span the pointer itself selected cannot fire on the template.

WHAT THIS ADDS, read-out only (no training path, no parameters, nothing serialized):
  - CopyStopMode { RunLength, Identity }; CopyStop gains pub mode.
  - CopyStop::new(min_span) stays the run-length rule, so Default is unchanged; CopyStop::identity(min_span)
    and CopyStop::with_identity(min_span, identity) are new.
  - CopyStopReport gains pub dropped: Option<u32>.
  - CopyStopRun::open branches on the mode: RunLength keeps source-or-source+1, while Identity opens ONLY
    at the selected source when the emitted id equals the window id there. That strict open is the
    mechanism's essential difference.
  - greedy_reply_with_copy_stop pops the dropped id, its window id and its trace step, so the reply is
    exactly the ids the pointer reproduced and trace.len() == ids.len().

TESTS, all passing (cargo test --offline -p uor-r4-training --lib copy_stop -> 9 passed, 0 failed):
six new identity tests including the one that matters - it does NOT follow the prompt template, where the
run-length rule fires on the same trace and identity never opens - plus the pre-existing three, with the
refusal test extended to refuse CopyStop::identity(1).

THE DEFAULT IS PROVEN UNCHANGED, NOT ASSERTED: a fixed workload (toy pointer, 18 histories x 3 gate
biases x {selection patterns, greedy_reply, rule-off, run-length rule, run-length trace}) was captured on
the PRE-change binary and re-run on the changed binary - 90 compared lines, 88 byte-identical, and the two
differences are only the new dropped: None field in the run-length report's Debug; removing that substring
leaves 0 of 90 differing. The pre-change literals are pinned in the regression test. Full suite: 789->795
passed with the SAME 14 pre-existing failures in identical name/order (dialogue_episodes.rs:613 overflow and
joint_campaign binding), so 0 regressions.

LIMITS, STATED: the sealed probe numbers (arm-ptr 35/84 with channels 0.5149/0.8351/0.5610/0.4901,
arm-a-w05 10/84, arm-c-ctrl 24/84) are NOT locally reproducible - the arm checkpoints and the external bind
probe live only on the pod volume - so the local evidence is the differential and suite comparison above,
not a re-measurement of the panel. Conventions chosen where the pod implementation was unreachable are
documented in-source: the span is measured against the window the run-length rule uses, emitted keeps its
meaning (the dropped id included, so the reply ends with emitted-1 ids), stopped_at is the last kept id,
and EOS/short-cycle stops still precede the copy stop. This is an offline capability: export refuses a
pointer carrying an identity term - there is no integer port - so none of it is claimed as served.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

