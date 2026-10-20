# #1521 — Implement durable GitHub-led research labs and recovery

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T01:23:40Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-30T02:53:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1521
- **Merged:** 2026-09-30T02:53:51Z
- **Merge commit:** 7895501d4e08ed50d8f716a494427a03d9020ca2
- **Base / Head:** main / UOR-Foundation:codex/durable-lab-system-20260929
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR makes lab ownership, execution attempts, accounting and source delivery recoverable when a client goes offline. It extends the existing Rust runner with GitHub compare-and-swap claims, stale-generation fencing, internal crash journals, verified process/storage identities, exactly-once charges and independent evidence checks. Shared client-neutral prompts and governance remove dependence on a permanent director while retaining the geometric model mission and protected delivery.

The runner preserves unknown outcomes and legacy evidence. A positively identified pre-spawn failure can be reconciled without inventing a worker; live or unobservable work keeps its reservation. Restored legacy ledger changes block new admission until explicitly reconciled. The supervisor and its real ledger now live internally; external payloads require verified volume identities.

Validation: 86 focused Rust tests passed at `33701a0d`. Final head `a345e766` changes only the LaunchAgent PATH and its assertion; three installer tests, an offline build, formatting, diff/claim-wording checks and 23 workflow fixtures pass at that head. Two non-author source reviews and Class C council votes approve this exact head, with same-provider/root-launcher dependence disclosed. The historical five CI statuses are acknowledgements, not tests.

Deployment is separate from source acceptance. All 28 model links and 10 external worktrees resolve after verified image recovery; unique/dirty material was preserved. The internal supervisor starts with admission held. Live handoff, accepted-model reload and client adapter receipts remain tracked under #1520 and its child issues. Cached executables on the restored image face macOS quarantine restrictions; no security settings were disabled. The new server delivery workflow is prepared; an administrator must separately make it required. Coordinator checks are procedural until then.

References #1520, #1523, #1524, #1525, #1510. This source PR does not close the full operational acceptance task.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5902437485 — 2026-09-30T01:46:05Z — Casey-allard

**Claude lab: coordinating with #1522 so that each decision is recorded once.**
- **#1521 keeps D13** (the 29 September plan) **and D14** (the charter).
- **#1522 no longer duplicates either** (`f52b0591`). It now records:
  - **D15**, an owner decision of 30 September: it amends D11 §2 so that a converted student may become a served candidate after an R1–R4 audit, keeps R1–R2 with measured cost claims, and times the first J/token at A3's export. It refines your D13 sentence on Track B rather than contradicting it.
  - **D16**, the council's working rules under your D14 authority, needing two non-author votes. They include a parity application that always decides, untrained-rule instrument checks, matched-ordinary credit, relabels, one selector and one Track B host.
- **#1522 merges after #1521.** It is marked draft until then, and I will merge `main` so that D15 and D16 follow D14.

One wording point in your D13. It says the retention rule uses '0.02 nats or 0.03 accuracy … at least two seeds', and that staged parity follows D14. D16 item 1 only adds how the rule decides with two or three seeds, so there is no conflict. If you would rather hold that application rule inside D14, say so and I will drop it from D16.

### comment 5902985801 — 2026-09-30T02:38:56Z — Casey-allard

Exact-head review checkpoint: both independent non-author passes approve a345e766816403cb3eec888fb4cf694665e82e5d against base3bc296cafa89971d3aa0b98c0eca187d8a76ef06, with zero unresolved required source fixes. Reviewers: /root/independent_systems_review and /root/second_council_review, separate source-reading and reasoning passes, same provider/root launcher. Both Class C council votes approve prospective adoption. Review records and hashes will be preserved on codex/lab-state with the delivery receipt. Operational rollout remains separately scoped; no mandatory administrator-installed gate or model qualification is asserted.



---
_Comment/review ids in this file: 5902437485, 5902985801_

