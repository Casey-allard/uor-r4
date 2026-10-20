# #2208 — Starling read rung 3: a content gate does not lift recall (source kept ~50%); value path suspect; row 3, line 3/3 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T19:09:22Z
- **Updated:** 2026-10-11T19:14:12Z
- **Closed:** 2026-10-11T19:10:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2208
- **Merged:** 2026-10-11T19:10:30Z
- **Merge commit:** 0517ea8bba788638cae02bfb33979934855f9152
- **Base / Head:** main / UOR-Foundation:claude/starling-content
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count **3/3** · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged). A pivot card follows on #2032.

**Result: row 3.** A content gate on the kept 7 does not lift recall.
- **Pre-registration:** [6112273931](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112273931).
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112290954), adversarial: completion and decision value.
- **Blocker fix:** [6112460879](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112460879).

| Arm (mqar, 3 seeds) | Recall | Hit rate: final |
|---|---:|---:|
| L, L2 read (rung 1) | 0.617 | — |
| SA7, address gate (rung 2) | 0.011 | 0.432 |
| **SC7**, content gate | **0.010** | 0.561 |
| **SC7w**, content gate | **0.008** | 0.465 |

The source is kept about half the time, but neither gate turns it into a recalled value. The value path (or per-head availability of the source) is suspect.

**What it adds:**
- `read_route=starling:M:K:content`: the kept sources are gated by the read's own q·k/√d. No new parameters; the helper still teaches the addresses.
- A tanh-form sigmoid on content routes. The exp form overflowed and diverged 5 of 6 first-attempt runs.

**Tests:** content changes the logits; the q/k gradients match finite differences; the address gradient survives; save/load; finite gradients at β = 1e4. Both fixes have negative gates.

Record: `docs/labs/starling-content-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6112636443 — 2026-10-11T19:09:24Z — Casey-allard

**Exact-head checks at `84675c82e471ff15bb6766b0f80248311d5f1100`** (laptop, this worktree's own target):
- `cargo test --release -p uor-r4-training --lib -- starling`: **6 passed**.
- **Negative gates** (shown during development): never computing the content scores fails the content test; the exp-form gate fails the finite-gradient check.
- `cargo build --release --example mqar-bench --example geometric-stack`: PASS. `check_claim_wording.py`: PASS.
- `cargo fmt --check`: the only `geometric_stack.rs` diffs are the read-binding lines already on main, not in this PR's hunks.
- **CUDA:** the pod bootstrap at 2675527d5 passed `cuda_stack_ops_parity`; all 6 scored runs reached step 1,800 on CUDA.

### review 5484547959 — 2026-10-11T19:09:25Z — Casey-allard — COMMENTED

**Self-review at exact head `84675c82e471ff15bb6766b0f80248311d5f1100`.**
- The address route is bit-unchanged; the content route alone uses the tanh-form sigmoid.
- The scoring follows the pre-registered rows, with the bases rescored from rung 2's archived logs.
- The record carries every FIT nit: the leniency of the hit-rate metric, the address freeze after α reaches 0, and the meaning of the row-1 bar.
- The count is reported as 3/3, with the pivot card next.

APPROVE (self-review is allowed under the owner's exact-head rule).

### comment 6112690171 — 2026-10-11T19:14:11Z — Casey-allard

**Correction to the checks comment above.** I wrote that the `cargo fmt --check` diffs were only in the read-binding lines already on main. That was wrong: three were in this PR's own new lines (the content-gate call site and two test hunks).
- Fixed in `4bb054fa4` by formatting only those hunks.
- `cargo fmt -p uor-r4-training --check` now reports, in `geometric_stack.rs`, only the two read-binding lines already on main (around 4966 and 9772).
- `cargo test --release -p uor-r4-training --lib -- starling`: 6 passed at this head.
- Re-reviewed at this head: whitespace only, no logic change. APPROVE.


---
_Comment/review ids in this file: 6112636443, 6112690171, 5484547959_

