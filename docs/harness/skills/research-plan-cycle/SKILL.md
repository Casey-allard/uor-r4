---
name: research-plan-cycle
description: Run one D21/D22 cycle on a milestone — status card, one pre-registered model-changing piece, one result PR with KEEP/REJECT, then the line count or a pivot card.
version: 1
harness: 1
kind: research
applies-to: [docs/labs/*/README.md, docs/integration/current-state.md, docs/integration/DECISIONS.md]
when: [start a cycle, next cycle, three negatives, line count, pivot card, blocked card, session goal]
requires: [context-hydration, experiment-prereg, cost-and-compute-admission, evidence-record, panel-scoring]
outputs: [a status card comment on the milestone issue, one pre-registration with FIT, one merged result PR, one dated docs/labs record, one newest-first current-state entry]
forbidden: [a second result PR in the same cycle, updating the count without merging a PR, closing a line without a pivot card, cancelling a healthy run mid-way]
milestones: [m1, m2, m3, m4]
---

## When to use

Use at the start of every lab session and again after each merged PR: a session
runs cycles continuously and never ends on a summary. The contract is
[session-goal.md](../../../labs/session-goal.md) (owner-pasted by `/goal`); D21 and
D22 are in [DECISIONS.md](../../../integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line).

## Do this

1. **Hydrate and fix the number.** Refresh `origin/main`, then read the milestone
   issue and copy its headline into the card:
   `git fetch origin && gh issue view 2029 --repo UOR-Foundation/uor-r4 --comments`
   (M1 #2029, M2 #2030, M3 #2031, M4 #2032 — see
   [milestones.json](../../milestones.json)). Read the newest `OWNER DIRECTION`
   comment first; it overrides older cards inside the issue.
2. **Post the status card** before any compute, and again every 90 minutes or 3
   merged PRs. One line each, no logs:
   `headline <n>/<of> -> target <t> · line <name> · count n/3 · next <step and how it moves the headline>`.
   `gh issue comment 2029 --repo UOR-Foundation/uor-r4 --body-file <card.md>`
3. **Pick ONE model-changing piece** aimed at that headline. A PR is only worth
   opening if the result coming out the other way would change the next step. At
   most two preparation PRs (diagnosis, instrument, reading) per cycle, and each
   names the run it prepares.
4. **Pre-register before compute** with the `experiment-prereg` skill: bar, panel
   noise, base artifact, data split, seeds/rows, steps, stop rule and complete
   cost, posted on the milestone issue, then a `TEST FITNESS: FIT` review on that
   issue. Admit the resources per the `cost-and-compute-admission` skill: CPU work
   on the owner's laptop, GPU work only through `scripts/pod/uor-pod` within the
   caps (≤ 4 pods, ≤ $8/h all labs, no new spending class).
5. **Run, then score on the frozen acceptance panel** (`panel-scoring`). A run is
   scored on the panel it pre-registered against, not on a new metric invented
   afterwards. If the run is healthy, extend it — never cancel it because a
   self-set budget ran out (AGENTS.md, owner direction 6 October 2026).
6. **Merge ONE result PR** with an explicit `KEEP` or `REJECT`, its dated record
   and its current-state entry (`evidence-record`). Every PR body carries
   `Line: <name> · count n/3 · headline before → after`, the exact-head checks
   that ran, and `References #N` unless acceptance is fully met.
7. **Count per D21 as amended by D22.** A merged PR that moves neither the
   headline nor a pre-registered mechanism target (without significant headline
   loss) adds one. A run that meets its own mechanism target without significant
   headline loss is KEEP and iterate and **resets** the count. Only a moved
   headline otherwise resets it; renaming, splitting or re-scoping a line does
   not. Tooling, numerical and infrastructure failures are blockers to fix, not
   negatives, and do not count.
8. **Decide at 3/3** by posting a pivot card that either stops and archives the
   line or names the one decisive model-changing run with its pre-registered bar.
   Stopping a novel mechanism (prime/zeta/R4/H4/icosian geometry, exact addressed
   memory, pointer/copy, read binding, softmax-free reads, protected or discrete
   constructors) needs an **owner decision**: the card proposes, it does not decide.
   A negative closes the configuration — recipe, dose, window, key, objective,
   base, steps — never the mechanism.
9. **Ask the owner and continue elsewhere** for a criterion, panel or grader
   change, a new spending class, or a conflict with another lab. Post the question
   on the milestone issue and stop that line, not the session.
10. **Close the cycle**: verify the merge commit and the delivered tree on fresh
    `origin/main`, delete the branch and worktree, run
    `scripts/storage/uor-hygiene --apply`, then start the next cycle at step 2.

## Output

- One status card on the milestone issue (step 2 format), at cycle start and at
  least every 90 minutes or 3 merged PRs.
- One pre-registration plus its `TEST FITNESS: FIT` review on the milestone issue.
- One merged result PR declaring `Line · count n/3 · headline before → after` and
  `KEEP` or `REJECT`.
- One `docs/labs/<topic>-<YYYY-MM-DD>/README.md` and one newest-first
  `docs/integration/current-state.md` entry with **Next:**.
- At 3/3, one pivot card instead of the next run.

## Do not

- Do not open a second result PR in the same cycle, or split one investigation
  into many small PRs.
- Do not ship a plan, a reading, an instrument, a handover or a "findings" PR as a
  deliverable; those are preparation for a named run.
- Do not change an acceptance criterion, panel, grader or threshold without an
  owner decision.
- Do not declare a next step and then defer it for more reading.
- Do not count a crash, a numerical failure or a missing CLI field as a negative,
  and do not close a mechanism on a configuration result.
