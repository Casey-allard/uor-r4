# #389 — certify: price the uncovered-vocab floor on the candidate scale (#387)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T18:29:28Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-03T18:56:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/389
- **Merged:** 2026-08-03T18:56:06Z
- **Merge commit:** 2c1bfa28a7548903c463d14b8cdb12171d99910c
- **Base / Head:** main / UOR-Foundation:graph-bits-offset-check
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #387.

## The bug

`outcome_bits` charged uncovered vocabulary at `root_floor` while candidate
scores carry `transition_offset`. The offset is added to **every** candidate —
rank-neutral, so no argmax, recall, or teacher-rank measurement could detect it —
but the floor does not carry it. Candidates and the floor were priced on
different scales.

On the real bundle the offset is **−118 nats**, putting every candidate that far
*below* where the floor prices an unseen token. The signature is an inversion:
bits were **higher when the teacher was retrieved** than when it was absent.

| slice | bits (all) | teacher present | teacher absent | mean offset |
| --- | ---: | ---: | ---: | ---: |
| exact_context | 13.31 | 10.47 | 15.52 | 0.00 |
| graph | 115.05 | **143.71** | **46.75** | **−117.90** |
| novel | 91.03 | 125.55 | 41.80 | −111.44 |

## The fix

Shift the floor by the same offset. Softmax is shift-invariant, so this is
equivalent to removing the offset from candidates, and it is a **no-op wherever
the offset is zero**. Each call site passes its own outcome's offset; `legacy`
and the root-only counterfactual pass `ZERO` because they carry none.

## Verification, both directions

**Real bundle** — corrected, inversion gone:

| slice | before | after | present | absent |
| --- | ---: | ---: | ---: | ---: |
| exact_context | 13.31 | **13.31** (offset 0) | 10.47 | 15.52 |
| graph | 115.05 | **24.07** | 25.00 | 21.87 |
| novel | 91.03 | **22.69** | 22.02 | 23.63 |

**Fixture** — offset is `0.000` on every slice, and bits are unchanged at
6.83 / 17.10 / 9.29. The fix provably cannot move it.

## Two consequences

1. **Fixture-based conclusions in #364 are unaffected.** The parity ceiling, the
   +3.09 / +1.32 residual-harm figures, and the 12.00 root-only baseline were all
   measured where the offset is zero. No re-derivation needed.
2. **The corrected real-bundle gap now matches the fixture.** Graph 24.07 vs
   exact_context 13.31 is ~10.8 bits; the fixture's 17.10 vs 6.83 is ~10.3 bits.
   The two corpora agree. The earlier "catastrophic 126 bits / confidently wrong
   output" reading was the artifact, not a property of the model — that
   characterization is withdrawn.

Top-1 and every rank-based result are untouched throughout, by rank-neutrality.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
certify suite (11 files) green.

Related: #387, #364, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

