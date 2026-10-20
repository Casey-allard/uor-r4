# #96 — HF-path (SmolLM2) full Gate C on the D3 distribution (issue #75)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T07:59:01Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-07-23T11:17:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/96
- **Merged:** 2026-07-23T11:17:19Z
- **Merge commit:** f9c1602fafc0c7822cc04796f2477486c17cb68f
- **Base / Head:** main / UOR-Foundation:issue-75-hf-gate-c-d3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Runs cover + score + Gate C for the SmolLM2-135M compile on both D3 partitions (teacher-generated continuity + Simple Wiki natural) and fills the M.V.G. target rows 1–2 in `docs/transformerless/BASELINE.md` §4 with measured values and plain pass/fail verdicts.

So far: fixes `Tokenizer::encode` buffer sizing (byte-level tokenizers decompose multi-byte chars into up to 4 tokens; the char-sized buffer panicked on the Simple Wiki corpus via the SmolLM2 tokenizer).

## Status

- [x] encode buffer fix + regression test
- [ ] HF natural-partition observation (3000 articles, in progress)
- [ ] SmolLM2 continuity corpus generation (150k target)
- [ ] score_report.json per partition posted to #75
- [ ] BASELINE.md §4 rows 1–2 measured with verdicts
- [ ] fixture-vs-D3 gap analysis

Do not enqueue until the measurement is in.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

