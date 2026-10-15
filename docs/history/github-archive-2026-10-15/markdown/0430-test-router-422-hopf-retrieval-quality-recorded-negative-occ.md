# #430 — test(router): #422 — Hopf retrieval quality: recorded NEGATIVE (occupancy spread without retrieval value)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T04:56:57Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-08-05T05:50:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/430
- **Merged:** 2026-08-05T05:50:33Z
- **Merge commit:** da16916ce5c4d9678980ace1670f0a73bea148db
- **Base / Head:** main / UOR-Foundation:i422-hopf
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #422 #306 #421. The measurement #306 conditioned on, finally run — and the condition FAILS. Shipped post-#306 signed projection: sector-filtered MRR **0.0045**; pre-#306 magnitude shim (verbatim reproduction): **0.0743**; shuffled control 0.0007. Exit rule needed A ≥ B+0.020 — measured **−0.0699** → recorded NEGATIVE. The remediation's 6.1× occupancy spread replicates at scale (A 456/512 sectors, max load 398 vs B 16/512, max load 9,510) but the spread carries NO retrieval value — sector agreement under A correlates with WORSE base MRR (0.0918 agree vs 0.2380 disagree). #306's 'improvement' claim is refuted on the deferred criterion; sector transport as shipped is not the router-quality lever — the projection needs content-aligned redesign, which is now #422's real scope (Ari: this is the interesting part).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

