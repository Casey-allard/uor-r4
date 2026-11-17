# ORDER for HANDOFF decision ownership

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). References #3380. Base: `16b088064183fce67458e4e9d6bdae916dd096ea`.

## Cancel probe

The M2 multi-turn item assigns its decision to a lab through open_to, while render_handoff overrides it to owner. The registered roadmap already prints the item's open_to value. The parent session requested a source repair during the November public documentation refresh.

## Scope and done check

Repair the blocked-table renderer and its fixture expectations in loop_render.rs. Exercise lab and explicit owner decisions, held items and missing ownership. First show the existing renderer fails the new test. Then run the focused render test target on the repaired source and check the real M2 item in a generated preview. Leave tracked generated views for their normal publication workflow. Stop if the fix would change an item's actual assignment, decision bar, or other tracker fields.
