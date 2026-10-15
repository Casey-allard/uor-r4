# #2051 — Pages deploy: keep gh-pages as a single build-output commit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:00:26Z
- **Updated:** 2026-10-09T16:03:05Z
- **Closed:** 2026-10-09T16:02:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2051
- **Merged:** 2026-10-09T16:02:36Z
- **Merge commit:** a66ecef50848cc11160fc0c5a1f188355731ff98
- **Base / Head:** main / UOR-Foundation:claude/pages-single-commit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`deploy.yml` already builds the site and pushes it to `gh-pages` on every push to `main`, which is what GitHub Pages serves, so nobody needs to update that branch by hand. Its history had grown to 526 deploy commits, one per merge. This adds `single-commit: true`: each deploy replaces the branch with one fresh commit, keeping it a pure build output. `gh-pages` has no protection rules, so the force-push is allowed.

A cleaner end state would serve Pages straight from the workflow (Pages source "GitHub Actions", `actions/deploy-pages`) with no branch at all. That needs a repository admin to change the Pages source, so it isn't done here.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6084510588 — 2026-10-09T16:00:34Z — Casey-allard

Exact-head self-review at `3c0646c9937bc93972fffc2e4a40a4c1a7a4d84e`: one option added to the deploy step; YAML parses; gh-pages unprotected (no rules), so the action's force-push is allowed.


---
_Comment/review ids in this file: 6084510588_

