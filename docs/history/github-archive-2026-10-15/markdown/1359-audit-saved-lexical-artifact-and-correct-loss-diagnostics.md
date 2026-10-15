# #1359 — Audit saved lexical artifact and correct loss diagnostics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T06:01:18Z
- **Updated:** 2026-10-09T13:17:45Z
- **Closed:** 2026-09-23T06:02:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1359
- **Merged:** 2026-09-23T06:02:31Z
- **Merge commit:** 01609446f16f285dc8c9246801f975c05d0ee4ee
- **Base / Head:** main / UOR-Foundation:codex/ordinary-lexical-audit-principal
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The saved ordinary/grounded Generate/Copy/Stop artifact now replays every reported panel after fresh-process deserialization. The audit also isolates one post-Copy top-1 decision: changing only the copied-token feedback leaves it unchanged, while changing the selected-source fingerprint changes it. The original 2/2 result therefore does not establish learned emitted-token feedback.

On the same open-development prose targets, a prose-only full-context recurrence fit scores 6.2039 bits/target; adding a direct two-token channel worsens it to 6.6732. This is a diagnostic, not a replacement artifact or a retirement of the channel family. Principal review repaired two reporting errors before delivery: the sealed channel receipt's negative interval was mislabeled (correct two-token minus recurrence-only +0.469, paired interval [+0.410,+0.527]), and the previous “token-conditional” score still normalized over Stop. A new sealed `olx-audit-5` replay measures true Generate-conditional loss 7.371855 versus Generate-action loss 7.372198, so Stop normalization is negligible on this panel. Original seals remain immutable and the errata are explicit.

The public README, current state, canonical plan, evidence JSON, resource ledger and Kimi handoff instructions now point to the measured next dependency: test the better per-position/full-context formulation in the same grounded learner while preserving Copy/Stop and checking loaded generation. The owner checkout's cumulative ledger was reconciled once from 380,915,640 to the already documented 386,415,640 ms; no second 5,500,000-ms charge was created.

Validation: `cargo fmt --all --check`, offline release build of `ordinary-lexical`, actual fresh-process `olx-audit-5` replay and seal verification (zero unlisted files), JSON parse, `git diff --check`, and `python3 scripts/check_claim_wording.py`. No broad training, fresh language qualification, energy measurement, or paid compute. References #820 and #973; their full acceptance remains open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

