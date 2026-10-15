# #1773 — addressed_attention: arithmetic reachability probe and arithmetic oracle (two bins whose commits missed #1770)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T00:59:45Z
- **Updated:** 2026-10-09T13:18:54Z
- **Closed:** 2026-10-06T01:00:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1773
- **Merged:** 2026-10-06T01:00:52Z
- **Merge commit:** 07b38c3e505fc681e88a6b854fe11fd0a4bc9a37
- **Base / Head:** main / UOR-Foundation:deepseek/aa-instruments
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two measurement bins that did **not** reach main when #1770 merged — the merge queue took the head as it stood when the PR was enqueued, and both commits landed on the branch afterwards. Content is byte-identical to the committed versions on `deepseek/aa-dispatch` (sha256 verified: `2329d7cb…` and `9aa84da3…`).

## `addressed-attention-arithmetic-reach.rs`

Answers *can any objective make the control head select `AddAB`/`SubAB`?* — **yes, 21** at 1000 steps, up to **209** with an EOS turn cycle. The earlier `selected_operations = 0` was a **mask-eligibility** artefact: `objects.rs:233` starts `AddAB`/`SubAB` hard-false, flipping true only when both operands decode as canonical decimals (`objects.rs:761-790` — optional `-`, **no leading zeros**, ≤20 bytes, ASCII digits).

The separation is measured, not argued: prose documents never offer arithmetic under *any* objective (including argmax and arithmetic-first); digit runs offer and choose it at ~25-32% of offers. The sharp control is digits that are space-separated — `decode_i64` rejects the spans, 0 offers. Raw mask probe and a 17-config sweep included.

## `addressed-attention-arith-oracle.rs`

Builds `arith-canonical-decimal/1` (32 examples, bare canonical decimals in the readable window) and decomposes the failure. Acceptance criteria frozen and sealed **before** any engine run:

| arm | correct | CE | offered | chosen | value exact | emitted == produced |
|---|---|---|---|---|---|---|
| Full | **0/80** | 5.567049 | 27 (11.64%) | 9 | 6/6 | **0/6** |
| ReadDisabled | 1/80 | 5.554818 | 0 | 0 | – | – |
| ExactPayloadMasked | 0/80 | 5.569378 | 27 | 13 | 3/3 | 0/3 |
| StateTransportDisabled | 0/80 | 5.590245 | 3 | 0 | – | – |
| forced read | 0/80 | – | 98 | 23 | 20/20 | 0/20 |
| **forced read + forced operator** | 0/80 | – | 98 | 83 | **80/80** | **0/80** |

**The mask, the read and the integer operator are not the blocker** — given canonical decimals the mask opens, the forced-read arm selected the intended operand spans 80/80, and the operator computed the exactly correct value 80/80. **The untrained 257-way emission head is the bottleneck**, emitting that value 0 times.

Caveats stated by the author: all arms use unfitted `Parameters::seeded(7341)`, so this cannot separate "cannot learn to copy" from "untrained"; and the split's expected answers were authored in the bin (exact `i64` + `objects::encode_i64`), not a frozen external fixture.

Both bins are measurement-only — production code unchanged, module tests pass 50/0/7. Sealed roots: `aa-arith-reach-20261005-203325`, `aa-arith-oracle-20261005-204407`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

