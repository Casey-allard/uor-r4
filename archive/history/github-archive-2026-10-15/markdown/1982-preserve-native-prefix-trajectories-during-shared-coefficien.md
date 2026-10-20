# #1982 — Preserve native Prefix trajectories during shared coefficient learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T01:24:31Z
- **Updated:** 2026-10-09T13:17:59Z
- **Closed:** 2026-10-09T01:27:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1982
- **Merged:** 2026-10-09T01:27:05Z
- **Merge commit:** 1a5af68db44d0371912cf6023104318c2d0362f3
- **Base / Head:** main / UOR-Foundation:codex/sol-prefix-trajectory-learning-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The earlier Prefix-only pass corrected a fixed fragment but regressed complete replies. This change preserves the complete original population of 380 correct-prefix winners during each accepted update of the same 960 shared Prefix coefficients. It reuses the authenticated 18-term gradient and fixed coordinate order, with no new backward pass or serving feature.

The bounded construction accepts 351 updates and preserves all 380 guards plus 17 objective references. Its coherent export/reload corrects token267 at the intended prefix. The actual nine-row own-feedback check preserves all eight original complete replies, but task245 then chooses91 instead of307 at position5 and reaches32 tokens without EOS. The candidate remains unselected; a new full512 result is NOT_RUN.

Measured source4e9ed13a/binary65a3216f/configa4cfe1f6 binds model report77227984/sealc1309528. Independent saved arithmetic auditPASS binds resultd4de2729/seal17ef4612 and reader6b9a822a, with separate actual Rust complete-file verification. Math and adversarial result reviews pass. The repaired resource-projection and reader serialization failures remain preserved as setup faults. Complete private archivef3893b42 was freshly downloaded and SHA verified on the pod; all work stayed pod-only and foreign DeepSeek work was preserved.

Actual exact delivery-head checks PASS at cc6027fc716c322d5044c11c3f4569c9d2f6ab15: four scoped Rust paths formatted; claim wording and diff checks pass; 87 training example tests and 13 core attribution example tests pass; both offline release builds pass. Pod job20261009T012306Z completed exit0 at01:24:39UTC. GitHub compatibility acknowledgements are separate from those checks. Measured model execution is bound to4e9ed13a and is not repeated to align the later delivery commit.

References #820. After actual protected delivery, recover full witnesses for the newly reached245p5 failure before admitting another learning task.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6072357465 — 2026-10-09T01:25:36Z — Casey-allard

Exact-head review and actual validation PASS at cc6027fc716c322d5044c11c3f4569c9d2f6ab15. Parent exact-source self-review is permitted by current owner rule; independent mathematical/adversarial saved-result reviews bind b14ad003/809e8325. Actual job20261009T012306Z passes four-path scoped Rust formatting, claim wording, diffcheck, 87 training example tests +13 core attribution example tests, and both locked/offline release builds. The four model source paths remain byte-identical to measured4e9; foreignmaina0a1 adds only DeepSeek result documentation. Measured model report77227984/cheap8a4d/auditd4de remain separately bound; no model rerun at final delivery head. Five CI historical status names are compatibility acknowledgements only. Candidate remains unselected after actual245p5 desired307→91/noEOS while8/8 original replies survive. Private complete archivef389 freshly download-SHA verified; finalhead preservation in progress before queue.


---
_Comment/review ids in this file: 6072357465_

