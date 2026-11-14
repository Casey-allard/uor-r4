# ORDER: infra-loop-harness, codex-3

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row: 2.
Parent: [codex-2](../codex-2/ORDER.md). Reviewed head: `74deb0a6b`.

## Owner approval

The owner approved the native train/serve pivot and the control-plane update:

> "yes please update your corvuswing loopkit and harness too so that you can continue down the correct path"

Recorded at `2026-11-14T02:57:18Z`. This repair records the same approval for the existing
owner-path changes in `CHARTER.md`, `AGENTS.md`, `docs/rules/changes.md`,
`docs/integration/DECISIONS.md`, and `docs/harness/owner-paths.json`.

## Why this repair exists

The fresh review found that the new native-state item's scope used a glob covering the
harness's shared registration files, which makes `loop claim` refuse the item. It also found
live rule summaries that still describe D37's float-master/export-gap route as current, and
an active PICKUP that predates the current main and CUDA PR.

## Goal

Make the next native-state item claimable, align the remaining live rule summaries with D85,
and refresh the tracked handoff. Do not add another harness rule or change the canonical
assertion.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/codex-2/PICKUP.md`
- `docs/loop/tasks/infra-loop-harness/codex-3/**`
- `docs/roadmap/items/infra-loop-harness.md`
- `docs/roadmap/items/native-space-state-contract.md`
- `docs/rules/README.md`
- `docs/rules/decision-map.md`
- `docs/geometry/README.md`
- `docs/integration/project-track.md`
- `crates/uor-r4-harness/src/harnessrules.rs`
- `.agents/skills/uor-research-architect/SKILL.md`
- `docs/harness/skills/uor-research-architect/SKILL.md`

## Size

About 40 to 120 changed lines.

## Done when

1. The native-state item's scope contains no glob covering a `SHARED_FILES` entry and can be
   claimed after it lands on `main`.
2. The live rules, geometry guide, project tracker and decision map describe D85 as the
   default and the historical float/export route as retained evidence.
3. The tracked PICKUP names the current main, PR head and CUDA branch/PR state.
4. Formatting, link checks, the focused native-space test, the full harness package and
   `loop report --check` pass.

## Stop

- Do not change the canonical assertion, a shared harness file list or a rule.
- Do not add a new check.
- Do not rewrite historical task records or measured evidence.
