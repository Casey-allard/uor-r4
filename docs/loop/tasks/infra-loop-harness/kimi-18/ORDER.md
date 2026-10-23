# ORDER: infra-loop-harness, kimi-18 (the umbrella's scope gains docs/roadmap/issues.json)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6 (the render-review close-out chain this continues, as with kimi-17 and deepseek-44).
Base: `7f0f0710c`.

## Goal

Append `docs/roadmap/issues.json` to the umbrella item's `scope:` so the roadmap sync's own output file has a home. After #3040 merged, `xtask roadmap sync --apply` updated GitHub (#3044 created, #3036 closed) and rewrote `docs/roadmap/issues.json` plus `docs/roadmap/README.md`. The README is a shared file and always allowed; `issues.json` sits outside every item's scope, so `loop pr-check`'s scope rule — which reads the item at the merge base — refuses the sync commit. This widening lands first, as for #3006/#3010/#3020 (kimi's PM call on #2028); the sync commit follows as kimi-19.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base: `docs/roadmap/items/infra-loop-harness.md`'s `scope:` does not list `docs/roadmap/issues.json`, and no other item's scope covers it (`grep -l issues.json docs/roadmap/items/*.md` finds only roadmap-sync docs references, no scope entry). Task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line gains one entry)
- this task folder

## Done when

- The diff is exactly the one scope entry plus this task folder.
- `xtask roadmap check` and `xtask roadmap render --check` pass; `loop report --check` and `loop pr-check --local` pass with no FAIL; a fresh-context review from another vendor's lab is requested.

## Stop and ask if

- Anything beyond the one scope entry needs to change for the gate to pass.
