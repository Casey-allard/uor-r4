# #1226 — Learn previous-record selection on minimal repeated-head chains in native geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-11T00:53:29Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-09-11T00:54:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1226
- **Merged:** 2026-09-11T00:54:36Z
- **Merge commit:** 8a3fb6980df86a8b5a54aec43e67cb030faac6d9
- **Base / Head:** main / UOR-Foundation:codex/minimal-head
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repairs the minimal-chain previous-record selection gap the third independent review reproduced: `Record: nelqi in opal harbor. nelqi in opal harbor. What was the previous location of nelqi? Answer:` answered a malformed `Unknown` continuation (plain) or `Unknown.` (owner-first) on `fc479376` and `90ebd843`, although record 1 was present, valid and offered at depth one under chain class seven; the construction had never crossed the two-record identical-assertion chain with the previous intent, so the artifact held no learned codes for that class. The repair is data through the existing native learner, with no new class, parser or answer rule: identical-assertion families (two and three records, competing owners between the repeats, all styles, the ring boundary and actual follow-ups) join the head-contract construction, and the candidate is refit from frozen parent `f0901dad`.

Retain `4568679d`: 1,464/1,464 construction targets in the disjoint typed partition 930 initial + 234 previous + 120 current + 180 abstain (follow-ups and previous-record rows are overlapping categories, reported as such), 3,427/3,427 preservation cases, complete removal reproducing the parent on all 1,464, the head-hop ablation leaving exactly the 1,040 targets that do not need it. A first candidate, `ec79c303`, passed the same construction but moved the source of two matched-history turns (`Where is selvi? Explain in a sentence.` after a repeated fact) from the live head to the root with unchanged text; it is rejected and preserved, and the construction gained 36 inherit-labeled explanatory current rows on the repeated chain before the refit. Before: `fc479376` passes 1,422/1,464 of the same targets, failing exactly the 42 two-record previous requests (28 `Unknown`, 14 right value through the wrong record or tense). The early actual-checkpoint artifact test on the candidate covers 24 sequences (8 head sequences including the two- and three-record identical chains), both contracts and 88 disabled-parent turns. Frozen before the draw; the independent draw (`angmv`, `ikurj`, `czmfx`) passes 1,464/1,464 targets and 1,010/1,010 preservation rows with the same control pattern. Row by row against `fc479376`: identical on every earlier construction, draw and the first two reviews' prompts; the third review differs only on the four repeated-head previous rows (213/221 against 209, no correct-to-wrong); the fresh draw differs on 58 rows, 46 of them the repaired previous requests and 12 owner-first explanatory current rows where `fc479376` emitted a past-tense sentence for a current question (on 8 the candidate returns the parent's accepted present-tense owner sentence; on 4 in the `Record: {v} holds {o}.` form neither is accepted and the candidate reproduces the parent's malformed continuation, recorded as an inherited failure). `fc479376` on the final construction: 1,422/1,464 targets, 3,415/3,427 inherited rows.

Validation executed locally:
- Optimized offline Rust core/API/example builds; formatting and claim-wording checks.
- Unit tests: 9 chain-traversal, 3 witness-format, 19 field/dispatch, 2 report-directory, 2 answer-oracle, the integer-kernel guard; the real-driver destination test now also covers the word/Rust `identity` report file (15 directory drivers plus two report files).
- Focused actual-artifact checks on the candidate: the actual-checkpoint identity test (24 sequences, 8 head sequences, both contracts, 88 disabled-parent turns) and the allocation census (zero allocations, zero bytes on every measured turn including the identical-assertion previous path).
- 2,928 construction-derived and 2,928 fresh-derived API inputs preserved on the candidate through the API driver.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact; all fourteen older behavior/control reports have zero checked differences; twelve multi-turn histories replay identically; 24 word/Rust cases are preserved; Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases; actual generated Rust compiles and executes.

Inherited formatting failures, recorded and not credited: the review's eight owner-first current requests on head chains return the correct bare value without naming the owner on every artifact including this one, and scoping probes show the `Record: {v} holds {o}.` owner-first explanatory form malformed and the plain owner-first explanatory form abstaining on `fc479376` and the candidate alike; the typed oracle is not relaxed for them and they are left to a separate cycle on the emission/instruction seam. Process corrections from the review: the `native_word_sentence identity` report is reserved before model load and covered by the real-driver check; every newly generated abstention target carries its typed accepted list; target subcounts are reported as a disjoint intent partition; the safeguards cycle's engineering total is corrected by a closure note (335,878 ms, not 330,967 ms) without touching the model ledger.

Resources: model 6,978,327 ms this cycle within the revised 7,000,000 ms allowance; ledger 74,050,616 / 74,950,000 ms after the recorded 5,000,000 + 2,000,000 ms extensions, 702,764 ms above the 196,620 ms reservation; engineering 336,092 ms; cycle storage growth 6,061,092,864 bytes within the revised 7 GiB projection, 2,412,974,080 bytes headroom above the 128 MiB margin; two nonzero-exit commands charged (a corrected test expectation and a driver-refused parent scoping run), none stopped. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

Limits: previous-distinct-value intent is not implemented; owner-first current-request formatting on head chains is an open inherited limitation; plain-style requests under two phrasings receive the inherited owner sentence; all three reviews' prompts are development evidence and only the fresh draw is held out; bounded authored forms with fresh spellings. Next: intermediate-version intent with exact depth labels.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

