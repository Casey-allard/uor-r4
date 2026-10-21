# REPORT: infra-loop-harness, claude-11 (D52 amended: Claude builds the loop setup while it runs alone)

Lab: claude. Base: `e61746628` (#2918 merged). Order: [ORDER.md](ORDER.md), committed (`62aafef8e`) before any edit. Its cancel probe found no "runs alone" text anywhere in scope, and 12 items still named another lab as builder.

## What did not work, first

- **The team split assumed five running labs.** D52 split the setup across Codex, DeepSeek and Antigravity. The owner then said only Claude would run at first. If that had stayed unrecorded, 12 setup issues (#2905–#2917, except PR 8) would have named builders that are not running.

## What changed

| File | Class | What |
|---|---|---|
| `docs/integration/DECISIONS.md` | rules | Row D52 keeps its text. Appended: "Amended 21 Oct by the owner: while Claude runs alone, Claude builds every loop setup PR, and each other lab's client adapter is added when that lab joins." The turned-down alternative is added. |
| `docs/rules/changes.md` | rules | New entry for 21 October 2026, D52 amended. It quotes both owner messages verbatim and says Claude worded the chosen answer. |
| `docs/loop/README.md` | docs | The build plan's Teams line adds the amendment. |
| `docs/roadmap/items/infra-loop-harness.md` | docs | The team-split step adds the amendment. |
| 12 loop items (PR 2–7, 7a–7c and 9–12) | docs | Their team step now reads "Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to …". PR 8 was already Claude's. |
| `docs/roadmap/items/infra-loop-pr09-client-adapters.md` | docs | Done-when: the Claude Code adapter and its receipt ship first. Each other client's adapter and receipt are added when that lab joins. |
| `docs/loop/tasks/infra-loop-harness/claude-11/` | docs | This ORDER and REPORT. |

D59, D46 and the rest of D52 are unchanged: the GPU caps, the owner as last fallback, and the kill switch. The charter does not state the team split, so it is untouched. No item's title, status or dependencies changed, so the rendered roadmap and the issues' labels stay as they are. The issue bodies need a sync after merge, because the team steps changed.

## Measurements

None. This PR changes docs only.

## Checks run

The PR body lists the checks run at the head:
- claim-wording;
- `xtask links` on the changed files;
- `roadmap check` and `render --check`;
- the harness real-repo rule tests;
- the per-file sweep counts;
- the D52 row's cell count;
- the local copyright check against the Loop Kit.

## Skills used

None of the private kit's skills.

## Still open

- **Issue bodies.** After merge, a roadmap sync (dry run first) brings the 12 issues up to date.
- **Later adapters.** PR 9 (#2914) can close with the Claude Code adapter alone. Each later adapter and its receipt are added as part of that lab's onboarding, after its probes on #2028. No separate item tracks them yet.
- **A review fix.** A fresh-context review of `c8da6cee3` found that the parent item named only half of the amendment. It now also says that each other client's adapter is added when that lab joins. Row D52's Reason cell gains the amendment's reason, and one REPORT sentence no longer claims what the unbuilt claim rules would do.
- **Loop PRs 2 and 4** (#2905, #2907) start next, each with its own ORDER.
- **A rule for after the setup.** D59 covers only setup PRs. Once the loop runs and Claude is still the only lab, D46 has no second vendor to review code and model PRs. The owner has not decided this yet; it needs a rule before the first non-setup code PR.

## Plain summary

While Claude is the only lab running, Claude builds the whole loop setup. The other labs' client adapters are added when each lab joins.
