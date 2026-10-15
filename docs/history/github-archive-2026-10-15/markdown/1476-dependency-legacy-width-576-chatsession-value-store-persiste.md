# #1476 — [Dependency] Legacy width-576 ChatSession value-store persistence

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-09-28T21:57:50Z
- **Updated:** 2026-10-09T14:43:14Z
- **Labels:** bug, roadmap:dependency
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1476
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parents #962/#1172. Scope is legacy `ChatSession` / `integer-session/1`. #1539 saved `IntegerStackSession` / `uor-r4.stack-session/1`, a different path, and does not fix this issue. Repair/qualify when this legacy path is consumed; do not block an unrelated stack or close the issue based on #1539.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

**Owner: Lab 1** (the session and the complete response path). Found by the #1474 salvage and review.

**Symptom.** A saved and restored width-576 `ChatSession` diverges from the original at its first step. This affects `uor-chat`'s restore on width-576 bundles, for example `dialogue-child-bundle-1`.

**Cause.** The session format `uor-r4.integer-session/1` never serializes the three width-576 value stores, and loading rebuilds them empty.

**Evidence.** `crates/uor-r4-integer/tests/capability_api_tests.rs`, test `restored_dialogue576_session_continues_identically`, is `#[ignore]`d with this reason. It fails when run with `--ignored` against the local fixture.

**Fix.** Extend the session format with a versioned bump that saves and restores the width-576 value stores. Decide how existing width-576 saves load: refuse them with a clear error, or migrate them. Add a save → reload → identical-tokens test on a width-576 fixture.

**Consumer:** I3 (the session) and the §8 milestone's reload-equality requirement.

</details>


## Comments (1)

### comment 6083191731 — 2026-10-09T14:43:14Z — Casey-allard

Kept open as a bug in the 9 October tracker rebuild (#2028); related milestone: #2031.


---
_Comment/review ids in this file: 6083191731_

