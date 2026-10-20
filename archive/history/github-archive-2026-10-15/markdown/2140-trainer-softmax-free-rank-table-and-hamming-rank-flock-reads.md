# #2140 — Trainer: softmax-free rank-table and Hamming-rank flock reads, preparation 1/2 of the A–D run (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T04:17:26Z
- **Updated:** 2026-10-10T04:31:34Z
- **Closed:** 2026-10-10T04:18:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2140
- **Merged:** 2026-10-10T04:18:39Z
- **Merge commit:** 5a389aed4f141227262109ff84446d2ed37c5a96
- **Base / Head:** main / UOR-Foundation:claude/softmax-free-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: softmax-free served read (M4 item 0) · count 1/3 · headline softmax at runtime yes → yes

**Preparation 1 of 2.** It prepares the pre-registered arm group 1 run (A, B, D × 2 seeds, [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433)). Arms B and D are training now from this PR's head `f46ec1964`, built on the EU-RO-1 pod with CUDA parity PASS.

## What and why

- `StackModel::set_read_weighting` and the CLI `read_weighting=rank|hamming_rank` add two softmax-free reads. `train` also gets `select=flock:W:K`.
- **rank:** the flock-kept sources plus NoRead, weighted by a fixed normalized `1/(r+1)` rank table. No exponential in the forward weights.
- **hamming_rank:** the same, after a straight-through sign of the query and key. For ±1 vectors the L2 or Lorentz score is a monotone function of `popcount(q⊕k)`.
- **Gradient (straight-through):** q, k and aux get exactly the flock-softmax gradient; v gets the rank weights.
- **Softmax path:** unchanged.
- **Persistence:** saved as `config.json` `read_weighting` (absent for softmax). Reports, lineage, resume checks and data-parallel replicas all carry it.
- **Refused:** bf16; no flock; and an address, latch, lineage or span, which replace the ordinary read. The independent review asked for the last one, and it was added.
- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md`.

## Result

No model result yet. That is the result PR's job. As a reproduction check, arm A (softmax, `main` 5b241d89c) at 512 windows reads float NLL **1.7250754 / 1.7253881** for seeds 1 and 2, which is 0.877118 / 0.877277 BPB on the stream basis. Seed 1 reproduces the recorded original 1.7250755 to 1e-7. Decision: NOT YET PROMOTED (preparation).

## Checks at head f46ec1964

- `rustfmt --check --edition 2021` on both changed Rust files: pass.
- `cargo test -p uor-r4-training --lib read_weighting`: 4 passed. These cover the rank forward against the table, the straight-through backward (q/k/aux bit-identical to flock-softmax, v = Wᵀ dOut), Hamming binarization with rank ≠ softmax ≠ hamming and the setter refusal, and save/load with legacy softmax bytes.
- `cargo test -p uor-r4-training --lib fused_read`: 2 passed. `cargo check -p uor-r4-training --examples`: ok.
- **Negative gates:**
  - Rank forward made to mix the softmax weights: 2 tests fail. Restored: pass.
  - The read's rank flag forced off: the model-level test fails. Restored: pass.
- **CPU smoke:** `train ... select=flock:4:4 read_weighting=hamming_rank|rank` exits 0, and the saved config and report record it.
- **Pod bootstrap at f46ec1964:** release build plus `cuda_stack_ops_parity`: PASS.
- `python3 scripts/check_claim_wording.py`: pass.

## Checklist

- [x] Checks run at this exact head (above)
- [x] Research record: `docs/labs/softmax-free-read-2026-10-10/README.md`
- [x] New entry at the top of `docs/integration/current-state.md`, with **Next:**
- [x] Stale docs: none. The headline is unchanged, so STATUS/ROADMAP/#2028 are unchanged.
- [x] README unchanged (no capability or result change yet)
- [x] `check_claim_wording.py` passes
- [ ] After merge: verify on fresh origin/main, delete the branch (worktree kept for preparation 2), hygiene, comment on #2032

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093682456 — 2026-10-10T04:17:46Z — Casey-allard

Exact-head self-review at `0663e2e67ab501300e2ef26db6263e0c684dc730`: an independent fresh-context review of baa41f163 found one medium issue (a later address/latch/lineage/span setter could silently bypass the rank read). It is fixed in f46ec1964 with a pinned test and a negative gate. The docs commit 0663e2e67 adds the record only. The checks in the description ran at f46ec1964. Docs-only delta since: the claim-wording gate passes. Based on main 5b241d89c.


---
_Comment/review ids in this file: 6093682456_

