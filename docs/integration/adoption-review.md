# #1081 independent adoption review

Reviewed source in `/Users/casey.allard/.codex/worktrees/r4-productization-plan/uor-r4` on 2026-09-03. Scope: planning changes, current native ownership, continuation prompt, knowledge package/import boundaries, and the new seed/sync/storage scripts. No model execution, ingestion, tests, installs, GitHub mutations or repository edits were performed by this reviewer.

**Final source-review verdict: GO.** The coordinator corrected the concrete findings below; the corrected source and adoption metadata were independently inspected. The coordinator owns actual sync/schema/source-link execution evidence and protected delivery. No additional tests or review expansion are requested.

## Findings

| Finding | Consequence | Review status |
|---|---|---|
| Sync accepted arbitrary authenticated repositories but hard-coded public visibility. | Private issue content could enter default public retrieval. | **Corrected and source-reviewed:** sync now checks `isPrivate` and exact repository identity before writing a snapshot/import. |
| Sync omitted the claim ledger, adopted issues and current planning mirrors. | The promised post-delivery knowledge refresh would leave claims/current planning stale. | **Corrected and source-reviewed:** named public paths plus typed claim/citation records are now included; escaping document symlinks are rejected. |
| Seed and sync use different `basis` strings for the same immutable claim `cites` edge. | Sync of an unchanged seeded ledger fails the entire transaction on an immutable edge collision. | **Corrected and source-reviewed:** sync now uses the seed's exact claim-edge basis. |
| Seed appends `#L…` to existing `#issuecomment-…` URLs. | Source references acquire a second fragment and lose the intended comment anchor. | **Corrected and source-reviewed:** seed and sync preserve existing fragments. |

Adoption finalization is present: `adopted-issues.json` now includes collection time, states, assignees and blockers; the landing page links adoption/map/continuation first; and `workflow-adoption.md` records native changes without claiming an eventual merge has already happened. Its newly recorded #1090→#965 release dependency was flagged for the same explicit mention beside #940 in the current map. This is a small map-completeness update, not a source blocker. The coordinator's full-sync result remains execution evidence separate from this source verdict.

## Boundaries reviewed without a remaining source defect

- The knowledge package exposes only four read-only MCP tools. Reads use read-only SQLite, scoped source/edge filters and bounded results. Private scope is correctly documented as a local filter rather than multi-user authentication.
- Imports validate field types, source digests, immutable item/edge identities, endpoint visibility and graph integrity before a single transactional commit. Failed imports do not partially change the index. The CLI is the explicit write boundary.
- The public catalog contains 552 entries with no explicitly private rows. Restricted inventory and the selected project-history documents are written only to private seed records; provider-history import requires the explicit path option. No code execution or network publication occurs in the seed builder.
- Current map/issue ownership preserves #1082 as the construction-only successor and #954 as blocked. Later specification issues do not silently authorize fitting, native export, lowering or broad capability claims. The #973→#954 handoff distinguishes the supplied-clause dense reference from unmet higher-context/final-serving requirements.
- The continuation prompt selects one eligible task, refreshes native/source authority, preserves unrelated and sealed artifacts, uses only named checks, follows the protected delivery path and stops after that task. It does not trigger roadmap-wide experiments or automatic cleanup.
- Runtime guidance now compares only scientifically eligible measured plans and reuses unchanged evidence. Four threads, processes and concurrent arms are explicitly distinct. Historical four-worker evidence and numerical results were not rewritten.
- Storage inventory measures metadata/allocated sizes, reports permission/time failures and overlapping/APFS limitations, and never deletes or opens model contents.

The plan's installation/ecosystem measurements remain supported by their owning audit records. This review does not recertify external tools or previously qualified model mechanics.
