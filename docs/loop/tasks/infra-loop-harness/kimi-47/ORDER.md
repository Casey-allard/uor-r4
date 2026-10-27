# ORDER: infra-loop-harness, kimi-47

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `5b560b6fbb35f8df2210ce26ae202123ab99ec90`. Setup row: 8 (notes/roadmap migration and render refresh).

## Goal

Deliver the PM roadmap sync and all rendered views at the merge base, after #3306. Keep the existing generated status changes only; do not change an item, bar, result or issue state in this PR.

## Cancel probe (run first)

The initial sync dry-run/apply changed only `m1-fresh-panel-transfer` from held to ready in `docs/roadmap/README.md`, with ready 12→13 and held 12→11. That matches the item frontmatter and antigravity-109's decision; no experimental claim changes. The ORDER-less sentinel review found that roadmap README is outside its allowed views and that the other views were stamped at an older base. `classification: instrument`. **Not cancelled: use a normal ORDER/REPORT and render every view at the merge base.**

## Files in scope

- `STATE.md`, `HANDOFF.md`, `AGENTS.md` (rendered lesson block only)
- `docs/harness/LESSONS.md`
- `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/infra-loop-harness/kimi-47/**`

## Done when

- `uor-harness loop render --at 5b560b6fbb35f8df2210ce26ae202123ab99ec90` refreshes every view and `uor-harness loop render --check` passes at that stamp.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check` and `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render --check` pass.
- REPORT records the sync plan/apply, generated changes, checks and out-of-scope live-record mismatches. Full gates and local policy pass; a fresh exact-head review approves; queue and fresh-main verification complete.

## Owner approval

At `2026-10-26T06:34:45Z`, Casey directed: "Then run the loop as PART 3 of the run file says" and "After it merges, run `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap sync`, a dry run first, then weekly." This ORDER performs that loop/PM render duty and regenerates the AGENTS.md lesson block without hand-editing its rules; it changes no owner rule or constraint.

## Stop and decide if

- Any item metadata, bar, threshold, result or issue state would change. The stale #2841 prose and closed-#2820/ready-item mismatch need separate units, not an incidental render edit.
- The render produces non-generated code or documentation changes, or the review finds the sync output inconsistent with committed item metadata.
