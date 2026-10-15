# #2237 — DeepSeek lab handoff: the corrective goal, the rules easy to get wrong, and the next slice (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T03:32:37Z
- **Updated:** 2026-10-13T04:16:37Z
- **Closed:** 2026-10-12T03:33:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2237
- **Merged:** 2026-10-12T03:33:42Z
- **Merge commit:** 2c8c838777dc463f3ad0aafcb94f5b4240047434
- **Base / Head:** main / UOR-Foundation:deepseek/handoff-doc
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The lab's paste-in session goal, landed in the repository** rather than left in a local file — main is the source of truth, and the next context should not depend on one machine's home directory.

**What it carries.** The **D22 corrective goal in full** (what went wrong, the binding rules, the four orders, the cycle contract, MAY/MAY-NOT, the finish line). Then the **project rules a fresh context gets wrong**, grouped so they cannot be skimmed:

- **Delivery** — verify the *actual merge commit on fresh `origin/main`* and the delivered content; delete branch and worktree; hygiene after merge; and the fact this lab discovered by breaking it: **CI does not compile `--features cuda`**, so a pod bootstrap at the merged sha is the only check on that path.
- **Pods** — status first, lease what you use, **never leave an idle pod**, verify inputs before queueing, **read the first fit's config before queueing the rest**, delete at the last fit, never fall back to the laptop for GPU work.
- **Local machine** — CPU-bound only, **no judge runs locally**, and **never a blanket worktree-removal loop** (the mistake that deleted a peer's worktree; delete only paths this session created, checked by name).
- **Evidence discipline** — read the compile or config **before** the claim, and verify on the artifact that **contains** the change: `--ref` fetches from GitHub, which is exactly how the retracted "12×" kernel claim happened.
- **Mechanisms** — briefs, `TEST FITNESS: FIT` before compute, and **the owner's tangent-null rule**: a permuted control destroys the structure and cannot separate "uses the geometry" from "uses *some* address".
- **Metric policy** — **tokens/s on the product path** beside panel acceptance; the panel is the quality gate; BPB is a diagnostic.

**Then the verified state and the exact next slice**: the geometric read (wrong-value 49 → 32 of 120, ahead on every seed, below its own margin), read binding's exhausted budget lever, the prepared invariant-addressed read (address primitive, tangent/normal movements, spanning tree, pure routing decision — **nothing called by the read yet**), and the one remaining slice with **both bar clauses** and the nine-fit plan. Plus the pre-existing `joint_campaign` failures (reproduced at `42dcb3828`) and the owner's two open questions.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (handoff documentation; the mechanism's own numbers are unchanged and no compute was spent)

References #2029
PR; sleep 270; gh pr checks 2>&1 | grep -v skipping | grep -vc pass; gh pr merge --squash --delete-branch=false 2>&1 | tail -1


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

