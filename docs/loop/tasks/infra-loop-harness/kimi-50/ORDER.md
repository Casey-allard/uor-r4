# ORDER: infra-loop-harness, kimi-50

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `7746eec32a854394c2cd03c6edd4efa04f277c89`. Setup row: 8 (notes/roadmap migration and render refresh).

## Goal

Render the #3317 fresh-panel item prose update and the other shared views at that merge base. Do not change item metadata, results or issue state.

## Cancel probe (run first)

#3317 changed only three item prose lines and is verified on main. The generated roadmap and status views now need their normal PM refresh. `classification: instrument`. **Not cancelled: regenerate at `149e5f422`; do not edit item inputs again.**

## Files in scope

- `STATE.md`, `HANDOFF.md`, `AGENTS.md` (rendered lesson block only)
- `docs/harness/LESSONS.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/infra-loop-harness/kimi-50/**`

## Done when

- All views match their render at `7746eec32a854394c2cd03c6edd4efa04f277c89`; roadmap check and roadmap render --check pass.
- Full gates/local policy and REPORT pass; fresh exact-head review, queue and fresh-main verification complete.

## Owner approval

At `2026-10-26T06:34:45Z`, Casey directed: "Then run the loop as PART 3 of the run file says" and "After it merges, run `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap sync`, a dry run first, then weekly." This ORDER performs that loop/PM render duty and regenerates the AGENTS.md lesson block without hand-editing its rules.

## Stop and decide if

- Any item, bar, threshold, result or issue state would change.
- The render includes another lab's unmerged input or anything outside generated views and this task folder.
