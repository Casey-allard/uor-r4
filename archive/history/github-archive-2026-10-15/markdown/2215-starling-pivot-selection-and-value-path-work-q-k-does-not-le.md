# #2215 — Starling pivot: selection and value path work; q.k does not learn to pick the source by content; owner decision requested (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T21:51:56Z
- **Updated:** 2026-10-11T21:53:13Z
- **Closed:** 2026-10-11T21:53:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2215
- **Merged:** 2026-10-11T21:53:02Z
- **Merge commit:** 02ca05532b30061f6f10d83b509a3d4597393f99
- **Base / Head:** main / UOR-Foundation:claude/starling-oracle
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 3/3 (pivot; owner decision requested) · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**The pivot record.** The geometric selection works, and so does the value path. What fails is learning q·k to pick the source among the kept candidates, which needs a competitive gradient.
- **Pivot card:** [6112716462](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112716462).
- **Owner decision requested:** [6114147704](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114147704). The recommendation is a training-only softmax surrogate; the runtime read stays softmax-free.

Every GPU step was pre-registered and adversarially reviewed (FIT); the total was under $0.50.

| Step | Result |
|---|---|
| D2, velocity (laptop) | The turn rₜ is mostly token identity, so velocity routing is dropped |
| D1, oracle | Recall 1.000 with the source at a fixed slot: the value path works (corrected scope) |
| Shared kept set | Per-head hit rate 0.61–0.72, recall 0.012 |
| Content-ranked | Source ranked first when kept: 0.03–0.29 (about 2× chance at best) |

**Code (opt-in, tested):**
- \`read_route=starling:M:K[:content][:shared][:ranked]\`;
- thread-local oracle and top-match diagnostics;
- the mqar-bench oracle and top-match rates, and positions-scored counting the content gate;
- \`examples/velocity-info.rs\`.

**Tests:** 10 starling tests plus \`content_order\`. Negative gates for the union, the re-rank and the sort direction.

Record: `docs/labs/starling-pivot-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6114200585 — 2026-10-11T21:51:57Z — Casey-allard

**Exact-head checks at `a083b79fc385b93ee249df465cf899bfada0085e`** (laptop, this worktree's own target):
- `cargo test --release -p uor-r4-training --lib -- starling content_order`: **10 passed**.
- **Negative gates**, run during development (#2032): skipping the union, skipping the re-rank, flipping the sort, and never computing the content scores each fail their test.
- `cargo build --release --example mqar-bench --example velocity-info --example geometric-stack`: PASS.
- `check_claim_wording.py`: PASS.
- `cargo fmt -p uor-r4-training --check`: the only diffs in `geometric_stack.rs` are the read-binding lines already on main (around 5136/9942). This PR does not touch them (`git diff origin/main | grep -c competing` = 0); `mqar-bench.rs` and `velocity-info.rs` are clean.
- **CUDA:** every pod bootstrap on this branch passed `cuda_stack_ops_parity`, and every scored run reached step 1,800.

### review 5485132813 — 2026-10-11T21:51:58Z — Casey-allard — COMMENTED

**Self-review at exact head `a083b79fc385b93ee249df465cf899bfada0085e`.**
- All new routes and diagnostics are opt-in; the default and existing routes are unchanged.
- The oracle and the records are thread-local and cleared on every path.
- The record states D1's corrected scope and the leniency of each metric.
- No KEEP is claimed. The line stays at 3/3 pending the owner's decision.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6114200585, 5485132813_

