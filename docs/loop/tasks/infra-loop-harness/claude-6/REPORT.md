# REPORT: infra-loop-harness, claude-6 (D59 amended: Claude reviews every loop setup PR)

Lab: claude. Base: `4611538bb` (#2899 merged). Order: [ORDER.md](ORDER.md). The ORDER was committed (`014105585`) before any edit. Its cancel probe found no existing rule giving Claude the review of setup PRs that other labs build.

## What did not work, first

- **D59 had a gap when it merged.** Claude's round-2 review of #2899 found it. Under D52, Codex and DeepSeek build PRs 2–7 and 10–12, and Antigravity builds PR 9. D59 as written would have sent each of those PRs only to fresh-context reviewers of the lab that built it. That would leave out Claude, which leads the architecture and is the one lab that knows the whole setup. The question went to the owner before any such PR existed, and this PR records the answer.
- **The first head left §6 stale.** A fresh-context Claude review of `111435b85` asked for changes. CHARTER §6 still said same-lab review applies to every setup PR, but after the amendment that is true only for the setup PRs Claude builds. The ORDER's scope was widened first (`4fa13b650`), and then the clause was fixed. The same review noticed that this REPORT and the PR body had misquoted the changes.md entry's title. That is also fixed.

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `CHARTER.md` | the D59 sentence in §5.5; the D59 clause in §6 | rules | Whichever lab built a loop setup PR, its exact-head review comes from Claude's fresh-context reviewers (Claude leads the architecture, D52), until the other labs run the loop. No other lab is asked. |
| `docs/rules/changes.md` | a new entry | rules | "21 October 2026: Claude reviews every loop setup PR (D59 amended)". It quotes the owner's chosen answer verbatim and says that Claude worded the two options. |
| `docs/integration/DECISIONS.md` | row D59 | rules | The original text is kept. The amendment is appended in the decision cell, and the turned-down alternative is added. |
| `docs/loop/README.md` | the D59 note in the build plan | docs | The same rule, in the build plan's words. |
| `docs/loop/tasks/infra-loop-harness/claude-6/` | new | docs | This ORDER and REPORT. |

D59's scope (the loop's setup PRs) and its end condition (until the other labs run the loop) are unchanged. D46 is unchanged everywhere else.

## Measurements

None. This PR changes rules only.

## Checks run

The PR body lists the checks run at the head: claim-wording, `xtask links` on the changed files, `roadmap check`, the harness real-repo rule tests, the D59 row's cell count, and the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- **PR 6's review gate.** As planned, it refuses a same-vendor review record. A Claude-built setup PR reviewed by Claude under D59 would fail it. PR 6's ORDER must exempt the setup PRs that D59 covers, or record that the gate applies only once the labs run the loop.
- **D59's end.** D59 ends when the other labs run the loop. In today's plan that happens after PR 12.

## Plain summary

- Every piece of the loop's setup is now reviewed by Claude, the architecture lead, whichever lab built it.
- The other labs still don't review the setup until they are running it.
