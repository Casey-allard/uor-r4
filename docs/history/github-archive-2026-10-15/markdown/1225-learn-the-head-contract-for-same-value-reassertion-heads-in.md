# #1225 — Learn the head contract for same-value reassertion heads in native geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-10T21:10:46Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-09-10T21:11:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1225
- **Merged:** 2026-09-10T21:11:51Z
- **Merge commit:** 18e8a9cf16cbff66eff919f411a8215a8dd33f35
- **Base / Head:** main / UOR-Foundation:codex/head-reassertion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes the disclosed, independently reproduced live-head same-value reassertion gap: a resident sequence such as `vemqi in moss harbor. vemqi now in pearl meadow. vemqi in pearl meadow.` answered `Unknown.` for initial and previous requests on `90ebd843`, `e44a9ae4` and `e3a906c8` because the frozen immediate-previous reader takes its first hop only below a revised head. The versioned chain contract gains a **head contract** (`reassertion_heads`): the first hop below a live head that is itself a resident same-value reassertion is valid (re-proven byte for byte against its predecessor); record-hop semantics are unchanged and stated explicitly (previous is the head's immediate previous record, `pearl meadow`; initial is the root, `moss harbor`); candidates reached through the head hop carry their own chain classes (six to ten, including truncation below a head), the field anchor records the head hop (`reassertion_head`) and restores only under the contract, a `HistoricalVersionIntentReassertionHeadDisabled` control withholds it, and legacy artifacts keep their exact wire form and behavior. No answer templates or oracle injection: selection remains learned over structurally admitted candidates.

Retain `fc479376`, refit from frozen parent `f0901dad` over the construction extended with head-reassertion, previous-record, head-boundary and head follow-up families: 1,250/1,250 construction targets (822 root answers, 96 previous-record targets, 162 abstentions, 182 follow-ups) with 3,351/3,351 preservation cases, complete removal reproducing the parent on all 1,250 and the head-hop ablation leaving exactly the 1,012 targets that do not need it. Before: `90ebd843` passes 1,012/1,250 of the same targets (0/120 head initial, 0/84 previous-record); the flag-only candidate `3688650a` (head contract enabled, no parameter changed) 1,020/1,250, which is why a refit was required. Per the review's process advice the actual-artifact multi-turn check ran immediately after the construction result and before any campaign: twenty conversations over 72 turns including four head-reassertion sequences pass, with head anchors restoring only under the contract. Row by row `fc479376` is identical to `90ebd843` on every prior population except the disclosed head rows (3669, 1252, 794, 890, 3789, 1372 identical; 10 head rows differ in each of 4285 and 1868). Second review's 113 scored probes: `fc479376` 113, `90ebd843` 111, `e44a9ae4` 83, `e3a906c8` 83, no correct-to-wrong change; the two changes are `known-limit/head-reassert/initial` (` Unknown.` → ` moss harbor.`) and `/previous` (` Unknown.` → ` pearl meadow.`). First review's 50 scored prompts: 50/50, identical to `90ebd843`. The separately frozen fresh draw passes 1,250/1,250 targets (96/96 previous-record, 162/162 abstentions, 182/182 follow-ups) with 934/934 preservation cases; every row differing from `90ebd843` (274) is a declared head-reassertion, previous-record, head-boundary or head follow-up row.

Validation executed locally:
- Optimized offline Rust core/API/example builds; formatting and claim-wording checks.
- Unit tests: 9 chain-traversal (head hop, eviction proof), 3 witness-format (head classes six to ten, flags), 19 field/dispatch, 2 report-directory, 2 answer-oracle, the integer-kernel guard.
- Focused verification: 38 checks on `fc479376` (9 chain-traversal, 3 witness-format, 19 field/dispatch, 2 report-directory and 2 answer-oracle unit tests, the integer-kernel guard, the actual-artifact test over twenty conversations and 72 turns with four head-reassertion sequences and contract-restore rejections, and an allocation census of zero allocations and zero bytes including head-hop cases).
- 5,000 native API completion/checkpoint checks (2,500 construction, 2,500 fresh), including every previous-record target, abstention and follow-up, from typed accepted lists in exclusively claimed sibling attempts.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact; all fourteen older behavior/control reports have zero checked differences; twelve multi-turn histories replay identically; 24 word/Rust cases are preserved; Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases; actual generated Rust compiles and executes.

Four fits of this cycle were refused before learning or interrupted (conflicting labels on identical frames, corrected rather than tolerated; one interrupted chain charged manually) and are preserved. The immutable evidence records 5,230.592 model seconds and 673.560 engineering seconds (cumulative 66,960.019 / 67,950.000 model seconds after a standing-authorized 5,000-second extension recorded before use, 196.620 reserved), one interrupted fit charged manually, and 57,342,459,904 / 60,633,018,368 accounted bytes leaving 3,156,340,736 after the 128 MiB margin. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

Limits: previous-distinct-value intent is not implemented; a previous request on a revised head whose previous record was evicted keeps the parent's answer; plain-style requests under two phrasings receive the inherited owner sentence; both reviews' prompts are development evidence and only the fresh draw is held out; bounded authored forms with fresh spellings. Next: intermediate-version intent with exact depth labels.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

