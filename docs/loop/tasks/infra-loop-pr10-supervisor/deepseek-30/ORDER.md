# ORDER: infra-loop-pr10-supervisor, deepseek-30 (repair A: PM-prompt cards on supervisor stops, and the close-out)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `f3622f66a`. Item PR; other-vendor exact-head review at the head.

## Goal

Build the last undelivered done-when clause under the PM ruling: "the PM prompts any lab that stopped" means the **supervisor** stopping — card on `STOP`, reaching an explicit multi-session `--max-sessions` limit, and an error exit — while a clean session exit that the supervisor restarts stays uncarded (the pinned progressing-client test keeps zero cards). Then flip the item to `done`.

PM ruling: Kimi on #2028 at **2026-10-22T18:19:09Z** (comment 6348017248), option **A**; Claude's reading A at 19:08Z; Casey asked in chat (no contrary answer).

## Cancel probe (run first)

Class **instrument** (host only). At `f3622f66a`: `run_sessions` returns cleanly on `Next::Stop` without calling the `card` hook; reaching `--max-sessions` completes the loop with no card; the CLI's `Err` branch only prints. The note post and the model pin are already delivered by #2987 on `main`. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive*.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, `docs/harness/README.md` (shared), `docs/roadmap/items/infra-loop-pr10-supervisor.md` (`status:` line only), `docs/roadmap/README.md` (rendered), and this task folder.

## Done when

- `Next::Stop` posts one PM card (tagged reason), the CLI's error exit posts a best-effort failure card, and reaching an explicit multi-session `--max-sessions` limit posts one card; the default single-session unit and a clean session exit post none, so `the_fake_clock_continues_a_committing_client_with_a_note` still asserts zero cards.
- New tests pin the `STOP` card and the multi-session-limit card; `loop_drive` passes; the docs state the behaviour.
- The item's done-when is delivered end to end (note post + model pin from #2987, cards from this PR), so its `status:` flips to `done` and the roadmap render is current.
- `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head.

## Stop and ask if

- The PM reverses the A ruling (then the planted-fault sentence/test change with B).