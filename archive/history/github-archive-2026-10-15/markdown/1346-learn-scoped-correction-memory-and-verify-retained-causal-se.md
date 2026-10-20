# #1346 — Learn scoped correction memory and verify retained causal sessions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-22T06:36:22Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-22T13:40:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1346
- **Merged:** 2026-09-22T13:40:53Z
- **Merge commit:** b5d36c9b8318e4fe2084ef49cc75cff55c1b2b6f
- **Base / Head:** main / UOR-Foundation:codex/scoped-correction-memory
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, #1139, #962, #963, #964; programme #820.

This adds learned statement and temporal-query intent to ordinary text binding, with exact scoped/versioned storage and owned dependent answers. A correction changes later answers without changing an already pinned answer. Parse confidence no longer determines which committed fact is current.

## Principal review and repairs

Submitted source `c4bd440a` retains useful authored-task behavior but had a repeated-eviction bug shared with its oracle, insufficient store/session validation, a child reload test that bypassed learned language intent, and an unpinned example that did not establish a fractured committed view. Corrected source `38bf325f` repairs these before reuse.

- Capacity expires the newly out-of-window revision and releases its token payload. Independent ordinal-retention expectations and repeated-overflow tests cover the original defect.
- Version2 store/session formats validate exact IDs, predecessor/chain membership, immutable visible-history and captured-payload witnesses, emitted prefix/EOS and action phase. Old bytes/executables remain preserved; intent format stays version1.
- The child process now interprets raw language, performs a learned correction after restart and resumes all11 saved answer phases across it. Explicit API history queries are reported separately.
- The pin intervention now produces an unpinned old-link/new-value answer absent from every intervening committed view. Three added historical-dependent questions distinguish first-hop history from downstream Current at the pinned view.

## Retained result and precise scope

| Primary panel | Language questions | Explicit API queries | Expected-outcome agreement |
| --- | ---: | ---: | ---: |
| Development |14|2|16/16|
| Exposed regression |10|1|11/11|
| Final lexical population |10|1|11/11|

The38/38 includes36 Complete, one NoHistory and one Unresolved. It is not38 emitted language answers. The final population first appears in submitted root13; root14 and principal executions replay it. Whole entity/value strings are disjoint from fitting, but interaction structure and authored forms are shared with exposed data. Broad syntax, multiword-name transfer, general language and general reasoning remain unqualified.

All511 original row behaviors and both learned artifact files are unchanged. Corrected reports contain519 rows, adding5 assertions and3 historical-dependent questions. Controls retain NoRead0/38, UpdateDisabled1/38, Unscoped33/38 and ParseScoreAuthority19/38. Reduced-capacity comparisons now pair the same16 development queries (two actual output changes), rather than comparing16 to38.

## Executed validation

- 17 focused module tests and25 runner tests pass; one legacy artifact-dependent test is explicitly ignored.
- Offline release build and corrected loaded-model execution pass; original and repaired executables retained separately.
- Independent raw-text oracle reconstruction, source/executable hashes and ten-file report seals verified; zero unlisted files.
- All11 resumed phases retain the pinned answer; a new raw question sees the post-restart correction.
- Format, claim-wording, JSON and changed-document link checks pass. Existing compiler warnings remain; no warning-free claim.

The first principal execution caught an EOS vocabulary guard introduced during repair. Protocol EOS can be outside lexical vocabulary; source38bf fixes it with a regression test. Failed attempt preserved. Principal root3 corrects root2's caller-reported dirty flag for pending documentation edits, with identical executable/parameters/rows; it is an exposed metadata replay.

Capacity bounds token payload retention per address, not total history/lexical metadata. History hashing currently scans/sorts/serializes. State witnesses establish internal consistency, not external authenticity. Whole-path D0-b and energy remain unqualified. Scope is host-authenticated; no geometric component explains this scoped-memory quality result.

## Next integration

Consume one owned Q8 result through **learned observed request → scoped Read → Apply → result-conditioned Read → complete Emit**, retaining corrections/history/restart. Ground lexical operands and operations explicitly; compare to a competent finite transition control. Signed order (`ij=k`, `ji=-k`) gives a concrete information-loss test. The owner's topological-folding intuition is retained as a projection/phase hypothesis, not a modified Euler identity or physical quantum claim. Broader E/S prose/conversation and executed Rust follow.

[Principal review](https://github.com/UOR-Foundation/uor-r4/blob/45b453e615d74d59dbaf1370d2eca87edc1c1761/docs/integration/scoped-memory-review-2026-09-22.md) · [Comprehensive DeepSeek brief](https://github.com/UOR-Foundation/uor-r4/blob/45b453e615d74d59dbaf1370d2eca87edc1c1761/docs/integration/deepseek-consumed-geometric-state-step-2026-09-22.md) · [Checks/resources](https://github.com/UOR-Foundation/uor-r4/blob/45b453e615d74d59dbaf1370d2eca87edc1c1761/docs/evidence/scoped-memory-principal-checks-2026-09-22.json)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5777619540 — 2026-09-22T13:42:52Z — Casey-allard

Protected delivery complete: PR #1346 merged as `b5d36c9b8318e4fe2084ef49cc75cff55c1b2b6f`. Full merged tree equals reviewed `45b453e615d74d59dbaf1370d2eca87edc1c1761`; all23 changed paths verified. The independent principal review repairs retention, restored state, raw-language restart and the fractured-read instrument.17 module and25 runner tests pass (1legacyignored), release build and actual corrected exposed execution pass. All511 original behaviors and both learned model files retained;264 query expectations independently reconstructed with zero errors.

Primary38/38 is34language+4API expected outcomes, including36Complete and two expected nonComplete terminals. The final population is lexical transfer on exposed structure, not broad structural/syntactic generalization.11 saved phases restore across a learned correction; new raw questions observe the update. Total history metadata and whole-path D0-b/energy remain unqualified.

Next: learned observed request and source select a shared Q8 computation whose grounded result changes a later exact read and complete answer in the same session. Signed order gives a concrete test of retained orientation; the owner's topological-folding idea is recorded as a projection/lift hypothesis, not an altered Euler identity. Then E/S prose/conversation and executedRust.

[Principal review](https://github.com/UOR-Foundation/uor-r4/blob/b5d36c9b8318e4fe2084ef49cc75cff55c1b2b6f/docs/integration/scoped-memory-review-2026-09-22.md) · [Full DeepSeek brief](https://github.com/UOR-Foundation/uor-r4/blob/b5d36c9b8318e4fe2084ef49cc75cff55c1b2b6f/docs/integration/deepseek-consumed-geometric-state-step-2026-09-22.md)

Six owning issues are current/open; knowledge and memory include verified delivery. Free space38,023,065,600B;2,568,081,408allocatedB inactive compiler outputs cleared, unique research andDownloads preserved. Ledger329,315,344/337,300,000ms, necessary extension recorded prospectively, no paidcompute. Owner checkout unchanged andclean.



---
_Comment/review ids in this file: 5777619540_

