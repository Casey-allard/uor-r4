# #766 — fix(#510): bring just vv-register back in sync with CI's R5 gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T16:33:22Z
- **Updated:** 2026-08-17T18:08:11Z
- **Closed:** 2026-08-17T17:49:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/766
- **Merged:** 2026-08-17T17:49:46Z
- **Merge commit:** 822123b9c10b141b627fda1da0788c56e9dc5dcf
- **Base / Head:** main / UOR-Foundation:i510-vv-register-r5
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

R5 (\`audit-limits\`) has been CI-enforced since #587/#588, but the local
\`just vv-register\` recipe still claimed it wasn't ready and never invoked
\`audit-limits\`. This let a contributor's local \`just vv-register\` pass
while the same change would fail R5 in CI.

- Add an \`audit-limits\` recipe to the Justfile.
- Wire it into \`vv-register\`: \`check-model register-bdd audit-deferral audit-limits deny\`.
- Rewrite the stale header comment that described the old (pre-R5) gate list.

## Test plan

- [x] \`just vv-register\` runs \`audit-limits\` locally and passes.
- [x] No change to CI itself -- this only brings the local recipe back in
      sync with what CI has already been enforcing since #587/#588.

🤖 Generated with local engineering work performed during a GitHub outage,
now being synced back now that GitHub is reachable again.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

