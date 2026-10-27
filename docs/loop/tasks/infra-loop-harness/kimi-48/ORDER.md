# ORDER: infra-loop-harness, kimi-48

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base after merge-forward: `12931837a` (now `1b8eb131a`). Setup row: 8 (notes/roadmap migration and render refresh).

## Goal

Repair the fresh-main verification of #3314 after #3313 merged first. Rebase the generated views onto the new merge base, preserve the newer infra title/scope, and prove the resulting patch on fresh main.

## Cancel probe (run first)

`loop close --verify --pr 3314` reports MERGED but FAIL content: the merged patch differs from reviewed head `0587bace`. The delta is #3313's new infra work and the generated infra title, not a lost Kimi result. `classification: instrument`. **Not cancelled: merge forward and regenerate at the new base; do not revert #3313.**

## Files in scope

- `STATE.md`, `HANDOFF.md`, `AGENTS.md` (rendered lesson block only)
- `docs/harness/LESSONS.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/infra-loop-harness/kimi-48/**`

## Done when

- origin/main is merged into `kimi/render`; all views are rendered at the new merge base, with no #3313 reversion.
- `loop render --check`, `roadmap check`, `roadmap render --check`, full gates and local policy pass.
- REPORT, fresh exact-head review, queue and successful `loop close --verify` on fresh main are complete.

## Owner approval

At `2026-10-26T06:34:45Z`, Casey directed: "Then run the loop as PART 3 of the run file says" and "After it merges, run `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap sync`, a dry run first, then weekly." This repair performs that loop/PM render duty and regenerates the AGENTS.md lesson block without hand-editing its rules.

## Stop and decide if

- The repair changes an item, bar, threshold, result, #3313 behavior or issue state rather than the generated base and delivery proof.
- A conflict cannot be resolved without discarding another lab's work.
