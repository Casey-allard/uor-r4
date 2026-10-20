# #1223 — Repair reassertion-versus-truncation chain contract and seal report directories

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-10T18:29:02Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-09-10T18:30:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1223
- **Merged:** 2026-09-10T18:30:14Z
- **Merge commit:** ce6046b58b5a668e15f542c8f46b8243099e9938
- **Base / Head:** main / UOR-Foundation:codex/reassertion-chain-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The independent review of PRs #1221/#1222 reproduced four correct-to-wrong answers in retained `e44a9ae4`: a resident, nonconflicting same-value reassertion (`selvi in Dusk Ridge. selvi in Dusk Ridge. selvi now in Copper Vale.`) was mistaken for a truncated history, so an initial-version request answered `Unknown.` where `e3a906c8` and `f0901dad` answered `Dusk Ridge`. The chain contract is now versioned in the witness (`reassertion_links`): a validated link is an explicit revision or a resident same-owner, same-value, nonconflicting reassertion re-proven byte for byte against its predecessor, and a chain is truncated only when the next predecessor is proven evicted from the ring; a rejected link is not absence. The same contract is carried through selection, the field-anchor proof, emission and checkpoint restore; a `HistoricalVersionIntentReassertionDisabled` control exposes the legacy contract. Legacy artifacts have no flag on the wire and keep their exact behavior.

Retain `90ebd843`, refit from frozen parent `f0901dad` over the construction extended with reassertion, equal-value, boundary, evicted-reassertion and multi-turn follow-up families: 976/976 construction targets (690 exact root answers and selections, 132/132 abstentions through actual no-read decisions, 154/154 follow-ups keeping the exact immediate-previous/current record after one or two earlier turns answered by the model itself) with 3,309/3,309 preservation cases, complete removal reproducing the parent on all 976, and the reassertion-ablation control leaving exactly the 820 targets that need neither the link nor the eviction proof. Before the repair, `e44a9ae4` passes 806/976 of the same targets (0/144 reassertion-link targets; ten follow-ups answer `Unknown.` for a current request after an abstained initial turn). The flag-only diagnostic candidate `6e8901a7` (no parameter changed) repairs the four confirmed regressions and the four inherited middle-reassertion failures but leaves four boundary abstentions wrong, which is why a refit was required. A first refit, `d9712bfe`, passed every single-turn gate and an independent draw and was then rejected by the focused actual-artifact test: in a multi-turn session it anchored an ancestor for the previous and current follow-ups; it is preserved with its evidence and the design was reopened with follow-up documents in training, evaluation and the API check. On the review's 50 scored prompts the retained candidate is correct on all 50 (`f0901dad` 37, `e3a906c8` 42, `e44a9ae4` 42) with no correct-to-wrong change. Row by row it is identical to `e44a9ae4` on every prior population (3669, 1252, 794, 890, 3789 and 1372 rows) and differs from `e3a906c8` only on the declared evicted-root abstention rows. The separately frozen second fresh draw passes 976/976 targets (132/132 abstentions, 154/154 follow-ups) with 892/892 preservation cases; on that draw every row that differs from `e44a9ae4` (200) or `e3a906c8` (330) is a declared reassertion, follow-up or eviction target.

Validation executed locally:
- Optimized offline Rust core/API/example builds; formatting and claim-wording checks.
- Unit tests: 8 chain-traversal tests (reassertion link, eviction proof), 3 witness-format tests (contract flag, follow-up documents), 19 field/dispatch tests, 2 report-directory collision/manifest tests, the integer-kernel guard.
- Focused verification: 35 checks on `90ebd843` (8 chain-traversal, 3 witness-format, 19 field/dispatch and 2 report-directory unit tests, the integer-kernel guard, the actual-artifact test over sixteen conversations and 56 turns with reassertion and evicted-reassertion sequences and contract-restore rejections, and an allocation census of zero allocations and zero bytes); the same test rejected the first refit `d9712bfe`.
- 3,904 native API completion/checkpoint checks (1,952 construction, 1,952 fresh), including every abstention and every follow-up replayed through the same API session.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact; all fourteen older behavior/control reports have zero checked differences; twelve multi-turn histories replay identically; 24 word/Rust cases are preserved; Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases; actual generated Rust compiles and executes.

Evidence preservation: every driver used now claims its report directory exclusively before model work and seals completed attempts with a BLAKE3 manifest (`uor_r4_core::report_output`); the API check creates its report file exclusively. The first cycle's claim that the rejected `d6144304` older reports survive in command logs was inaccurate: those logs hold at most a one-line count; the fourteen reports and the word/Rust report were overwritten in place. An erratum record lists what survives, what now occupies the paths, and the read-only search; the original snapshot bytes are unchanged; nothing was reconstructed. Two narrative counts are corrected (twelve fit/refit invocations, not seven; 76 owner-sentence alternatives on the final draw, not 64).

The immutable evidence records 8,831.255 model seconds and 1,404.460 engineering seconds (cumulative 61,371.301 / 62,950.000 model seconds after standing-authorized ledger extensions of 3,000, 1,000 and 4,500 seconds recorded before use, 196.620 reserved), one storage-monitor stop and one rejected refit with its full qualification, and 52,884,475,904 / 54,190,567,424 accounted bytes leaving 1,171,873,792 after the 128 MiB margin. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

Limits: abstention after a reassertion chain loses its root is by exact-record identity, not value recovery; a head that is itself a reassertion has no historical read; plain-style requests under two phrasings receive the inherited owner sentence; intermediate-version intent is not claimed; bounded authored forms with fresh spellings only.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

