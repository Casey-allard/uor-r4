# ORDER: infra-loop-harness, kimi-29 (roadmap sync records: m7-studio-self-host-page-assets ↔ #3075)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item records, as with kimi-22 and kimi-27).
Base: `7bbaadcdd`.

## Goal

The post-#3072 `xtask roadmap sync --apply` created issue #3075 for `m7-studio-self-host-page-assets`, updated #2868 (body, labels; the merge's `Closes #2868` already closed it) and refreshed the milestone #2035 checklist. The rendered records on main still say "not yet synced" for the item. This unit lands the sync's written outputs: `docs/roadmap/issues.json` gains the `"m7-studio-self-host-page-assets": 3075` mapping and `docs/roadmap/README.md` links #3075 in the ready table and the M7 table.

## Cancel probe (run first)

Class docs/PM (host-only). At `7bbaadcdd`: `grep -n "not yet synced" docs/roadmap/README.md` matches the new item's rows while issue #3075 exists (created by the applied sync). Task stands.

## Files in scope

- `docs/roadmap/issues.json`
- `docs/roadmap/README.md` (shared rendered file)
- this task folder

## Done when

- `issues.json` maps `m7-studio-self-host-page-assets` to 3075 and the README links `#3075` in both rows where it said "not yet synced".
- `cargo xtask roadmap render --check` and `roadmap check` pass; `loop report --check`, `loop pr-check --local` and `loop gates --full` pass.
- The PR body carries `References #2904`, `References #3075` and `References #2868` (nothing closes; #2868 is already closed by the #3072 merge).

## Stop and ask if

- The sync plan would have created, closed or edited anything beyond what the applied run recorded (create #3075, update #2868, update #2035) — that would mean the plan drifted and the owner should see it first.
