# #308 — Measure external SmolLM2 teacher floor

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T21:17:41Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-31T21:44:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/308
- **Merged:** 2026-07-31T21:44:42Z
- **Merge commit:** 940a0e9fe783ea8ac74aa531feb161f04dc780f4
- **Base / Head:** main / UOR-Foundation:agent/external-referee-300
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #300.

## Summary

Adds a standalone external referee for the D3 teacher-floor measurement. The script uses Hugging Face Transformers directly on the pinned SmolLM2-135M-Instruct weights and raw D3 article text.

It deliberately does not read r4 corpus records, observation vectors, compiled stores, graphs, or score reports.

## Result

- 596 D3 held-out articles
- 71,714 target tokens
- first 128 tokens per article
- no BOS token or chat template
- external bits/token: **3.8425**
- r4 story-contiguous floor: **11.17 bits/token**

The approximately 7.33-bit gap confirms that the residual floor is pipeline-internal rather than an intrinsic SmolLM2 quality bound.

The reproducible run and interpretation are documented in `docs/transformerless/EXTERNAL_REFEREE_300.md`.

## Validation

- `uv run --with torch --with transformers --with safetensors --with blake3 python -m py_compile scripts/external_referee.py`
- Full external referee run completed successfully.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

