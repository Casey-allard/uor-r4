# #1339 — Learn shared geometric transitions with verified finite-state controls

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T20:34:55Z
- **Updated:** 2026-10-09T13:17:56Z
- **Closed:** 2026-09-21T21:09:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1339
- **Merged:** 2026-09-21T21:09:00Z
- **Merge commit:** 85df17394d3326cb30deae1d3b1ac5b895464ced
- **Base / Head:** main / UOR-Foundation:codex/shared-transition
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native computation prototype previously lacked a reusable primitive-action state. This change learns one shared geometric action per observed primitive and retains the computed result through multi-token output, with independent artifact reload. It also adds a competent finite-state comparator and fixes the serving/evidence boundaries needed to interpret the result.

References #973, #820, #1139, #962, #963, #964. Partial component result; none closed.

## Behavior and interfaces

- `s0=E[selected payload]`, `s_j=A[observed primitive_j]*s_(j-1)`, `token=D(s_j)`. Output tokens do not become new instructions. Result state currently persists within one serving call.
- Instructions come from the declared span of the actual prefix. Absent reads never fall back to a gold operand. A supplied instruction span is not a learned parser.
- RLST v2 distinguishes `Exhausted` without a stop policy from Stop; v1 retains historical forced-exhaustion behavior. The fitted remaining-count table is explicit-program completion, not learned semantic continuation.
- The shared finite-state comparator learns initial and recurrent transitions from the same development intermediate labels and uses its own preceding prediction at serving. All learned candidates are independently exported, loaded and used.
- All-arm metric passes save actual source/payload references, emitted outputs, geometric states and terminal events. Source removal, reader disabling, changed payload, changed order and decoder-label permutation are separate executed interventions.

## Corrected exposed results

| Arm | Development /288 | Length four /128 | Reversal /64 |
| --- | ---: | ---: | ---: |
| H4 shared transitions | 186 | 48 | 10 |
| Additive C120 | 153 | 24 | 15 |
| Shared finite transitions | 288 | 124 | 64 |
| Whole-program cache | 288 | 0 | 0 |
| Value-only / Local NoRead | 4 / 0 | 0 / 0 | 0 / 0 |

H4's outputs are identical to the submitted primary on all 480 items. The finite comparator answers every source-correct item; its four length-four failures are precisely the incorrect reads. The old cache cannot complete an unseen length-four program by construction, so superiority over ordinary shared transitions is not established. Finite JSON is 1,668 bytes versus H4 RLST v2 386 bytes; this is equal supervision, not equal footprint or total cost.

All reversal sources are correct, so reader confounding does not explain the H4 reversal failure. Length four is 32 programs crossed with four values, all sharing the first two primitives; the four showcased generations use one program. Calibration/outer fitting divides values by index parity while sharing all development programs. Both halves supervise optimization; final decoder refits all development. The strongest latent order witness gives one correct complete response and a wrong reversed final answer, despite different geometric states.

## Architecture and next milestone

Development observations alone identify a closed regular Q8 action with consistent initial grounding. This supports constructive learning of geometric coordinates from the observed transition graph; it is a mathematical result, not a newly trained model. The next milestone combines this grounding with an owned resumable source/result session and the first dependent language-facing memory answer: changing the first source changes a later query, selected occurrence and complete derived answer. Signed quaternion/Hopf fiber distinctions are retained; larger representations follow an observed need. Codex remains principal reviewer; DeepSeek has substantive implementation and diagnostic autonomy with prospectively accounted necessary local extensions.

[Principal review](https://github.com/UOR-Foundation/uor-r4/blob/eda3204f4c663fd9e21b427fd3c7f408f954b480/docs/integration/shared-transition-review-2026-09-21.md) · [complete next brief](https://github.com/UOR-Foundation/uor-r4/blob/eda3204f4c663fd9e21b427fd3c7f408f954b480/docs/integration/deepseek-grounded-dependent-session-step-2026-09-21.md) · [independent corrected audit](https://github.com/UOR-Foundation/uor-r4/blob/eda3204f4c663fd9e21b427fd3c7f408f954b480/docs/evidence/shared-transition-corrected-replay-audit-2026-09-21.json) · [observed-action audit](https://github.com/UOR-Foundation/uor-r4/blob/eda3204f4c663fd9e21b427fd3c7f408f954b480/docs/evidence/shared-transition-observed-action-audit-2026-09-21.json).

## Executed validation and limits

Five focused library tests and two new runner tests pass; offline debug build, `cargo fmt --check`, claim wording, five JSON files, 968 relative document targets and diff checks pass. Unchanged runner tests were filtered, not counted as pass. Independent audit reconstructs all 2,880 arm/item records and 8,316 step/terminal events; seven-file seal, five source hashes, four artifacts and the debug executable verify. Existing unused/dead-code warnings remain. Compatibility CI acknowledgements are not tests.

One corrected exposed experiment took 49.754 seconds in debug mode. Original reports, artifacts, release executable and owner checkout are preserved. Principal debit 1,958,296 ms; live cumulative 242,357,806/243,900,000 ms, no extension needed. Free after checks 43,314,593,792 bytes; deleted zero bytes; no paid compute. Debug experiment runtime is not optimized serving performance. This finite supplied-program fixture establishes neither fresh-final/general-language qualification, a persistent source lease/dependent read, whole-path D0-b nor energy savings.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

