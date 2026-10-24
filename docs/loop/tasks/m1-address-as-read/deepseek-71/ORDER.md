# ORDER: m1-address-as-read, deepseek-71 (reconcile stale FIT-source wording)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `dc5082f03`. `References #2833`.

## Goal

Two current (non-historical) sentences still say the next FIT "must come from a lab other than deepseek (deepseek has authored text in this sheet)", which the claude-71 update superseded: every lab has now authored sheet text or a panel, so the next FIT is a **fresh-context review** (CHARTER §4.5 counts it as non-author) dispatched by the PM. The lab README also still lists the panel hash as an open prerequisite and omits the now-pinned fresh-V2 BPB stream (merged #3139). Reconcile the wording only: no FIT verdict, bar, tolerance, checkpoint, panel, hold or item status changes.

## Cancel probe (run first)

At `dc5082f03`: `grep -n 'lab other than deepseek' docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md docs/labs/m1-address-as-read/README.md` finds the stale sentences; the sheet's claude-71 bullet already records the fresh-context resolution and #3139 already pins the stream. The task stands. Class: instrument.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/labs/m1-address-as-read/README.md`
- this task folder

## Done when

- The deepseek-38 sentence is marked superseded by the claude-71 fresh-context rule (history kept), and the README lists remaining prerequisites as the held checkpoint SHA, a fresh-context non-author FIT, PM dispatch and the typed release — with panel-v2 and the #3139 BPB pin recorded as done.
- No verdict/bar/tolerance/checkpoint/panel/hold/status changes; `loop report --check`, `loop pr-check --local` and `git diff --check` pass; a review records at the head (independent subagent allowed under the 2026-10-24T05:29Z owner direction when labs are slow).

## Stop and ask if

- A verdict, bar, tolerance, checkpoint, panel or hold would change, or a file outside the list is needed.