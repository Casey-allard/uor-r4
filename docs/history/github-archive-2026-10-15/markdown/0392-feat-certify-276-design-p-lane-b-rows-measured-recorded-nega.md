# #392 — feat(certify): #276 Design P lane-b rows — measured: recorded NEGATIVE, phase channel retired

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T20:49:24Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-03T21:05:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/392
- **Merged:** 2026-08-03T21:05:20Z
- **Merge commit:** 7c0e3dfb4f444fed7f48cc4fdd96b6fdb15d391b
- **Base / Head:** main / UOR-Foundation:i276-phase-clock
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements the #276 flag decisions and delivers the measurement the design demanded. The exit rule fires:

| row | top-1 | agreement | WB bits | keys |
|---|---:|---:|---:|---:|
| **A (shipped, same run)** | **34.7%** | **39.0%** | **8.0249** | 179,068 |
| A-P(12) | 12.5% | 14.6% | 11.4772 | 1,083,900 |
| A-P(8) | 13.6% | 15.7% | 10.9235 | 1,044,897 |
| A-P(12,F8) | 11.6% | 13.4% | 13.4134 | 671,040 |
| A-P∘G(12) composition | 16.7% | 19.2% | 9.0582 | 1,382,981 |

Every P row is below the best shipped-family row on both metrics — including the composition, where adding 4 phase digits per stage quadruples the key space and roughly halves top-1: **fragmentation dilutes support faster than the phase channel adds discrimination.** Per the design's preregistered exit rule, Design P closes as a recorded negative and the phase channel is retired from address-space candidates. The #247 session-lane use is unaffected (state channel, not an address claim). The rows stay in-tree as the reproducible record.

Bearing on #364's proposal-mechanism conclusion: this rules out the phase completion as the missing clustering signal in its simplest transport-composed form — the proposal-side search continues elsewhere.

Mechanics: analytic completion of the zeta clocks (token→prime by first-appearance, resultant phasor over the story-bounded window, b-bucket digits, 4 freqs/stage); frequencies = the 16 design-matrix window centers, exposed as `zeta_projection::window_center_gammas()`. Gates from cleaned targets incl. lib-only clippy `-D warnings`.

Closes #276. Refs #313, #243, #364.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

