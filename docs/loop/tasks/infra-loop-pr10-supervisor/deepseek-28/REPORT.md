# REPORT: infra-loop-pr10-supervisor, deepseek-28 (repair: the real check-in note and the model pin)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `fb6df367b`. Item PR; the item stays `ready`.

## What did not work, first

- **The close-out I withdrew flipped `done` over two undelivered clauses, and Claude was right.** This repair delivers only the unambiguous half: a real progress note and a checked model pin. The other clause ("the PM prompts any lab that stopped") is a done-when/bar reading with a conflict against the planted-fault sentence, so it is on a decision card (#2028, 17:28Z) and is **not** resolved here.
- **The previous `notice` hook was a `println!`.** It now posts through `gh issue comment` (the PR the last GO names, else the tracker #2028), keeps a copy under `local/loop/drive/<lab>/`, and reports a failed post to stderr without killing the session. The pipeline is tested with a fake `gh` script that records its argv and the body file — no network.
- **The model was free text.** `--model` must now appear in the launch argv or the drive refuses with a usage error; the recorded model is therefore the launched one. The remaining limit is honest and documented: the pin itself still comes from the caller's launch command, because the owner-only registry does not store model ids yet.
- **The item is not flipped.** It stays `ready` until the card clause is decided and built; this PR is explicitly partial.
- **No owner path, no GPU, no spend.** Host tests only.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | +~110 | `note_body`, `post_note_with`, `post_note`, `model_in_launch`; CLI `last_target` + posting notice; the model-pin refusal |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | +~70 | the note body, the fake-`gh` post pipeline (argv + body + kept file), the model-pin both ways |
| `docs/harness/README.md` | docs | +5 | the note post and the checked model pin |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-28/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 190 changed lines (ORDER estimate about 180, 1.05).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 21 passed (18 before + 3 new).
- **Fake-`gh` pipeline:** the recorded argv contains `issue comment 1234 --body-file`; the recorded body contains the lab tag, `session target: #77` and the no-claim line; the note file is kept under `local/loop/drive/deepseek/`.
- **Model pin:** `deepseek-flash` in a launch argv passes; `claude-haiku` and the empty string are refused.
- **Whole harness suite:** run before the review request.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: nine GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: nine CAUGHT.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the ambiguous clause is left to its decision card instead of being guessed; the delivered half has a real post-pipeline test, not a mock of the decision.
- **scope-guard**: only the supervisor source, its tests, the shared usage doc and the task folder.

## Still open

- **The card clause** (supervisor stops only vs every self-exit) waits on the #2028 decision card; then a small follow-up repair adds it and flips the item to `done`.
- **The model pin source** is still the launch command; storing model ids in the owner-only registry would need an owner change.

## Plain summary

- **A progress note is now a real GitHub comment.** The check-in posts it on the PR the decision names (or #2028), keeps a local copy, and never claims a result; the fake-`gh` test proves the post pipeline.
- **The model is checked, not just recorded.** A `--model` missing from the launch argv is refused, so the recorded pin matches the launched model.
- **The item stays `ready` on purpose.** The remaining clause is a done-when reading under a decision card; flipping now would repeat the mistake Claude caught.