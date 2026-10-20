# #300 — External-referee floor measurement: SmolLM2-135M on the D3 held-out text outside the r4 pipeline

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-30T06:17:09Z
- **Updated:** 2026-07-31T21:44:43Z
- **Closed:** 2026-07-31T21:44:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/300
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The #268 close-out's single remaining experiment: teacher-force the same held-out articles (raw text, 128-token context, same positions) through the same checkpoint via HF transformers or candle, outside the r4 observation/eval pipeline entirely, and measure bits/token. ~4-6 ⇒ the remaining floor is pipeline-internal (tokenization/stream reconstruction — next hunt); ~11 ⇒ the floor is the model's true quality on this stream and stands as measured. One run decides. Quoting rule until then: 11.17 bits/token (per #268 close-out).

## Comments (1)

### comment 5147547946 — 2026-07-31T21:17:47Z — auser

External referee result is complete in draft PR #308: https://github.com/UOR-Foundation/uor-r4/pull/308

Using Hugging Face Transformers directly on the pinned SmolLM2-135M-Instruct weights and raw D3 text (no r4 corpus records, observations, stores, graphs, BOS, or chat template):

- 596 D3 held-out articles
- 71,714 target tokens
- first 128 tokens per article
- 3.8425 bits/token

The r4 story-contiguous floor is 11.17 bits/token, so the ~7.33-bit gap confirms the residual floor is pipeline-internal. The reproducible script and report are included in the PR.


---
_Comment/review ids in this file: 5147547946_

