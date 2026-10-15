# #1042 — Record #1017 normal-use raw-continuation boundary (#1041)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T02:45:15Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-09-02T02:46:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1042
- **Merged:** 2026-09-02T02:46:39Z
- **Merge commit:** 19d05d2264b5a61ddad9f8d7e21643ca55550f09
- **Base / Head:** main / UOR-Foundation:issue-1041-normal-use-scope
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Records the frozen #1041 normal-use decision for the merged #1039 native dashboard and loopback endpoint.

- Native dashboard and all seven canonical direct requests completed on Apple Accelerate.
- Every causal/projection/R4/output-policy/decode/source-read condition passed with zero future, provider, and Ollama reads.
- Fresh narrative probes N1 and N3 passed; N2 swapped the supplied entity/object binding: 2/3.
- H1 did not recover `green box`; H2 did not recover `under the wooden chair`: 0/2 supplied-history bindings.
- Predeclared terminal: `KEEP_RAW_CONTINUATION_ONLY`.

The immutable checkpoint remains a bounded disabled-by-default source-backed single-turn story-continuation reference. This PR adds no model or runtime code and authorizes no history serializer, multi-turn/chat adapter, retraining, or prompt widening.

## Evidence correction

The record distinguishes the configured 256-token checkpoint context from response `model_shape.sequence_capacity`, which is the request-allocated horizon. All seven canonical direct requests use the originally frozen `max_tokens = 32`; three initially shortened observations were superseded by exact 32-token reruns before the decision ledger was frozen.

## Documentation

Adds the full prompt/output/timing/CID ledger and synchronizes the current README, roadmap, research, lifecycle, completion-plan, glossary, ADR, #1017/#1019 records, contributor guidance, and Milestone 21 boundary.

## Verification

- `python3 scripts/check_claim_wording.py`
- `cargo fmt --check --all`
- `git diff --check`
- independent read-only result and final documentation reviews

No Rust source changed, so no corpus-scale or workspace test campaign was launched.

Closes #1041

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

