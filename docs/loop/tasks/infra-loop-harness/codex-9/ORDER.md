# ORDER for the final November status-refresh readback

Lab: codex, acting as Kimi's cover for this refresh. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row 8. ORDER base: `e52f26595ae9da20a85c99e25e106dfef4ecad4d`. References #3380 and the first cover render in #4071.

## Goal

Finish the same November public refresh by showing the closing disposition of #3380 in the generated views. The status decision belongs to the top session under D68. The cover work under Charter §5.2 reads that decision from its merged roadmap source.

## Cancel probe (run first)

The first render was merged at the base above. The source still said `ready` when this ORDER was prepared. Before rendering, verify that the closure commit is on freshly fetched `origin/main` and that `docs/roadmap/items/docs-status-refresh.md` says `status: done` at that commit. Merge main forward into `kimi/render` and require `git merge-base origin/main HEAD` to equal the fetched main tip. Cancel if the rendered counts, task disposition and four stamps already match at that base.

## Files in scope

- `STATE.md` and `HANDOFF.md`
- `docs/harness/LESSONS.md`
- `AGENTS.md`, only inside the two `loop:lessons-top-ten` markers
- This ORDER and its REPORT under `docs/loop/tasks/infra-loop-harness/codex-9/`

## Done when

Use the harness built from the merged source to render at the full merge-base SHA and check each stamp. Compare the four output files byte-for-byte to the first cover render after accounting for the stamp and three expected data changes: STATE's ready/done counts move from 32/113 to 31/114, HANDOFF's ready heading moves from 32 to 31, and the single `docs-status-refresh` ready row disappears. Confirm that all 156 items are accounted for, including 11 held and zero decision items, and that the blocked-owner rows, lessons text and AGENTS.md outside the marked block match. Verify the roadmap, full gates, per-ORDER REPORT and local PR policy before the top session's exact-head review and protected delivery.

## Owner approval

The [first cover ORDER](../codex-8/ORDER.md) records Casey's appointment: "Yes, for this refresh". Its transcription time was `2026-11-17T21:00:58Z`. This readback completes that same one-time appointment and regenerates the marked lesson block in AGENTS.md. The source of the approval remains the original ORDER merged in #4071.

## Stop and decide if

Report any additional view-data change, any edit outside the four generated projections and this task folder, or a missing closure source to the top session. Record the actual final stamp and comparisons in the REPORT. Issue synchronization and any change to source status belong to the closing documentation unit.
