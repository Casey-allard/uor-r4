# REPORT: infra-loop-harness, kimi-18 (the umbrella's scope gains docs/roadmap/issues.json)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6 (the render-review close-out chain, as with kimi-17 and deepseek-44).
Base: `7f0f0710c`. ORDER: [ORDER.md](ORDER.md), committed first (`710bf5012`).

## What did not work, first

- The roadmap sync commit was tried first on this branch and `loop pr-check --local` failed its scope rule: `docs/roadmap/issues.json` lies outside every item's `scope:`, and the rule reads the item at the merge base, so widening the scope in the same PR cannot fix it (`crates/uor-r4-harness/src/loopprcheck.rs:425`). `docs/roadmap/README.md` needed no entry — it is a shared file (`crates/uor-r4-harness/src/loopclaim.rs:69`).
- The sync's first write happened in bench-main and I reverted it there with `git checkout --` before realising the ORDER/commit-order flow needed its own worktree; the sync output is regenerable from GitHub state at any time, so nothing was lost.
- The item rename `infra-loop-render-review` → `loop-render-review` is formally dropped (note on #3036): the item is done and closed, and no gate benefit is left.

## What changed

| Commit | What |
|---|---|
| `710bf5012` | The ORDER, alone. |
| `da7bad323` | `docs/roadmap/items/infra-loop-harness.md`: `docs/roadmap/issues.json` appended to `scope:`. |
| this REPORT's commit | This REPORT. |

## Measurements

- Class **instrument** (docs metadata; no GPU, no model compute), matching the ORDER's cancel probe.
- Diff size outside this task folder: 1 file, 1 line changed.

## Checks run at the head

- `xtask roadmap check`: 90 items OK.
- `xtask roadmap render --check`: current (the scope line is not part of the render).
- `uor-harness loop report --check` and `loop pr-check --local`: see the PR body; expected all-PASS now that the changed item file matches the umbrella's own `docs/roadmap/items/*.md` glob.

## Skills used

- scope-guard: one scope line, no more.

## Still open

- **kimi-19:** the sync commit itself — `docs/roadmap/issues.json` (+3 mappings: infra-bench-inbox-rule #2955, infra-loop-render-review #3036, loop-render-review-hardening #3044) and the re-rendered README — follows once this widening merges, under the same chain.
- Cards filed earlier this cycle: #3041, #3042 (harness bugs), #3043 (render recipe on the loop page).

## Plain summary

The roadmap sync writes `docs/roadmap/issues.json`, but no item's scope covered that file, so the sync commit could not pass the scope gate. This PR adds the file to the loop umbrella's scope; the sync commit follows as a separate PR.

References #2904
