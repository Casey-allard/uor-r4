# #1167 — feat(native): add experimental geometric source-span copying

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T05:49:21Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-09-07T05:50:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1167
- **Merged:** 2026-09-07T05:50:39Z
- **Merge commit:** c4ab214db0598e6a65785fcc3f4be2e7c89362a8
- **Base / Head:** main / UOR-Foundation:codex/contextual-emission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained native model selects the right source but truncates values such as `New York` to `New.`. This adds an optional learned Continue/Finish component using the existing signed-H4 code/landmark learner and causal copy cursor. Exact adjacent source words and separators form a bounded span, while the initial source commitment and all parent parameters remain fixed. Extended spans are limited to 28 total bytes within the existing 32-step response limit; no serving matrix products or new expert/paging system are introduced.

The fitted artifact is **experimental and not promoted**. `c6a98c04` remains the active checkpoint. The separator-only feature repairs multiword copying but cannot distinguish semantic boundaries at the same separator:

| Complete answers including EOS | Parent/disabled | Candidate |
| --- | ---: | ---: |
| Construction | 2/5 | 5/5 |
| Exposed open cases | 1/6 | 6/6 |
| Separate fresh diagnostic | 3/10 | 9/10 |
| Retained construction | 663/663 | 647/663 |
| Older exposed role-read cases | 24/24 | 20/24 |

All twenty regressions copy past a location into `holds OWNER`. The explicit same-space phrase-boundary case fails; double-space transport is unsupported. Other numeric, literal, identifier, relation, dependent and session preservation passes. The next direction is context-sensitive extent using candidate-owned ordered identity and query/value binding, with fresh same-separator stop/continue construction contrasts. README, roadmap, current state, lifecycle, research and architecture audit pointers preserve this decision and its negative evidence.

Validation: pinned Rust release example and test binaries compiled; six focused codec/extent/snapshot tests, integer-kernel source guard and native architecture policy passed. The actual artifact test checks complete parent restoration, malformed code/extent rejection, original source commitment, interrupted copying, every-byte checkpoint replay and zero steady-state allocations. Corrected uncached continuation timing has 28 samples, median 292 ns/max 145,667 ns, excluding initial source selection, loading, encoding, ingestion and checkpoints. This is mostly copying work, with no end-to-end, energy or broad language claim. The initial cached timing receipt is preserved and excluded from uncached evidence. Formatting and claim-wording checks passed. Required remote statuses remain compatibility acknowledgements, not model tests.

One fit; no post-fresh tuning. All model work and verification consumed 83.477 seconds within the 90-second cycle; cumulative 4156.246/4170 seconds leaves 13.754 seconds. Engineering checks consumed 422.552 seconds within 900 seconds; sampled growth is approximately 66.9 MB within 96 MiB. No external compute, budget extension, research deletion or artifact removal. Full evidence and artifact identities are in `docs/native_geometric_source_span_1139.md` and its linked JSON.

Refs #1139, #1140. This does not close either capability goal.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

