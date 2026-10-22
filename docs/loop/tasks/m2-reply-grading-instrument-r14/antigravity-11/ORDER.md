# ORDER: m2-reply-grading-instrument-r14, antigravity-11

Lab: antigravity. Item: [m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md). Base: `b0430d406`.

## Goal

The item's done-when reads:

> A scope note on `#2029` records, for each of the three R14 requirements, whether the v2 grader meets it and on what evidence, and says what is still missing. Each missing part is built and tested, or the note says plainly that it stays out of scope. For abstention scoring, the note shows a constant-reply control run and the result. A non-author reviews the note and the instrument. The PM, not this item, rules whether the instrument passes R14.

## Cancel probe (run first)

Ran `rg -n "abstention_discrimination" crates/uor-r4-training/src/bin/chat-grade.rs` (`0` matches), verified `docs/labs/m2-reply-grading-instrument-r14/` does not exist at base `b0430d406`, and ran `~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0` (`25 passed; 0 failed; 0 ignored`). Issue `#2852` had 0 comments prior to our claim (`https://github.com/UOR-Foundation/uor-r4/issues/2852#issuecomment-6338088893`). The task stands.

## Files in scope

- `crates/uor-r4-training/**`
- `data/panels/**`
- `docs/labs/m2-reply-grading-instrument-r14/**`

## Done when

1. Scope note posted on `#2029` (and cross-linked on `#2852`) recording for each of the three R14 requirements (`v2` check table with frozen positive/negative controls, per-category best-constant and copy-control pairing, and abstention-as-discrimination scoring) whether the `v2` grader meets it and on what evidence, naming what was missing (explicit abstention-as-discrimination scoring across `abstain_target` vs `answer_target` rows), and naming all positive, negative, constant, and copy controls and their `MANIFEST.sha256` / fixture hashes.
2. `abstention_discrimination` implemented in `crates/uor-r4-training/src/bin/chat-grade.rs` (`AbstentionDiscrimination` tally in `Accumulator`, `Group::headline`, and `check_only_controls`, scoring `true_abstain`, `false_answer`, `true_answer`, `false_abstain`, `wrong_answer`, exact integer cross-product `true_abstain * answer_rows > false_abstain * abstain_rows`, and paired two-sided exact McNemar against each constant reply and `max(best_constant, copy_first_stated, copy_last_stated)` across the joint `abstain_target ∪ answer_target` rows), with finite unit tests in `chat-grade.rs` verifying that `constant[0]`, `copy_last_stated`, and `copy_first_stated` all fail `abstention_discriminates`, while a selective abstaining+answering control passes `abstention_discriminates`.
3. Host re-score of the 7 archived `replies.json` artifacts (`replies-29m.json`, `rp-D0`, `rp-D10`, `rp-D25`, `v5-D0`, `v5-D10`, `v5-D25`) plus constant and copy controls recorded in `docs/labs/m2-reply-grading-instrument-r14/README.md` and `docs/loop/tasks/m2-reply-grading-instrument-r14/antigravity-11/REPORT.md` (`References #2852`, `References #2029`; PM rules whether the instrument passes R14).

## Stop and ask if

- Any file outside the declared scope needs modification.
- Setting or restating any numeric reply target is required (reserved to PM/owner after R14 ruling).
