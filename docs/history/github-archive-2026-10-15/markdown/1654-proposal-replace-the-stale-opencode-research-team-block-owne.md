# #1654 — Proposal: replace the stale OpenCode research-team block (owner decides; References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T18:24:10Z
- **Updated:** 2026-10-09T13:17:06Z
- **Closed:** 2026-10-04T04:49:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1654
- **Merged:** 2026-10-04T04:49:26Z
- **Merge commit:** 8c17f8438205e55e48d02465266a04b25452bdd7
- **Base / Head:** main / UOR-Foundation:codex/ag-agents-proposal
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**Draft. Proposal only; owner decides. Docs only; `AGENTS.md` is not edited.**

Support-lab task 5. Adds `docs/labs/proposals/agents-md-research-team-block-2026-10-03.md`.

**Key finding:** the "OpenCode research team" block is **not in tracked `AGENTS.md`**. `origin/main` has no `research-kit` markers (`git log -S` finds none). It is a 13-line **uncommitted insertion** in the owner's checkout `~/uor-r4` (read-only inspection; nothing there was changed). So there is nothing on `main` to replace, and I deliberately did **not** edit `AGENTS.md`: an edit near the end of the file would conflict with that uncommitted insertion on the owner's next pull.

The proposal doc contains:
- a table of what in the block is stale (Sisyphus/OpenCode names, Kimi K3 architect, "no Claude or Codex", `.kimi-code` pointers) and what is still correct (worktree/never-edit-owner-checkout, `gh` only, no tokens);
- a harness-neutral replacement block, with owner-to-confirm items in square brackets (notably: the name/entry point of DeepSeek's own harness, which I do not know);
- optional wording fixes for two present-tense mentions in tracked `AGENTS.md` (lines 4 and 89);
- a list of OpenCode-related files and docs left for the owner to decide, with nothing moved or deleted.

Claim-wording gate passes. No cargo, no model code, no changes to `crates/`. Not merged.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5976551546 — 2026-10-04T04:25:07Z — Casey-allard

Closing and reopening only to trigger a fresh advisory delivery-evidence run on the current workflow (#1651 reports drafts as not applicable); a re-run reuses the pre-fix workflow. No content change.

### review 5404379309 — 2026-10-04T04:48:40Z — Casey-allard — COMMENTED

**Exact-head review at `1bdec19af48cf2b82ddf5b4cb7b656d2758eb5b2` (Claude lab, independent of the Antigravity author).** Docs only: `docs/labs/proposals/agents-md-research-team-block-2026-10-03.md`, a proposal record. Tracked AGENTS.md is not edited. Claim-wording passed (author), and it merges cleanly onto main.

Note for the record: on 4 October, at the owner's direction, the uncommitted OpenCode research-team block this proposal discusses was removed from the owner's checkout. It is archived as `owner-checkout-leftovers-20261004`. Tracked AGENTS.md never contained it, so this document stands as the historical proposal. **APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5976551546, 5404379309_

