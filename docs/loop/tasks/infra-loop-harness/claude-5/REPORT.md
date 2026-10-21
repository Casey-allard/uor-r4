# REPORT: infra-loop-harness, claude-5 (D59 recorded: no lab review of the loop setup)

Lab: claude. Order: [ORDER.md](ORDER.md), committed before any edit, with its cancel probe: no D59 or similar exception existed.

## What did not work, first

- **The direction was missed the first time.** The owner's first statement ("none of the other labs are onboarded yet so having them review the onboarding will be problematic - anyway, go") was a direction. Claude read it as an open question and asked Kimi to review loop PR 1 (#2898). The owner corrected this. The requests were withdrawn on #2898 and #2028, and this PR records the direction as D59.
- **The scope was too narrow.** It did not cover the AGENTS.md decision-index label, which a new row makes stale. The ORDER was widened in a commit before that one-line edit, not after it.
- **The first head gave two different end conditions.** A fresh-context Claude review of `3b3464780` asked for changes:
  - **End condition (medium).** changes.md and this REPORT ended D59 at the onboarding PR (PR 9). The charter and the row ended it "when the labs are onboarded". The build plan, meanwhile, switches labs on only after PR 12. So under one reading, loop PRs 10–12 would have needed review from labs that are not running the loop, which is the case the owner forbade.
  - **Undefined term (low-medium).** changes.md also added "the loop docs PRs", which no other place defined.
  - **Charter wording (low).** §5.5 and §6 read against each other without an "exception to D46" clause.
  - **Wrong base (low).** The ORDER named the pre-rebase base.
  - **Stale loop page (low).** The loop page's build plan still sent every review to another vendor.
- **All five are fixed at the next head.** The ORDER was widened first, in its own commit.
  - All five places now share one end condition in the owner's own word: until the other labs are *running* the loop.
  - The undefined phrase is gone.
  - §5.5 opens with "As an exception to D46", and §6's D46 entry names the D59 exception.
  - The ORDER gives the true base.
  - The build plan carries one D59 note.

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `CHARTER.md` | 1 sentence in §5.5, plus its citation; one clause on §6's retired D46 entry | rules | As an exception to D46, until the other labs are running the loop, the loop's setup PRs are reviewed by fresh-context reviewers of the authoring lab, never by the other labs (D59). |
| `docs/loop/README.md` | 1 note in the build plan | docs | The setup's own review follows D59; the plan's cross-vendor reviews start once the other labs run the loop. |
| `docs/rules/changes.md` | a new entry | rules | 21 October 2026: the owner's two statements, verbatim, and what they decide |
| `docs/integration/DECISIONS.md` | row D59 | rules | the decision, its reason, the alternative turned down, and the cost to undo it |
| `AGENTS.md` | 1 label | rules | Decision index D0–D59 |
| `docs/loop/tasks/infra-loop-harness/claude-5/` | new | docs | this ORDER and REPORT |

## Measurements

None. This PR changes rules only.

## Checks run

The checks run at the head are listed in the PR body: claim-wording, `xtask links` on the changed files, `roadmap check`, the harness real-repo rule tests, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

D59 ends when the other labs are running the loop. The build plan in `docs/loop/README.md` switches labs on one at a time after PR 12, so every setup PR in today's plan falls inside D59. Once the other labs run the loop, D46's cross-vendor review applies to every code and model PR. If the owner wants a different end point, such as each lab's own switch-on, that is the owner's call.

## Plain summary

- The owner's direction is now written into the charter: the other labs don't review the loop's setup while they aren't running it.
- A fresh-context reviewer from the authoring lab reviews the setup instead.
