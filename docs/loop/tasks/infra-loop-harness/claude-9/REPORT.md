# REPORT: infra-loop-harness, claude-9 (three D59 wording leftovers)

Lab: claude. Base: `72071ac75` (#2902 merged). Order: [ORDER.md](ORDER.md), committed (`5ad8e1a77`) before any edit. Its cancel probe found all three leftovers still present.

## What did not work, first

- **#2902 left three wording gaps, and its fresh-context review named them:**
  - Line 45 of the loop page stated the review rule with no setup exception.
  - The parenthetical "(another vendor's, or Claude's …)" listed another vendor first. That could be read as letting another vendor approve a setup PR, which PR 6's gate refuses.
  - `AGENTS.md` item 6, the first delivery summary a new agent reads, named only "another vendor's lab".
- **`AGENTS.md` is an owner path.** The edit to it rests on the owner's message, quoted verbatim in the ORDER.
- **The first head's item 6 had the same ordering problem it was meant to fix.** A fresh-context Claude review of `dc8e14329` asked for changes. The first head placed the setup exception after the doc-only self-review clause, without "whichever lab built them". An agent reading only `AGENTS.md` could then have self-reviewed a doc-only setup PR built by a lab other than Claude, which CHARTER §5.5 rules out. The fix puts the setup exception first, so it wins, and limits the self-review clause to "other" doc-only or generated PRs. The same review found that the queue rule used a comma where step 12 uses a semicolon; it now uses a semicolon too.

## What changed

| File | Class | What |
|---|---|---|
| `AGENTS.md` | rules summary (owner path) | Item 6's exceptions now read: "loop setup PRs, whichever lab built them, until the other labs run the loop: Claude's fresh-context reviewers, D59; other doc-only or generated PRs: a fresh-context self-review". The setup exception comes first, so it takes precedence. Item 6 is still one line, and the file is still 70 lines. |
| `docs/loop/README.md` | docs | Line 45 adds that during the setup, Claude's fresh-context reviewers review the setup PRs instead (D59). Step 12 and the supervisor queue rule now read "Claude's for a setup PR under D59; otherwise another vendor's". |
| `docs/roadmap/items/infra-loop-pr07c-queue-board.md` | docs | The done-when's parenthetical has the same Claude-first order. The planted fault is unchanged and still matches its row. |
| `docs/loop/tasks/infra-loop-harness/claude-9/` | docs | This ORDER and REPORT. |

D46, D55 and D59 are unchanged. Every edit restates D59 where a reader meets the review rule.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head: claim-wording, `xtask links` on the changed files, `roadmap check` and `render --check`, the harness real-repo rule tests, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- **The doc-only allowance outside the setup.** Line 45, step 12's "otherwise another vendor's" and PR 7c's done-when still leave out D46's same-lab fresh-context review for doc-only and generated PRs that are not setup PRs. "The review record" paragraph grants that review. The omission predates this PR, which neither widens nor narrows it.

- **The roadmap sync.** The 14 loop items still need it to become issues.
- **The wording of CHARTER §6.** Three reviewers read "except for the loop setup PRs that Claude builds" as contradicting §5.5. It is accurate, because same-lab review survives only for Claude-built setup PRs, but clearer wording is the owner's call.

## Plain summary

The three places a reader could still miss the setup exception now name it, with Claude first.
