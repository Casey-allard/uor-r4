# #1034 — #973: freeze paired-H4 prompt-capacity result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T14:37:13Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-01T14:38:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1034
- **Merged:** 2026-09-01T14:38:37Z
- **Merge commit:** 22218652fcd9bddf76f0a574e67111a9ad4bdd99
- **Base / Head:** main / UOR-Foundation:issue-973-prompt-conditioned-capacity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- implements `R4PairedH4LanguagePathV1`, changing only the qualified retained cell's per-layer exact-H4 token address
- adds the create-once CPU4 prepare/probe/run campaign and independently sealed `R4RetainedPromptSwapContrastV1` criterion
- preserves byte-identical preparation, probe, started, commitment, reveal, and terminal result envelopes
- updates every authoritative research/roadmap/claim surface to the binding result and next action

The scientific implementation was frozen at commit `8681129d`, trainer tree CID `blake3:5ab122985b7c539fb56dcbfb7e141a25e91509a9d9db8ffd5b22633135778e64`.

## Result

Terminal: `PAIRED_H4_PROMPT_CAPACITY_FAIL`

Result CID: `blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`

Construction-only repeated joint addresses fell from `1,937,864` to `47,522` (`97.5477123%`), but that representational improvement did not increase independently frozen prompt conditioning.

Fresh nonoverlapping language passed every nonregression gate:

- candidate NLL/top-1: `3.8832293739` / `29.7801710%`
- frozen V1 NLL/top-1: `3.8901151940` / `29.7063569%`

Prompt-swap capacity did not:

- candidate `G=0.0062477543`, `282/512` wins
- V1 `G=0.0063672952`, `300/512` wins
- candidate-minus-V1 `G=-0.0001195409`
- frozen gates: candidate `G >= 0.0433216988`, delta `>= 0.0253415693`, wins `>=308/512`

Both state-off arms collapsed exactly to zero prompt gain and zero paired-logit delta. Causal/replay/reveal/forbidden-read controls passed. A fresh-process independent audit reproduced all six prompt-score passes, four trace CIDs, and all three 247,920-row heldout evaluations without discrepancy.

The provisional `9e04...` population was rejected before public freeze and before any model run: its exploratory selector omitted canonical whitespace normalization and therefore mixed canonical train/dev/test stories. The production canonical selector reproduced population CID `blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340` twice before sealing.

## Decision

Reject paired addressing as the capacity successor and preserve qualified `R4RetainedLanguagePathV1`. Do not tune or retry this revealed seam and do not run candidate generation. Attention remains established at V1's bounded scope.

The next independently frozen #973 rung isolates prompt-state-to-logit readout while holding the qualified recurrence, representation, data, optimizer, and budget fixed. #973 remains open; #954 remains blocked.

No H4-specific superiority, autonomous prompt coherence, general inference/reasoning, E8, exact lowering, WASM, or release claim follows.

## Focused verification

- paired-H4/prompt/campaign unit tests: `23/23 PASS`
- Ruff over changed Python and tests: PASS
- CLI import/help: PASS
- terminal resume/idempotence: same result CID, no retraining/rescoring
- six repository evidence envelopes: byte-identical to create-once campaign files
- independent full scorer + heldout replay: exact reproduction
- `python3 scripts/check_claim_wording.py`: PASS
- `git diff --check`: PASS
- all new JSON: parse successfully
- all changed local Markdown links: resolve

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

