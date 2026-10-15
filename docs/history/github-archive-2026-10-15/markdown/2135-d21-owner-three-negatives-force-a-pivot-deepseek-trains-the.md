# #2135 — D21 (owner): three negatives force a pivot; DeepSeek trains the pointer fix, Codex stops the constraint line (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T03:15:27Z
- **Updated:** 2026-10-10T03:17:29Z
- **Closed:** 2026-10-10T03:16:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2135
- **Merged:** 2026-10-10T03:16:49Z
- **Merge commit:** c1584d50ae212d14cb9f2be5a865d56a29aaa357
- **Base / Head:** main / UOR-Foundation:claude-d21-pivot-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner decision, 9 October, after a review of the last eight hours of lab work:
- 33 DeepSeek PRs and 15 Codex PRs merged.
- Neither M1 (43/232 reply panel, 10/40 v5 memory) nor M2 (8/512) moved.

**What this PR adds**
- `DECISIONS.md` **D21**:
  - **§1, the three-negatives rule (all labs):** after three PRs on one line without a headline change, the next action is a pivot card. It either stops the line or makes one decisive, model-changing run scored on the acceptance panel. Diagnosis PRs count toward the three and never reset it.
  - **§2, DeepSeek:** the next M1 piece is the supervised pointer-target fine-tune (`ReadSupervisionGroup` bound/competing + `gate_supervised_loss`, per #2129). It trains on generated dialogues, never the frozen v5 rows, is scored on the frozen panels, and lands as one PR.
  - **§3, Codex:** archive the protected/discrete-constructor line (#2079…#2132). The next M2 piece aims directly at 8/512.
- **`AGENTS.md`:** a short Progress-control paragraph, which every lab inherits.
- **Records:** `docs/labs/lab-pivot-2026-10-09/README.md` (the review, PR by PR) and a current-state entry.

No acceptance criterion, serving contract or cap changes.

References #2028, #2029, #2030

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093206941 — 2026-10-10T03:15:35Z — Casey-allard

Exact-head self-review at `8dd02d783232750e99d9a000b016bc9c8a400547`: docs only; origin/main merged in before push (no conflict); claim-wording gate passes; every PR number cited in the record was checked against its title/result section.


---
_Comment/review ids in this file: 6093206941_

