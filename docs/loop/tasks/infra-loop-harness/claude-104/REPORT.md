# REPORT: infra-loop-harness, claude-104

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2819.

## What did not work, first

- **`loop claim` refused this unit.** Codex's idle draft #3169 holds an order on `infra-loop-harness`, so this ORDER is written by hand under the run file's umbrella exception. #3169's files are `PLAN.md`, `docs/integration/DECISIONS.md` and three files in its own task folder, and none of them overlaps this unit.
- **No result and no compute.** This unit only lets claude-103 (#3203) start building. The schedule, the sheet and the GPU arms are all still ahead.

## What changed

`docs/roadmap/items/carrier-credit-hard-selection.md`:
- **`scope:`** gains `crates/uor-r4-discrete/**`, as 19 items on `main` already do. The hard-selection schedule has to live in that crate: `Device::apply` is `pub(crate)`, and a move commits only through `PreparedDriver::transition`.
- **`notebook:`** is bound to `docs/research/notebook/m1-discrete-attention.md`, which already exists.
- **The done-when line is replaced** by the exact line in claude-103's approved [decision card](../../carrier-credit-hard-selection/claude-103/DECISION.md), with one note giving the reason and a link to the card.
  - The archived sheet cannot be replayed: its objective is a log-mean-exp over softplus of a log mass (§3.2), its modules are compiled out, and saved437 is only in cold storage.
  - The new line keeps the step, every registered arm and the recorded result. It adds CUDA integer-only selection, model-blind controls, a fresh FIT and fresh rows.

`docs/roadmap/README.md` is re-rendered, and `roadmap render` touched no other file. This item's own edits change no README line. The whole README diff is a missing render left from #3192: `m1-a8-full-dose-rerun` is already `status: done` on `main`, so the counts go ready 47 → 46 and done 39 → 40, and its row moves.

The decision was approved after a fresh-context decision review: CHANGES in rounds 1–3, then APPROVE in round 4. It is logged on #2819 as an owner log (comment `6395405669`).

## Measurements

None. This is a records unit, so there is no run, no GPU and no model.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 92 items OK.
- `cargo run -p xtask -- roadmap render`: wrote only `docs/roadmap/README.md`.
- `cargo run -p xtask -- roadmap render --check`: `docs/roadmap/README.md` is current.
- `loop report --check`: PASS.
- `loop pr-check --local`: PASS. The review-record and required-checks rules report N/A in a local run.

## Skills used

- scope-guard: the scope gains one crate glob, used the same way by 19 other items, and nothing else.

## Still open

- **claude-103 (#3203)** builds the schedule, the fresh binding and the sheet, then runs the eight arms.
- **#3169's idle hold on `infra-loop-harness`** keeps forcing hand-written umbrella orders. It is Codex's draft, so it is not mine to close.

## Plain summary

This lets the carrier-credit work edit the one crate where the live learner lives, binds its notebook, and rewrites its goal into the version that can lawfully be run: a new, reviewed test on today's integer learner, not a replay of a banned older one.
