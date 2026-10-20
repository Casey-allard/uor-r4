# #1122 — Implement bounded sparse geometric recurrent attention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T18:17:39Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-04T18:20:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1122
- **Merged:** 2026-09-04T18:20:07Z
- **Merge commit:** 83e7a5efc8a408adf7adba8c60c637d0f101c3b2
- **Base / Head:** main / UOR-Foundation:codex/issue-973-sparse-geometric
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The fixed recurrent reader delivered by #1120 still considered every occupied live and summary record. This change adds the first versioned sparse geometric admission step so the accepted recurrent state can advance toward the architectural alpha without changing its learned reader.

The new `R4SparseGeometricCandidateSoftmaxKVBindingV1` inspects bounded metadata for at most eight live records and four summary banks, keeps current mandatory, and admits at most eight persistent sources before any selected K/V payload is gathered. It derives `source^-1 * current` from the exact H4 sidecar, prioritizes nine signed-S3 angular shells, resolves over-budget shells with full-root pairwise maximin separation, then breaks remaining ties by causal age and physical slot. The accepted learned Q/K softmax reader, recurrent fold/eviction behavior, state size, tokenizer, sampling, geometry identities, and artifact remain unchanged. The trace retains the full `Z[phi]` root and the established q0/q1 heatmap, chirality, cosine-polarity, and typed-null fields as evidence; those fields do not become semantic scores.

The frozen two-prompt, source-free comparison used seed 9738, 16 requested generated tokens, and no fit or population evaluation. Aggregate materialized attention scores fell from 3,824 to 3,240 (15.27%), and peak read sources fell from 13 to 9 while recurrent storage remained 2,304 f32 values / 9,216 bytes. Geometry selected a different set from age-only top-eight on 33 of 35 sparse decisions. The continuations shared 12 and 3 generated tokens with the fixed recurrent comparator, so this establishes execution and boundedness only; it does not establish language quality, retrieval benefit, long-context retention, or geometric advantage. The prior #967 shortest-Cayley and #970 q0/q1 heatmap negative results retain their tested scope.

The roadmap and evidence ledger now mark sparse geometric attention complete and make one finite R4 nonlinear operator block the next build stage under #973.

Validation:

- `PYTHONPATH=tools/r4-softmax-trainer/src /Users/casey.allard/uor-r4/.uor-models/research/issue-1014/venv/bin/python -m unittest tools/r4-softmax-trainer/tests/test_fixed_recurrent_kv_binding.py` — 3 passed
- Python byte-compilation for the four changed modules
- JSON parse for the policy and claim ledger
- `python3 scripts/check_claim_wording.py`
- `git diff origin/main...HEAD --check`
- Independent source and artifact review, including all 14,400 canonical ordered H4 root pairs, causal read-before-write, per-lane selection accounting, and omitted-payload independence

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

