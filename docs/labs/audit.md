# Lab audit: every lab's track, peer-reviewed on a schedule

Every three hours, an audit run checks **all labs, the Claude lab included**, against the [session-goal contract](session-goal.md) and [D21](../integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line). The run happens in a fresh context, as a scheduled task on the owner's machine. The auditor reads GitHub only, and it does no lab's work.

- **Independence (each lab is reviewed by a different provider):**
  - The scheduled Claude audit checks the DeepSeek and Codex labs.
  - The **DeepSeek lab checks the Claude lab**, in its own harness, at each of its status cards. It reviews the Claude lab's PRs since its last card against the list below, and posts `PEER AUDIT (DeepSeek → Claude)` on [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028).
  - The scheduled run lists the Claude-lab PRs that have no DeepSeek peer card yet as `awaiting peer audit`. It never passes them itself.

## What each run checks, per lab, since the previous audit card

1. **Status cards:** a card on the lab's milestone issue at least every 90 minutes while it is merging PRs.
2. **Line declaration:** every PR states `Line · count n/3 · headline before → after`.
3. **Count:**
   - no line goes past 3/3 without a pivot card;
   - renaming or splitting a line doesn't reset it.
4. **Preparation limit:** at most two diagnosis, plan or reading PRs per session, each naming the run it prepares.
5. **Decision test:** each PR could change the lab's next step. Flag a PR that couldn't.
6. **Direction:** the work matches the newest `OWNER DIRECTION` on the milestone issue (today: D21 §2–§3).
7. **Pre-registration:** every model run was pre-registered (bar, base, split, cost) on the milestone issue before it started.
8. **Delivery:**
   - each PR merged and verified on `origin/main`;
   - no completed work left unmerged;
   - no stale branches or worktrees;
   - no `current-state.md` conflict left open.
9. **Headline:** the milestone's number before and after the window.

## Output

- **One card per run on the tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028).** It gives, for each lab: the headline number, the PRs in the window, the line and count, and `OK` or the breaches, each with its PR number.
- **For each breach:** one short comment on that lab's milestone issue naming the PR and the rule. The lab answers with a pivot card or a fix.
- **The auditor changes no code, merges nothing and stops nothing.** Directions come only from the owner.
