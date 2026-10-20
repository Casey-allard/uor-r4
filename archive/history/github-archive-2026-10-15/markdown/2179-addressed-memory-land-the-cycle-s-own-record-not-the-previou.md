# #2179 — Addressed memory: land the cycle's own record, not the previous one (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T21:13:53Z
- **Updated:** 2026-10-10T21:18:51Z
- **Closed:** 2026-10-10T21:16:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2179
- **Merged:** 2026-10-10T21:16:22Z
- **Merge commit:** 4d8e17ae4d678312cbc2c20b15a61eee3d9cf6c2
- **Base / Head:** main / UOR-Foundation:deepseek/mem-record-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Fixes a delivery mistake of mine:** the memory-result PR [#2177](https://github.com/UOR-Foundation/uor-r4/pull/2177) merged with the **previous** cycle's record in place of the one its title, body and validation comment describe, because my delivery script copied the round's earlier file. This restores the correct record at `docs/labs/addressed-memory-2026-10-10/README.md`: the device fix measured at **0.19 s/step against 7.4 s/step on CPU**, the two-seed run at D10's own recipe reading **19/40** and **19/40** against the brief's **27/40** bar with G1's float BPB **1.20184** against the anchor's **1.17425**, the **REJECT for this configuration with the mechanism open**, and the half-steps development reading (**22/40**, wrong-value **11**) with the step-matched control it names as the next piece.

Docs only: one file, no source, criterion or panel changes. `python3 scripts/check_claim_wording.py` passes at this head. References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

