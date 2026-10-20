# #1537 — fix(runner): preserve uncertain executions and use stable boot identity

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T05:11:47Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-30T07:01:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1537
- **Merged:** 2026-09-30T07:01:00Z
- **Merge commit:** d563b02f3270a2c931a7905d6624ba0beeb71d2a
- **Base / Head:** main / UOR-Foundation:codex/runner-stable-boot-recovery-20260930
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A macOS clock adjustment changed `kern.boottime` while a check job was still running. The runner interpreted that as lost process identity, wrote UNKNOWN, and moved the payload's directory before it could finish writing its result/FIFO. This change uses validated, platform-tagged boot UUIDs with host binding and keeps uncertain execution files and Child handles until a separate positive stop reconciliation.

Legacy boot-time receipts cannot authorize adoption or signals. Existing host-bound UNKNOWN receipts can be reconciled only through fresh process absence; their original outcome and charge remain immutable. Receipt replay retains uncertain jobs, reverified ownership permits a bounded stop, and supplemental accounting precedes archival.

Validation: clean head `8e90aa33`, standalone rustfmt and diff-check passed. Kimi executed the approved bounded recovery packet: `cargo test -p lab-runner --offline -- --test-threads=2` passed **89 tests, zero failures/ignored**; subsequent offline build passed. Hashed logs and two independent post-execution approvals are published under `codex/lab-state:delivery/pr-1537/8e90aa33b8ef941d3ac5de57ad448859e20ad76b.json`. Sampled total wall21.799s, peak RSS786MiB, cache growth153MiB. These are runner checks, not model evidence. Production deployment and admission release remain separate; old hold remains in place.

Focused regressions cover stable/legacy/foreign boot identities, reused supervisor rejection, live UNKNOWN path/FIFO and Child preservation, reverified stop, and receipt/debit replay before and after reconciliation. No original research or job evidence is deleted.

References #1536, #1520, #1510. Closing #1536 additionally requires protected delivery, held deployment, reconciliation of the actual interrupted Claude attempt, and a bounded continuity pilot.



## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

