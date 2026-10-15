# #1117 — #973: Make retained attention address-aware

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T15:22:16Z
- **Updated:** 2026-09-04T15:24:48Z
- **Closed:** 2026-09-04T15:23:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1117
- **Merged:** 2026-09-04T15:23:41Z
- **Merge commit:** abb21aa20ccd53f8afc4223430505c58ae8c5646
- **Base / Head:** main / UOR-Foundation:codex/973-attention-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained reader transported keys, values, and occupancy through the same H4 permutation, then scored only query/key content. That makes the immediate softmax read invariant to the transport and leaves the current relative group address out of the attention decision.

This adds `R4ContextualKeyValueAddressReadLanguagePathV1`, with one zero-initialized logit bias per layer, head, and current relative address. The version preserves the contextual K/V writes, exact-H4 state transport, decay/write gates, 23,040-value recurrent state, occupancy, tied head, and historical policies. It adds 960 learned scalars for 253,120 parameters total. Inference requires a complete address-aware artifact; only the fitter may load the pinned retained V1 artifact as a zero-bias warm start.

The one bounded fit completed 128 updates over 2,048 open windows / 245,760 targets in 96.298573 seconds on CPU4. Artifact: `blake3:fb82bfe43289f3de82ff4889eb1963bc426a446d6d8b574e724ef59094356ef1`. Last-16 mean training loss was 3.9163646698 versus 3.9164103419 for the immediately preceding contextual-K/V arm, a 0.0000456721-nat difference. Both fixed generated continuations were token- and byte-identical to that arm. This is a bounded negative; it does not justify a full epoch or establish that attention or H4 generally fails.

Validation:
- Python source compilation and diff checks
- zero-bias logits/K/V state exactly match the preceding reader
- nonconstant address bias changes logits
- strict inference rejects the base artifact; the fit-only loader accepts the pinned V1 warm start; the complete fitted artifact loads all 960 bias values
- build-first policy check
- independent focused source and result review

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

