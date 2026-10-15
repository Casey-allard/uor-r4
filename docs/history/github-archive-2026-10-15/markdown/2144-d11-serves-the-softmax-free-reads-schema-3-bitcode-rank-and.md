# #2144 — D11 serves the softmax-free reads: schema /3, BitCode, rank and Hamming-rank reads, audit FULL PASS — preparation 2/2 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T05:40:22Z
- **Updated:** 2026-10-10T06:04:28Z
- **Closed:** 2026-10-10T05:41:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2144
- **Merged:** 2026-10-10T05:41:24Z
- **Merge commit:** c69a6aabc59bbd78be008d4ea09dff562ea795fd
- **Base / Head:** main / UOR-Foundation:claude/softmax-free-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: softmax-free served read (M4 item 0) · count 2/3 · headline softmax at runtime yes → yes

**Preparation 2 of 2.** It prepares the served half of the pre-registered arm group 1 run ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433)): the served BPB at 512 windows and the served cost of arms B (rank) and D (Hamming-rank) against arm A. Decision test: without this port, B and D have no served column and the bar's served-cost clause cannot be judged. With it, the result PR can decide KEEP or REJECT.

## What and why

- **Schema `uor-r4.lut-stack/3`:** `shape.read_select {window,k}`, `read_weights "rank"` and `read_binary`. `/2` already means a pointer stack, and the frozen D10 engine refuses `/3`.
- **D11 rank read:** an allocation-free integer flock selector, rewritten as bounded insertion because the library sorts compiled to `madd`/`mul`. NoRead is ranked and loses ties. The weights are Q31 `1/(r+1)` tables precomputed at load. At most `window+k+1` value rows are read per head.
- **D11 Hamming-rank read:** `BitCode` (sign bits, XOR binding, SWAR popcount without a multiply or NEON). The score is a load-time per-head table indexed by Hamming distance, equal to the engine's own dense score of ±ONE vectors.
- **Exporter:** writes `/3` for flock + rank/hamming-rank models. Every other refusal is kept: flock + softmax, a non-zero sink, U(1).
- **`d11-evaluate reference=none model=…`:** D11 against float on the same windows, for artifacts D10 cannot read.
- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md` § Serving.

## Result

No model result: that is the result PR's job. Decision: NOT YET PROMOTED (preparation).

## Checks at this head

- `cargo test -p uor-r4-integer --lib`: 282 passed. This includes `read_schema_*`, `read_rank_*` (weights equal `rank_table_q31` by rank, 0 off the support, NoRead tie rule), `read_binary_*` (table equals the dense ±ONE score; served scores are table plus age) and the BitCode SWAR check against a naive popcount.
- `stack_d11_oracle`: 13 passed. /1 and /2 still serve bit-identically to D10.
- `stack_softmax_free_export`: 3 passed. D11 against float on random 4-bit weights: rank −0.039, hamming −0.012 nats, top-1 1.0. Softmax + flock is refused, a plain stack is unchanged, and U(1) + rank is refused.
- `--lib stack_export`: 17 passed. Frozen `uor-r4-lut`: 25 passed.
- `python3 scripts/audit_zero_matmul_serving.py --stack target/release/uor-r4-stack`: **FULL PASS**, 0 forbidden instructions, 91 reachable functions.
- **Negative gates:**
  - Library sorts restored: the audit fails on `flock_select_integer`'s sort callees.
  - The rank path given softmax weights: the `read_rank` tests fail.
  - Binary layers given the dense score: `read_binary` fails.
  - `read_binary` export forced false: the export test fails.
  - The U(1) fix set aside: the new test fails. All restored: pass.
- **Independent fresh-context review:** one high finding (the early return skipped the U(1) refusal), fixed in cfefcf481 with a pinned test. One medium (the NoRead slot when `null=None`) does not apply: rank needs a geometric stack, and every geometric read passes its NoRead slot. One low remains: `debug_assert` plus return on unreachable head-loop failures, noted.
- `check_claim_wording.py`: pass.

## Checklist
- [x] Checks at this exact head (above)
- [x] Record section and current-state entry with **Next:**
- [x] The headline is unchanged, so STATUS/ROADMAP/#2028 are unchanged
- [ ] After merge: verify on fresh origin/main, delete the branch and worktree, hygiene, comment on #2032

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6094296608 — 2026-10-10T05:40:30Z — Casey-allard

Exact-head self-review at `ff4cc49cedaf21421b6ef0b9a67e09f11ce1218d`: the checks in the description ran at cfefcf481. The only later commit is ff4cc49ce, docs/record only, and the claim-wording gate passes. Based on origin/main after #2142. The fresh-context review's high finding is fixed with a pinned test and a negative gate.


---
_Comment/review ids in this file: 6094296608_

