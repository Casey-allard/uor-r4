# #1143 — Retain exact learned relations and record remaining write-transfer failure

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T00:10:00Z
- **Updated:** 2026-10-09T13:17:42Z
- **Closed:** 2026-09-06T00:35:10Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1143
- **Merged:** 2026-09-06T00:35:10Z
- **Merge commit:** 50caf04de078b2eb5f225936bbc08c1d8291e4c0
- **Base / Head:** main / UOR-Foundation:codex/native-relation-memory
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Persistent facts were lost when their original words left the native model's token and recent-word windows. This adds an optional Rust-trained relation writer/reader, a bounded store of sixteen exact versions, and a current-version directory. Learned assert/revise/contradict/NoWrite choices retain exact participant bytes and geometric witnesses; persistent reads feed the existing observed source commitment, copying and completion path. Snapshot validation now also handles a joint NoRead commitment correctly.

**Development delivery for #1138; its capability handoff remains unmet. Keep the relation-memory issue open; selective routing remains blocked at this version-1 result.**

- The final artifact gets 56/56 OPEN answers and exact write sequences. Reserved names give 26/28 answers versus 12/28 for the unchanged #1137 parent, but only 21/28 exact write sequences. One missing initial association also corrupts contradiction handling. All three fits and their negative results are preserved.
- All 62 older responses, 24 prior role-reader responses and eight binding outputs remain correct. Thirty-two generated Rust case sources match their previously checked bytes. An actual native CLI run matches every saved Generation field.
- Five successive session reads/revisions pass after repeated raw-window eviction, with unrelated facts retained, isolated state, exact committed-version restoration and forged-commit rejection. Historical stale-index rejection work differs after restore and is reported separately; all other checkpoint fields agree.
- New relation records occupy 168 bytes each; the store/directory is 2,840 bytes. The selected model adds 124,496 JSON bytes. Complete counters identify write-candidate scoring as the dominant new cost. No general memory, alpha, measured allocation-purity or geometric efficiency claim.

Validation: combined release CLI/example build; final test executable; fourteen copy, two role-read and two relation tests; formatting, native architecture policy, claim wording and diff checks. Broader release qualification was not run. The four historical compatibility statuses do not constitute audit/fuzz/WASM/Gate C execution.

Resource accounting includes failed checks and corrected rebuilds: 90.108/120 seconds model work, 1,090.152/1,200 seconds engineering commands; cumulative model work 1,228.950/1,800 seconds. The existing storage cap sufficed, with no deletion or paid compute. Source/artifact/report identities and complete limitations are in `docs/native_geometric_relation_memory_1138.md` and its compact evidence JSON.

The current-state, plan, continuation and research pointers keep #1138 immediate: learn a participant-independent local role/context representation while preserving the completed exact store/read path. The proposed R4/zeta role-path correction is unimplemented and requires a fresh complete build/evaluation projection.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

