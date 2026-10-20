# #1959 — Diluting the demand mixture keeps compliance but not retention (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T20:06:20Z
- **Updated:** 2026-10-09T13:19:56Z
- **Closed:** 2026-10-08T20:07:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1959
- **Merged:** 2026-10-08T20:07:59Z
- **Merge commit:** 7d61658087406ff540400a90dc44a387f6417c46
- **Base / Head:** main / UOR-Foundation:deepseek/dilution-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Diluting the demand mixture keeps compliance but does not restore retention

Round 119's breakthrough used 2,860 demand-bearing documents and produced compliance across every
form class while both retention turns dropped to "I do". This arm reduces the store to 700
documents - about 0.03 percent of training tokens - with everything else identical.

Compliance survives the 4x dilution: "Yellow." for a one-word demand, "Yes." and "No." for yes/no,
"4." and "60." for numbers, "Red, blue," for a list. Retention does NOT return at either
concentration: the cats turn gives "One healthy" and the name turn gives "Imagineopotamids have".

So the trade seen in round 119 is not a simple mixture-ratio effect at this scale. Compliance is
robust to the ratio - a small, well-formed demonstration of a demand and its matching answer is
enough to teach the behaviour, which makes the fix cheap - and retention is a separate problem
rather than a weight.

The natural next candidates are the store's content, since its documents are single-turn by
construction and the mixture therefore acquires thousands of documents that never require
carrying anything across a turn, and the terminal weighting interacting with multi-turn
documents. Both are separable by building a TWO-turn demand store whose second turn demands a
form about what the first turn stated - a small extension of the generator that exists.

Factual accuracy inside the correct form is still imperfect at this size ("Yellow" for grass, "60"
for the days in a week), which is now a familiar scaling question rather than a form problem.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

