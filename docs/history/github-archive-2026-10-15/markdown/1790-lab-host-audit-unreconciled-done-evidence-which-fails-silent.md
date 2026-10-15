# #1790 — lab-host: audit unreconciled DONE evidence, which fails silently

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:41:53Z
- **Updated:** 2026-10-09T13:21:26Z
- **Closed:** 2026-10-06T04:42:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1790
- **Merged:** 2026-10-06T04:42:57Z
- **Merge commit:** 0861534debdaf269027aaecd199d0ab396868066
- **Base / Head:** main / UOR-Foundation:deepseek/orphan-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds section **[7] unreconciled DONE evidence** to `tools/lab-host/host-policy-check`. Read-only; no production code.

## The gap

A job writes durable evidence under `done/<id>/exit.json`, and the coordinator is supposed to learn about it. **Nothing guarantees it does.** Only *tracked* non-finalized attempts block anything, so an evidence-complete job the coordinator never heard about is **invisible**: it doesn't block, nothing reconciles it, and nobody is told.

Measured on this host — **3 of 13 DONE dirs**:

```
claude-1541-p1-e4-a1                exit evidence present, ABSENT from coord
opencode-b0-flock-g1a2-20260930-g2  exit evidence present, coord phase=reserved
opencode-b0-flock-unit-20260930-a   exit evidence present, ABSENT from coord
```

Two were **never registered in `coord` at all**, and one belongs to **another lab** — so this is cross-lab, not a single-lab artefact.

The `reserved` one is the live blocker: it prevents `AdoptPolicy`, which prevents **every `Claim` from every lab**. The other two are the same failure mode *without* the consequence — which is exactly why they went unnoticed for days.

## What this is and isn't

This is the **detection** half. The reconciliation half — an opt-in `Action::Abandon` finalizing a `reserved` attempt whose exit evidence positively proves `never_started`, requiring the reserving session and **no** live claim — is in preparation separately, alongside a runner change so a re-evaluable admission hold self-clears instead of latching forever.

References #820, #1747.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

