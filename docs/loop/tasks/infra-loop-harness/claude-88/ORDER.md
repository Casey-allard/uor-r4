# ORDER: infra-loop-harness, claude-88 (register the W2 code-utilization item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in kimi-27). Base: `9ae329d4e`.

## Goal

W2 (`m1-w2-capacity-capture`, #2845, merged as `9ae329d4e`) classified the seed-1 Q4 read code as H_mixed. That is row 3 of its sheet's §4.7 table, whose next step is "Code-utilization/capacity diagnostic; do not change the learning law yet". No roadmap item carries that diagnostic, so `loop next` cannot offer it.

This unit adds `docs/roadmap/items/m1-w2-code-utilization.md` and nothing else. Its fields:
- `scope:`, because unscoped items are never offered;
- `depends_on: [m1-w2-capacity-capture]`, `needs_gpu: false`, `done_class: host`;
- `notebook:` pointing at `m1-discrete-attention`.

The unit re-renders the roadmap page and applies `cargo xtask roadmap sync`, so the item has its issue before work starts (§5.3), and records the number in `docs/roadmap/issues.json`.

The pre-registration sheet is the next unit, under the new item's own task folder. pr-check requires an ORDER's item to exist at the merge base, so the sheet cannot ride in this PR.

## Cancel probe (run first)

Class: docs (host only). At `9ae329d4e`:
- `grep -l "code-utilization\|code utilization" docs/roadmap/items/*.md` finds no item.
- `docs/roadmap/issues.json` has no `m1-w2-code-utilization` key.
- `m1-w2-capacity-capture` reads `status: done`.

The unit stands.

## Files in scope

- `docs/roadmap/items/m1-w2-code-utilization.md` (new)
- `docs/roadmap/issues.json` (the synced issue number)
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- `cargo xtask roadmap check` passes, and `cargo xtask roadmap render --check` matches the committed page.
- The issue for `m1-w2-code-utilization` exists, and its number is in `issues.json`.
- `uor-harness loop next` on the merged tree can offer `m1-w2-code-utilization`: ready, scoped, and its only dependency done.
- `loop report --check` and `loop pr-check --local` pass. Scope WARNs are advisory under D76; any WARN is named in the REPORT.
- A fresh-context review at the exact head counts (D66).
- The REPORT says `References #2904`. The new item's issue stays open: this unit registers the item and does not do it.

## Stop and ask if

- The sync plan touches more than the new issue, #2845's body and the #2029 checklist.
- The item would change a bar, a status or a milestone rather than add one ready item.
