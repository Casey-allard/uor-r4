# #1026 — feat: train relation supervision through attention (#954)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T23:58:57Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-09-01T00:00:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1026
- **Merged:** 2026-09-01T00:00:01Z
- **Merge commit:** 3d9bdc3ddedc1ca62741c68f2a9c35e3ae806a98
- **Base / Head:** main / UOR-Foundation:issue-954-relation-representation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #954.

## Outcome

Implements and executes the independently frozen C1-SB3 `R4AttendedRelationAdapterV1` slice. The mechanism result is positive at bounded representation transfer and negative at the exact promotion gate, so the campaign stops before Rust parity, full fitting, development, product reveal, or serving promotion.

## Implementation

- trains only rank-8 Q/K/V/O LoRA deltas in all six existing causal-softmax R4/Spin attention layers;
- uses the fixed tied ` yes`/` no` verbalizer with no trainable classifier head;
- adds independent lexical populations, product commitments, fail-closed manifests, and a Rust research-only artifact/answer seam;
- preserves exact source copying plus typed abstain and contradiction;
- persists a future passing preflight checkpoint and Rust binding, while the observed failed run emits no checkpoint;
- updates the current README, research, lifecycle, intelligence-plan, and #954 evidence mirrors.

## Frozen result

- base sealed positive recall: `0/76`;
- trained sealed positive recall: `73/76`;
- trained sealed negative specificity: `234/239`;
- fit outcomes: `124/126`;
- sealed outcomes: `56/63`;
- all `24/24` intended attention tensors changed and no non-attention tensor changed;
- terminal: `FAIL_REPRESENTATION_TRANSFER_PREFLIGHT`;
- result CID: `blake3:b55bd25715705450742d80f9ecde56cdd965beefb1c122e08b477b03d0949b92`.

Attempt 01 is explicitly invalid bookkeeping evidence. The corrected replay reproduced the optimizer trace. Three internal named-control booleans from the observed report are excluded from claims because they were not separately isolated; that did not affect the terminal because fit and sealed semantic exactness had already failed. The producer now scopes those controls independently.

## Verification

- 32 focused Python relation/data/campaign tests pass;
- 12 Rust relation-artifact tests pass;
- 2 Rust grounded-answer tests pass;
- focused Rust clippy with warnings denied passes;
- wasm32 library check passes;
- formatting, claim-wording, canonical JSON, and staged-diff checks pass.

The merge group owns the exhaustive workspace suite. This PR does not close #954, qualify a product answer backend, reveal product probes, establish geometry advantage/transformerlessness/reasoning, or authorize a release.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

