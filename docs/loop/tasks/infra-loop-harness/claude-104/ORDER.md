# ORDER: infra-loop-harness, claude-104 (give carrier-credit-hard-selection the crate it must change, its notebook, and its lawful done-when)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-88, -90, -93, -96 and -101). Base: `dc5f409b0`.

## Goal

This unit makes one item-file fix, the scope fix that claude-103's approved D68 decision card calls for: [DECISION.md](../../carrier-credit-hard-selection/claude-103/DECISION.md). The card is logged as an owner log on #2819 (comment `6395405669`). It changes `carrier-credit-hard-selection` (#2819) in three ways.

1. **`scope:` gains `crates/uor-r4-discrete/**`.**
   - The hard-selection schedule has to live inside the discrete crate. `Device::apply` is `pub(crate)`, and a move commits only through `PreparedDriver::transition` with a controller admission.
   - `loop guard` reads scope from `origin/main`, so it would refuse every edit there.
   - Nineteen items on `origin/main` already carry this scope.
2. **`notebook: docs/research/notebook/m1-discrete-attention.md`** is bound. That page is the live learner's notebook, and it already exists.
3. **The done-when line is replaced** by the exact text the card approved. The archived sheet cannot be replayed lawfully: its objective is banned (§3.2), its modules are compiled out, and saved437 is only in cold storage. §4.5 lets archived float sheets "return only as new sheets with a fresh FIT". The new line still requires the hard-selection step, every registered arm and the recorded result. It adds CUDA integer-only selection, model-blind controls and fresh rows.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. `loop claim infra-loop-harness` refused: "item infra-loop-harness is already claimed by PR #3169 (docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md)". #3169 is Codex's draft, idle since 01:07Z. Its five files are `PLAN.md`, `docs/integration/DECISIONS.md` and three files in its own task folder, and none of them overlaps this unit's files. So this ORDER is written by hand, under the umbrella exception in the run file.

## Cancel probe (run first)

Class: records. Run at `dc5f409b0`; all must hold, or the unit is cancelled:
- `carrier-credit-hard-selection`'s `scope:` holds no `crates/` path. **Holds:** it lists only its prep, lab and task globs.
- No open PR touches `docs/roadmap/items/carrier-credit-hard-selection.md` or `docs/roadmap/README.md`. **Holds:** the open PRs are #3169 (`PLAN.md`, `docs/integration/DECISIONS.md`, its task folder) and #3203 (claude-103's task folder).
- The decision card is approved. **Holds:** round 4 APPROVE, owner log `6395405669`.

## Files in scope

- `docs/roadmap/items/carrier-credit-hard-selection.md` (the `notebook:` and `scope:` lines, the done-when line and one note)
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- The item edit is made. `roadmap check` passes, and `render --check` matches.
- `loop report --check` and `loop pr-check --local` pass, and the fresh review counts at the exact head.
- The REPORT says `References #2904` and `References #2819`.

## Stop and ask if

Stop and decide, never stop and wait.

- `roadmap render` touches `PLAN.md` or any file #3169 holds.
- The review finds that the new done-when is weaker than the old one.
