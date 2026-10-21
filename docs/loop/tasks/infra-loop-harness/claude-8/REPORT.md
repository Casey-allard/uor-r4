# REPORT: infra-loop-harness, claude-8 (D59 carried into the remaining loop items and steps)

Lab: claude. Base: `500c61c2c` (#2901 merged). Order: [ORDER.md](ORDER.md), committed (`3e21aadd7`) before any edit. Its cancel probe found D59 in none of the 13 items, and not in the loop page's steps 10 and 12 or PR 7c's row.

## What did not work, first

- **The build items were written before D59 existed.** Before this PR, each of the 12 loop items (PR 2–5, 7a–7c and 8–12) told its builder to "ask a lab of a different vendor for the exact-head review". PR 8's item went further and named Kimi as its reviewer. A builder following them would have asked a lab that is not running the loop to review the setup. That is the case the owner forbade on 21 Oct.
- **Two statements in claude-7's REPORT need correcting.** claude-7's REPORT is not edited, because the loop page says an ORDER and REPORT are "never changed after merge". The corrections are recorded here instead, as #2901's delta check asked:
  1. It says the end switch "is now owner-only". That is what the spec requires, not the current state. The switch becomes owner-only only once the owner lists its path in `docs/harness/owner-paths.json`. Until then, PR 6's gate treats D59 as in force (low 1, fixed in this PR).
  2. It says steps 10 and 12 "both feed `loop queue` in PR 7c". Only step 12 does. Step 10's tool is `loop review`, which PR 6 builds.

## What changed

| File | Class | What |
|---|---|---|
| 12 loop items (`infra-loop-pr02` … `pr12`, except `pr06`) | docs | **Team step:** "This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59)". PR 8's step also says Kimi does not review it while the other labs are not running the loop. **Review step:** put `Setup row: <row>` in the ORDER and ask Claude for the exact-head review, where `<row>` is the item's own build-plan row. **Delivery rule:** names D59. Each file now mentions D59 three times and its own row once, and has no other-vendor review wording left outside PR 7c's done-when. |
| `docs/roadmap/items/infra-loop-pr07c-queue-board.md` | docs | **Done-when:** the queue needs the APPROVE that PR 6's gate accepts, and no later CHANGES that the gate counts. **Planted fault:** "a later CHANGES record blocks queueing, unless it comes from a lab other than Claude on a setup PR (D59)". |
| `docs/roadmap/items/infra-loop-harness.md` | docs | The team-split step and the delivery rule name the D59 exception, with Kimi included among the labs not asked. "Why it matters" and "What is missing" also name it. The second had described today's practice as "a review from another vendor"; the sweep's own check found both, and the ORDER was widened before the edit (`f2fd3f11f`). |
| `docs/roadmap/items/infra-loop-pr06-review-records.md` | docs | Delta-check low 1: until the owner has listed the switch's path, the gate treats D59 as in force whatever the file says. |
| `docs/loop/README.md` | docs | **Step 10:** names Claude for setup PRs under D59. **Step 12:** queues on the approval the policy gate accepts. **The advisory-policy paragraph's supervisor queue rule:** says the same thing; the sweep's own check found it, and the ORDER was widened first (`4c6209362`). **PR 7c's row:** carries the same planted fault as its item, word for word. |
| `docs/loop/tasks/infra-loop-harness/claude-8/` | docs | This ORDER and REPORT. |

D46, D55, D59 and D52's team assignments are unchanged. No item's title, status or dependencies changed, so the rendered roadmap does not move. No loop item is synced to an issue yet: `docs/roadmap/issues.json` has no `infra-loop` key.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head:
- claim-wording;
- `xtask links` on the changed files;
- `roadmap check` and `roadmap render --check`;
- the harness real-repo rule tests;
- the per-file D59 and `Setup row:` counts;
- PR 7c's fault matching between item and row;
- the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Checked and left as they are

The sweep's own check also found five other-vendor review lines that name no D59. Each was read, and none needs a change:
- **Loop page, line 45.** It states the steady-state rule for the loop once it runs.
- **Loop page, line 202.** It records what PR A changed in the charter. That is history.
- **Loop page, lines 206 and 208.** They are the build-plan intro, and the D59 note right below them (line 210, from #2899) already qualifies them.
- **Parent item's done-when.** It describes the end state, after every lab is onboarded, when D46 applies again.

## Still open

- **AGENTS.md item 6.** It still summarizes delivery as "exact-head review by another vendor's lab", without the D59 setup exception. AGENTS.md summarizes and never overrides the charter, so the charter governs. A one-line pointer there would help builders who read AGENTS.md first, but that file was outside this ORDER.
- **The roadmap sync.** The 14 loop items still need it, so they become issues a lab can claim.

## Plain summary

- Every loop build step now tells its builder the same thing the charter does: during setup, Claude reviews the PR, whichever lab built it.
- No other lab, Kimi included, is asked to review the setup.
