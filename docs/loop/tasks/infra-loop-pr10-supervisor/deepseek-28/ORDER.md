# ORDER: infra-loop-pr10-supervisor, deepseek-28 (repair: the real check-in note and the model pin)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `fb6df367b`. Item PR (partial repair; the item stays `ready`).

## Goal

Repair the two **unambiguous** findings from Claude's CHANGES on the withdrawn close-out (#2984): (1) the soft check-in says "real progress continues with a note on the issue", but the `notice` hook only prints; (2) `--model` is recorded free text and is never checked against the launch argv. The disputed clause ("the PM prompts any lab that stopped" — card on stop events vs only supervisor stops) is under a decision card on #2028 and is **not** resolved here.

## Cancel probe (run first)

Class **instrument** (supervisor repair; host only). At `fb6df367b`: the CLI `notice` closure is `println!` only (no `gh` call, no post test), and `--model` is not checked against the launch argv. Both reproduce; the ambiguous card clause is left to the decision card.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, `docs/harness/README.md` (shared), and this task folder.

## Done when

- A check-in note is posted for real: `note_body` is tagged, carries the session target and the no-claim line; `post_note_with` runs `gh issue comment <issue> --body-file`, keeps a copy under `local/loop/drive/<lab>/`, and a fake-`gh` test asserts the argv and the body file content.
- The CLI `notice` hook posts the note (target = the PR the last GO names, else #2028) in addition to printing, and a failed post is reported to stderr without killing the session.
- The model is a checked pin: `model_in_launch` and the CLI refuse a `--model` the launch argv does not carry, with tests both ways.
- `docs/harness/README.md` states both behaviours; `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head; the item stays `ready` (no status flip).

## Stop and ask if

- The card-clause reading is decided the other way (then a follow-up repair adds it and flips the item).
- A file outside the scope above would need to change.