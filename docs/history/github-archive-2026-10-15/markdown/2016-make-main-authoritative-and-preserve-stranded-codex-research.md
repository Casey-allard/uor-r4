# #2016 — Make main authoritative and preserve stranded Codex research source

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:10:01Z
- **Updated:** 2026-10-09T13:27:00Z
- **Closed:** 2026-10-09T13:12:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2016
- **Merged:** 2026-10-09T13:12:45Z
- **Merge commit:** 20a62312f7211a73a8a2e4c78fd5e400fa0b2529
- **Base / Head:** main / UOR-Foundation:codex/main-consolidation-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction October9: origin/main is authoritative for all participants. Remove mandatory isolated worktrees and require protected merge verification plus preservation/cleanup before any successor. Inventory all577remoteCodex and620localCodex branches, retain exact original histories in a restorable source bundle tracked on main, and identify unfinished implementation without replacing current runtime with historical snapshots. The queue forces SQUASH and this account cannot administer it; the tracked bundle preserves source through delivery. Reconciles Claude consolidation2015. Live lab-state and foreign active work are excluded from retirement. Actual validation:1197headbindings retained by source archive ancestry; thin-pack strict object validation against main prerequisites PASS; active crates/scripts unchanged from refreshed main; policyJSON parses, claim-wording and whitespace PASS. Missing runtime integration is explicit:59distinct absentRustpaths, including pendingCUDA1841 and failedcoupledexport. Retention is not model promotion. Cleanup follows actual merge and fresh-main archive/source verification. References #820.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081636064 — 2026-10-09T13:15:41Z — Casey-allard

Delivered actualmerge20a62312f7211a73a8a2e4c78fd5e400fa0b2529 verified on freshorigin/main; all17delivered fileblobs match reviewedhead63855169a. Independent exacthead source/docreview PASS:1197bindings/659uniquecommitIDs all retained under bundlehead;514prerequisites ancestralto mainbase; bundlebytes/SHA match. Originalsrcpack strict validation PASS, source preservation does not promote unfinished implementations. Fivecleanownedworktrees now removed after verification. Dirtytails are separately pending2018 and remain local until actualmerge.


---
_Comment/review ids in this file: 6081636064_

