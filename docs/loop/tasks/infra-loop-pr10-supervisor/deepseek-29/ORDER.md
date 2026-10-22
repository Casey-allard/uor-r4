# ORDER: infra-loop-pr10-supervisor, deepseek-29 (repair: whole-argument model pin and the note lows)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `5864dc83b`. Item PR repair; the item stays `ready`.

## Goal

Repair Claude's CHANGES on #2987 at `5864dc83b`: the model pin is a substring match, so `--model deepseek` passes against `deepseek-flash` and `--model prompt` passes against `{prompt}`; the LOWs ask for a target-choice test, a note-file collision guard, and a cleaner fake-`gh` test.

## Cancel probe (run first)

Class **instrument** (supervisor repair; host only). At `5864dc83b`: `model_in_launch` is `launch.iter().any(|arg| arg.contains(model))`, so the prefix and placeholder cases pass; `post_note_with` names files by second; and the fake-`gh` test neither removes its temp root nor reads the kept copy. All reproduce. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, and the `deepseek-29` task folder.

## Done when

- `model_in_launch` matches whole arguments only: exact argument, `--model=<model>`, `-m=<model>`, or the value after `--model`/`-m`; prefix, placeholder and empty model all fail; `--model=x` and bare-value spellings pass.
- `note_issue` chooses the `#<n>` target number else #2028, and the CLI uses it (tested).
- Two notes in one second keep two files; the fake-`gh` test removes its temp root and asserts the kept copy's content.
- `loop_drive` tests, clippy `-D warnings` for the harness, `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head; the item stays `ready`.

## Stop and ask if

- The model pin needs the owner-only registry to store model ids (a separate owner change).