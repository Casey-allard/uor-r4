# ORDER: infra-loop-harness, kimi-35 (reconcile the address roadmap record)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8 (notes/roadmap migration). Base: `42f377598`.

## Goal

Update the m1-address-as-read roadmap item to match the delivered consumer, registered oracle bar, frozen panel-v2 and pinned fresh-V2 stream. Leave the item `ready` and name only the remaining checkpoint, FIT/dispatch/owner approval and evaluation work. This is a PM record repair, not a mechanism or milestone status change.

## Cancel probe (run first)

At `42f377598`, the item still says address-as-read was never built, the oracle bar is not registered, and training waits for Casey to type a release. Those statements contradict the merged sheet and current CHARTER. The task stands.

## Files in scope

- `docs/roadmap/items/m1-address-as-read.md`
- `docs/loop/tasks/infra-loop-harness/kimi-35/**`

## Done when

- What-is-missing names the delivered address consumer/finite controls and the remaining held checkpoint, fresh-context FIT, PM dispatch and evaluation; it no longer says nothing consumes the store.
- Step 4 quotes the registered 27/40, 7/20, 20/20, zero-substitution and 0.01 BPB bar with frozen panel-v2, and step 5 uses owner approval in chat rather than a typed release.
- Frontmatter status/scope, the done-when, every registered value and every hold remain unchanged.
- Report/local PR checks, full gates and diff-check pass; a fresh-context doc-only review under CHARTER §5.5 covers the exact head. Roadmap issue-body sync is a separate post-merge PM action.

## Stop and ask if

A status, bar, checkpoint, panel, tolerance, FIT verdict, hold or done-when must change, or a generated roadmap file outside this scope is required.
