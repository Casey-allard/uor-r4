# #1228 — Scope the reader's owner scan to the current turn and size its feature buffer in native geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-11T07:53:26Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-09-11T07:54:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1228
- **Merged:** 2026-09-11T07:54:28Z
- **Merge commit:** 49da65c509e8b1b9f10e7891918d3618c65e9f20
- **Base / Head:** main / UOR-Foundation:codex/reader-scope-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repairs the two defects the fifth independent review found in PR #1227's reader-request-window contract and corrects that record's labels. (1) Cross-turn owner scope: the sixteen-word owner scan was bounded only by the committed-fact byte cutoff, so a previous turn's question and the model's own answer named the owner for the next turn; after `Record: nemvi in cedar quay. polru in silver garden. Where is nemvi? Name the owner first. Answer:` the request `Where is zalfe? Answer:` returned ` cedar quay.` instead of abstaining and `Where is polru? Answer:` returned ` cedar quay.` instead of ` silver garden.` (56 of 144 actual follow-ups previously correct went wrong). (2) Capacity: `RELATION_FEATURES` was 64 while the scan can produce 2 + 4 × 16 = 66 features, so sixteen owner matches in one request panicked.

The change: `relation::read_features_scan` takes the committed-fact cutoff (a byte coordinate) and the current turn's input boundary (a token coordinate, the same one the historical query view and the handoff's scoped view use); the scan ends at the first word outside either bound and a neighbour outside the scope contributes no lexical context. Stored records are untouched and cross-turn lookup of a legitimately named owner still works. The semantics are a new versioned flag, `reader_turn_window`, layered on `reader_request_window`; `f3620cb7` keeps its measured post-fact-only scan, `Control::HistoricalVersionIntentReaderTurnScopeDisabled` withholds only the turn boundary and reproduces its leak in the artifact test, and removing the witness reproduces the frozen parent. `RELATION_FEATURES` is derived as `2 + 4 × WORD_QUERY` with unit tests at 0/1/8/15/16 matches, mixed floods and the legacy base, and the review's sixteen-`selvi` flood is generated under four controls and both public diagnostics in the artifact test. The retained artifact `15baec48` is a promotion of `f3620cb7` (no refit; the learned witness of `4568679d` is unchanged through both promotions).

Fifth review's 677 scored prompts: actual two-turn follow-ups **132/144** for the candidate against 70 (`f3620cb7`) and 106 (`4568679d`, `fc479376`); all 56 regressed rows recover; explicit owner-first emission 72/72, single-request controls 120/144, prior groups unchanged; no row correct-to-wrong against any of the three artifacts and no property degraded on the 72, 76 and 96 changed rows. The sixteen-match flood that panicked completes and equals `4568679d`. Construction-4b (2,904 targets: 930 initial + 234 previous + 1,160 current + 580 abstention including 400 absent-owner): **2,708/2,904** against 2,602 (`f3620cb7`) and 2,362 (`4568679d`); every one of the 196 failing rows also fails on both priors and is named with its inherited reason; one row loses EOS relative to `f3620cb7` because that artifact's leak happened to terminate (its answer equals `4568679d`'s). Preservation 3,386/3,386; removal reproduces the parent on all 2,904; matched histories 12/12 with exact sources; handoff-fresh 764/764; early actual-artifact test with 14 two-turn scope sequences and the flood under four controls; identical to `f3620cb7` on every earlier population and the first four reviews. Independent draw (`dplqx`, `wlaey`, `lbuyh`): **2,708/2,904**, preservation 974/974, the same inherited negatives, no degraded property against `4568679d`. API 3,044/3,045 on each population with the single failure a declared inherited row; 72/216/764 retained comparisons, fourteen older reports, word/Rust, Copy/Add, action, generated Rust and the zero-allocation census pass.

Qualification correction of PR #1227 (appended to its record, sealed reports untouched): the two defects above, the restricted disposition of `f3620cb7` (a measured candidate with single-turn gains and a cross-turn regression, not a fully preserved baseline for multi-turn requests), strict authored text alone 1,552 (1,548 is strict text plus selection), the fourth review's denominator 389 (381/389 against 357 and 337), and the reconciled totals (6,290,486 model ms, 701,081 engineering ms, live ledger 80,417,906 ms after that cycle) in `resources-closure-note.json`. Candidate provenance (source commit, dirty patch, untracked inputs, binary digests and binary copies) is now bound before a candidate's first behavioral run.

Validation executed locally:
- Optimized offline Rust core/API/example and test-binary builds; formatting and claim-wording checks.
- Unit tests: 4 historical-version (three-part reader scope under every control), 4 answer-oracle, relation owner-scan capacity and turn-boundary tests, 2 report-directory; the integer-kernel guard.
- Focused actual-artifact checks on the candidate: the actual-checkpoint identity test (46 sequences: 14 two-turn scope sequences, the withheld-boundary negative control, the sixteen-match flood under four controls and both diagnostics) and the allocation census (zero allocations, zero bytes).
- 3,045 construction-derived and 3,045 fresh-derived native API checks: 3,044 pass on each; the single failing check is the declared inherited row `turn-scope/0/forward/o-first/explain/repeat/same/plain`, reported not hidden.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact; all fourteen older behavior/control reports have zero checked differences; twelve multi-turn histories replay identically with exact source identities; 24 word/Rust cases are preserved; Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases; actual generated Rust compiles and executes.

Resources: model 6,924,221 ms this cycle within the projected 7,000,000 ms; ledger 87,572,342 / 89,950,000 ms after the recorded 7,000,000 ms extension, 2,181,038 ms above the 196,620 ms reservation; engineering 899,439 ms; cycle storage growth 7,236,526,080 bytes within the projected 7 GiB, 1,248,325,632 bytes headroom above the 128 MiB margin; twelve nonzero-exit commands charged (compile errors, a data-generation panic, my own wrong test expectations corrected in the tests, and API checks that exit nonzero on one declared inherited row), none stopped. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

Limits: previous-distinct-value intent is not implemented; absent-owner requests on reverse-form records and after a plain first turn, the eight owner-first bare-value rows on head chains and the four wrong-occurrence rows are inherited failures kept in their denominators; all five reviews' prompts are development evidence and only the fresh draw is held out; bounded authored forms with fresh spellings. Next: intermediate-version intent with exact depth labels, after these gates.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

