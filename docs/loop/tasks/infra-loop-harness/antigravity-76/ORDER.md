# ORDER: infra-loop-harness, antigravity-76 (unhold r-secant-surrogate-descent after #2823 and add float-train crate scopes)

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-101 and claude-104). Base: `77c241c709814200f5e4541605d37b57c3a0af66`.

## Goal

Following the pattern of `claude-101` (`#3200`) and `claude-104` (`#3206`), perform the D68 item-file maintenance needed now that `float-training-module` (`#2823`, `crates/uor-r4-float-train`) is `status: done` (`d9dfbdb40`) and `d11-exporter-float-trunk` (`#2820`, `export_float_trunk_uorlut01`) has landed (`77c241c70`):

1. **Unhold `r-secant-surrogate-descent` (`#2886`) from `status: held` to `status: ready`:** In `antigravity-61` (`#3202`, `#2886` comment `6395161953`), `#2886` was placed in `status: held` solely until `float-training-module` (`#2823`) landed, with the explicit dispatch gate: *"As soon as `#2823` lands on `main`, this sheet runs on a leased `uor-pod` RTX 5090 under the lab's own D67 pre-run review."* Now that `#2823` is `status: done` on `origin/main` and all `crates/uor-r4-discrete/**` ready items are locked by `PR #3203` (`claude-103`), unholding `#2886` lets the loop take `#2886` (which unblocks `#2824`, `#2890`, and `#2891`).
2. **Add `crates/uor-r4-float-train/**`, mechanism docs, and `notebook:` bindings to the five D37 offline-float items (`#2886`, `#2824`, `#2890`, `#2891`, `#2822`):** At `77c241c70`, those five items list only their `docs/harness/prep/*`, `docs/labs/*/**`, and `docs/loop/tasks/*/**` globs (`0` crate paths), so `loop guard` and `pr-check` would refuse any runner or test added in `crates/uor-r4-float-train/**`.
3. **Re-render `docs/roadmap/README.md`** via `~/.cargo/bin/cargo run --offline -p xtask -- roadmap render` and verify `roadmap check`.

## Cancel probe (run first)

Class: `instrument` (records/umbrella check at `77c241c709814200f5e4541605d37b57c3a0af66`):
1. `uor-harness loop claim infra-loop-harness --lab-topic float-items-unhold-scope` refused solely because `PR #3169` (`codex-1`, idle draft since `2026-10-25T01:07:47Z`) holds the `infra-loop-harness` umbrella. Verified via `gh pr view 3169 --json files,isDraft,updatedAt` that `#3169` touches only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/{ORDER,PLAN,REPORT}.md` (`0` overlapping files). Claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella exception.
2. Verified `float-training-module` (`#2823`) is `status: done` on `origin/main` (`77c241c709814200f5e4541605d37b57c3a0af66`), while `r-secant-surrogate-descent` (`#2886`) is still `status: held` and all five D37 float items (`#2886`, `#2824`, `#2890`, `#2891`, `#2822`) lack `crates/uor-r4-float-train/**` in `scope:`.

## Files in scope

- `docs/roadmap/items/r-secant-surrogate-descent.md`
- `docs/roadmap/items/hamilton-gate-offline-float-sheet.md`
- `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md`
- `docs/roadmap/items/spiralcore-fbs-offline-float-sheet.md`
- `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md`
- `docs/roadmap/README.md` (shared registration file)

## Done when

- Fresh-context D68 decision review approves unholding `#2886` (`status: held` -> `status: ready`) and adding `crates/uor-r4-float-train/**`, mechanism docs, and `notebook:` bindings to `#2886`, `#2824`, `#2890`, `#2891`, and `#2822`, and is logged (`[antigravity] owner log:`) on `#2886`;
- `~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass (`References #2904`, `References #2886`).

## Stop and ask if

- Any file outside the declared scope is needed, or any gate in `uor-harness loop gates --full` fails.
