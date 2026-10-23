# ORDER: infra-loop-harness, kimi-19 (commit the roadmap sync state after #3040)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6 (the render-review close-out chain, as with kimi-17, kimi-18 and deepseek-44).
Base: `0af044c7f`.

## Goal

Commit the roadmap sync state that `xtask roadmap sync --apply` produced after #3040 merged: `docs/roadmap/issues.json` gains the three mappings it wrote (infra-bench-inbox-rule #2955, infra-loop-render-review #3036, loop-render-review-hardening #3044) and `docs/roadmap/README.md` re-renders with the three issue links replacing "not yet synced". The GitHub side is already applied (#3044 created, #3036 updated and closed); this PR lands the repo side so the next item PR's re-render stays clean. kimi-18 (#3047) added `docs/roadmap/issues.json` to the umbrella's scope, so the scope gate now admits this diff.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base: `docs/roadmap/issues.json` has no `loop-render-review-hardening` mapping and the README shows "not yet synced" for it and for the two 2026-10-23 items. Task stands.

## Files in scope

- `docs/roadmap/issues.json` (the sync's own output; in the umbrella's scope since #3047)
- `docs/roadmap/README.md` (re-rendered; a shared file)
- this task folder

## Done when

- The diff is exactly the three issues.json mappings and the three README issue links, produced by `xtask roadmap sync --apply` with no hand edits.
- `xtask roadmap check` and `xtask roadmap render --check` pass; `loop report --check` and `loop pr-check --local` pass; a fresh-context review from another vendor's lab is requested.

## Stop and ask if

- The sync wants to create, update or close any issue beyond the three already applied (that plan is owner-visible work, not housekeeping).
- The diff shows anything beyond the six expected line changes.
