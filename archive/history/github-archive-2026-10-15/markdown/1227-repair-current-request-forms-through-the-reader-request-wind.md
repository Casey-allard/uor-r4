# #1227 — Repair current-request forms through the reader-request-window contract in native geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-11T03:53:04Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-09-11T03:54:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1227
- **Merged:** 2026-09-11T03:54:05Z
- **Merge commit:** bc4af4850ceff22d2a7db6f263a495387a0874af
- **Base / Head:** main / UOR-Foundation:codex/current-form-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repairs the current-answer interaction the fourth independent review found in PR #1226 and corrects its qualification. On `Record: amber quay holds pemru. Record: amber quay holds pemru. Where is pemru? Explain in a sentence. Name the owner first. Answer:` the retained `4568679d` emitted ` quay holds pemru is the place.` and ran to the output cap, where `fc479376` had emitted a complete but wrong-tense, wrong-record sentence. The first divergent decision is the frozen persistent relation reader: its owner scan covered only the eight most recent request words, so a request whose two instructions follow the question (twelve words, owner ninth) produced no relation candidate; the dispatch fell back to a query-occurrence copy that the extent operator mis-spanned on reverse-form records, and the owner-first field composer could not anchor.

The change is a versioned **reader-request-window contract** (`reader_request_window` on the historical-version witness): the reader scans the request words after the latest committed fact, up to the sixteen-word view, for the named owner. `relation::read_features`, the inherited eight-word feature base of every later learner, is untouched; the extent operator, field composition, emitters, learned codes, dictionaries and receipts of `4568679d` are unchanged, so the retained artifact is a promotion (`promote-window`), not a refit. `Control::HistoricalVersionIntentReaderWindowDisabled` withholds it and removing the witness reproduces the frozen parent exactly, asserted by the actual-artifact test on eight new sequences. An unscoped sixteen-word variant (`52fc916b`) was rejected at the stop gate: it repaired the same 24 review rows but answered absent or competing owners from another record on 15 retained handoff rows.

Retain `f3620cb7`. Fourth review's 413 scored prompts: **381** for the candidate against 357 (`4568679d`) and 337 (`fc479376`), explicit owner-first emission **72/72**, no row correct-to-wrong against either, and on every one of the 24 changed rows acceptance, complete value, expected record and exact head source improve with EOS restored where it was lost and no property degraded. Construction-3 (1,704 targets: 930 initial + 234 previous + 360 current + 180 abstain): **1,700/1,704** against 1,644 for `4568679d`, the typed current family **236/240** against 180, preservation 3,386/3,386, complete removal reproducing the parent on all 1,704; matched histories 12/12 with exact sources; handoff-fresh 764/764; the early actual-artifact test with 8 new reader-window sequences and 104 disabled-parent turns; identical to `4568679d` row by row on every earlier population except the four repaired reverse-form rows. Independent draw (`xtpxf`, `vtzkh`, `ccgzz`): **1,700/1,704**, preservation 974/974, 56 changed rows against `4568679d` all improved and none degraded. 3,408 + 3,408 API inputs, 72/216/764 retained comparisons, fourteen older reports, word/Rust, Copy/Add, action, generated Rust and the zero-allocation census pass.

Qualification correction of PR #1226 (appended to its record, sealed reports untouched): the four changed reverse-form rows were not "malformed on both artifacts"; `fc479376` gave a complete, root-anchored, terminated, wrong-tense sentence and `4568679d` an unanchored, incomplete continuation without EOS on three of four. The 36 unlabeled explanatory rows are replaced by 240 typed current targets (three layouts × repeat × competitor × five request forms × single or actual second turn) labeled with the live head, complete value, present tense and declared form; `answer_oracle::accepted_explanatory` fixes the explanatory acceptance (owner sentence only under an explicit owner-first instruction; the inherited `VALUE is the place.` form or the owner sentence otherwise, per the reviewer's own correction). Selection of a current target requires the live head record's own value occurrence; a read through the committed relation source is counted separately. Every differing row against `4568679d` and `fc479376` is classified by acceptance, owner/tense/form, complete value, expected record, exact head source, EOS and relation-state integrity (`classify-rows.py`). Two stop gates (matched histories with source identity, handoff panel, early actual-artifact test) now precede any expensive comparison; the previous cycle's engineering total is reconciled by a closure note (336,370 ms).

Validation executed locally:
- Optimized offline Rust core/API/example and test-binary builds; formatting and claim-wording checks.
- Unit tests: 4 historical-version (reader-window control law included), 4 answer-oracle (explanatory forms), 22 relation/field/handoff/role, 2 report-directory; the integer-kernel guard.
- Focused actual-artifact checks on the candidate: the actual-checkpoint identity test (32 sequences, 8 reader-window sequences, 104 disabled-parent turns) and the allocation census (zero allocations, zero bytes including the two new current rows).
- 3,408 construction-derived and 3,408 fresh-derived API inputs preserved through the native API driver, typed current targets included.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact; all fourteen older behavior/control reports have zero checked differences; twelve multi-turn histories replay identically with exact source identities; 24 word/Rust cases are preserved; Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases; actual generated Rust compiles and executes.

Resources: model 6,290,011 ms this cycle within the projected 8,000,000 ms; ledger 80,417,431 / 82,950,000 ms after the recorded 8,000,000 ms extension, 2,335,949 ms above the 196,620 ms reservation; engineering 700,807 ms; cycle storage growth 5,607,665,664 bytes within the projected 8 GiB, 4,259,397,632 bytes headroom above the 128 MiB margin; four nonzero-exit commands charged (two compile errors, two wrong test expectations corrected in the test), none stopped. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

Limits: previous-distinct-value intent is not implemented; the eight owner-first bare-value rows on head chains remain an inherited emission limitation; four second-turn plain requests after a repeated fact read another occurrence (text-correct, not selected); all four reviews' prompts are development evidence and only the fresh draw is held out; bounded authored forms with fresh spellings. Next: intermediate-version intent with exact depth labels, after this repair.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

