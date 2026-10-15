# #1471 — Apply #1465's non-author review: owner ratification, Codex-lab status, role lines, §2b labels; record the charter

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T20:40:34Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-28T20:59:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1471
- **Merged:** 2026-09-28T20:59:05Z
- **Merge commit:** 3337cd7b8d31a2209cccf2755e5ab9fb9de56e47
- **Base / Head:** main / UOR-Foundation:lab/claude/policy-review-fixes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1 (Claude main). Class C** (policy). References #820 and #973.

This follows up #1465, which merged before its [non-author review](https://github.com/UOR-Foundation/uor-r4/pull/1465) fixes. It applies findings 1–15. Finding 9 went into #1470, which edits the same row.

**Blocking fixes:**
1. **Owner authority.** The owner row now carries strategic authority and ratification of every `DECISIONS.md` entry, the organization, lanes and merge criteria. A class C change to `DECISIONS.md`, D11, the stable invariants, or the organization and merge criteria **needs the owner's recorded ratification**. Lab 2's row and the JSON match.
2. **Codex lab.** Live documents that described an active Codex lab now carry a "historical since 2026-09-28" banner: `README.md`, `project-track.md`, `current-state.md` and `.codex-lab/README.md`. Other fixes:
   - the ledger line (ROADMAP §6);
   - `AGENTS.md`: "covers" becomes "covered";
   - the Codex lab is described as "not part of the three-lab organization; no assignments; returns only by owner direction".

   `.kimi-code/` is Lab 2's own team configuration, so its Codex lines are assigned to Lab 2.
3. **ROADMAP role lines:** I2, I3, rulings 4 and 6, the stack table row, T1's next step and the §4.3 heading now match the decisions, and the tokenizer line reads "decided in §2b".

**Other fixes:**
- **§2b labels.** The tokenizer and memory rows are labelled as the owner's charter, with Lab 1's choice of #1017 marked separately. The codec row separates the owner's 17:00 decision from Lab 1's `MapCodec`.
- **D6** is marked delivered (#1464).
- **A "Checkpoint (I1)" row** is added.
- **The cloud-obligation row** is corrected against HANDOFF.md.
- **Cycle-5 disposition** is corrected.
- **Class B wording:** results are re-read by Lab 1, and Lab 1's own results by another lab or an independent pass. Promotion needs Lab 1's decision.
- **Precedence** yields to a newer owner instruction.
- **The merge check** becomes "the squash diff equals the PR diff".
- **Owned paths** are defined.
- **The startup reading order** is aligned with `AGENTS.md`.
- **The charter and the owner's merge directive** are recorded verbatim in `docs/integration/three-lab-charter-2026-09-28.md`.

**Checks:** `r4-policy-check` passes, the JSON is valid, and the claim-wording check passes.

**Needs:**
- the original reviewer's re-read of the blocking fixes;
- **the owner's ratification**, because it changes the organization and merge-criteria section.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5878401217 — 2026-09-28T20:57:13Z — Casey-allard

_edited 2026-09-28T20:57:25Z_

**Owner ratification (recorded by Lab 1).** The owner ratified this PR as written in the Lab 1 session on 2026-09-28 ("Ratify as written"). The re-read's remaining edits R1–R5 and its nits are applied verbatim in `7076d31c`. `main` was merged in after #1470, and the §2b conflict was resolved so that #1470's pins and #1471's labels both survive. Merging.


---
_Comment/review ids in this file: 5878401217_

