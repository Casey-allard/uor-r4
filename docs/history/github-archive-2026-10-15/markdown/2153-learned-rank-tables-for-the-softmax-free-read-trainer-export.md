# #2153 — Learned rank tables for the softmax-free read: trainer, export, D11 (the decisive run's one preparation) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T10:36:00Z
- **Updated:** 2026-10-10T10:48:59Z
- **Closed:** 2026-10-10T10:48:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2153
- **Merged:** 2026-10-10T10:48:45Z
- **Merge commit:** bc0a6937699dc6c40abf08b5ce1996ed045fca76
- **Base / Head:** main / UOR-Foundation:claude/learned-rank
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: softmax-free served read (M4 item 0) · count 4/3, the single decisive continuation after the 3/3 pivot · headline softmax at runtime yes → yes

**The one preparation PR** the [pivot card](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6096272058) allows. It prepares arm L, rank tables learned in training (2 seeds, `flock:8:8`, the M4 recipe), which is already training on `583vvwhk05yy1p` from this branch's trainer head `383e49410`. Decision test: without this PR, L has no export or served column, so the bar cannot be judged.

## What and why

- **Trainer, `read_weighting=learned_rank`:** per-layer `read.rank_logits` `[heads, window+k+2]`, initialized to `ln(1/(r+1))`, so it starts equal to `rank`.
  - Weights per support size are `softmax(ℓ[..m])` by rank, with an exact logit gradient.
  - q, k and aux keep the straight-through flock-softmax gradient. There is no weight decay on the logits.
  - Save/load, CLI `train`/`dialogue-train`, resume checks and replicas all carry it.
- **Export:** `read_weights: "learned"` and u32 `read_rank.<l>` tables of `heads × M × M` Q31 weights. Each row is rounded by largest remainder to sum to exactly 2^31.
- **D11:** reads the (head, m) row by rank, and refuses missing or malformed tables. The tables are constant: no exponential or softmax at runtime.
- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md` § The decisive run, plus a current-state entry.

## Checks at this head

- `cargo test -p uor-r4-training --lib -- read_weighting stack_export fused_read`: 28 passed. This covers fresh learned = fixed, a finite-difference logit gradient, trained logits changing, and save/load bit-exact.
- `--test stack_softmax_free_export`: 4 passed. Learned D11 against float −0.061 nats, top-1 1.0 on random weights, and served differently from the fixed table.
- `--test stack_d11_oracle`: 13 passed. `cargo test -p uor-r4-integer --lib`: 284 passed. `uor-r4-lut`: 25 passed. `cargo check -p uor-r4-training --examples`: ok.
- `audit_zero_matmul_serving.py --stack`: **FULL PASS** (91 functions).
- **Negative gates:**
  - The forward ignoring the learned table: the sharp-profile and gradient tests fail.
  - The read routing LearnedRank as Rank: the trained-logits test fails.
  - The engine using the fixed table: the serving test fails.
  - The exporter writing "rank": the export test fails. All restored: pass.
- **CPU smoke:** `train … select=flock:4:4 read_weighting=learned_rank` exits 0, and the logits are saved and trained.
- `rustfmt --check` on the changed files and `check_claim_wording.py`: pass.

## Result
No model result yet; that is the result PR's job. Decision: NOT YET PROMOTED (preparation).

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6096708558 — 2026-10-10T10:47:54Z — Casey-allard

Exact-head self-review at `33e778d836d89294b84baaff8dfc23f3ef1fc4d4`. The independent fresh-context review's verdict was SHIP, with two minor findings, both fixed here:
- A non-finite rank logit is now refused at export, instead of stalling the round-robin.
- A new two-head finite-difference test pins the per-head logit-gradient offset. Negative gate: with the offset forced to 0 the test fails; restored, it passes.

At this head: `read_weighting` 10 passed, `stack_softmax_free_export` 4 passed, rustfmt clean. The other checks in the description ran at e8ca72211. The delta since then is these two fixes only.


---
_Comment/review ids in this file: 6096708558_

