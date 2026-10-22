# ORDER: infra-loop-pr12-git-hooks, deepseek-6 (loop PR 12, round-3 repair: restore-on-failure, commentChar=auto, and the missing tests)

Lab: deepseek. Item: [infra-loop-pr12-git-hooks](../../../../roadmap/items/infra-loop-pr12-git-hooks.md) (#2917). Setup row: 12. Base: `3994731aa`.

## Done in deepseek-3, deepseek-5 and deepseek-6 (history)

- deepseek-3 ORDER `b37bde663`: three managed hooks, `check-message`, `push-check`, the `git-hooks` gate.
- Round-1 CHANGES at `77fd684db`; deepseek-4 ORDER `74c7b345c`: branch-gated trailers, LFS chain, owner-only installer, all-or-none, real planted break.
- Round-2 CHANGES at `d65aba83a`; deepseek-5 ORDER added alone at `6e5c5ca37`: stock LFS pre-push replaced after backup, comment/scissors stripping, exact generated forms, Lab match, restore-safe staging; `pr-check --local` then passed whole.
- Round-3 CHANGES at `ec3cf5805`: restore-on-failure deletes a hook that was already current; `core.commentChar=auto` is read as the letter `a`; and the restore, scissors and commentChar tests are missing.

## Goal

Answer the round-3 CHANGES record at `ec3cf5805`:
1. **Do not stage or rename a hook that is already current.** On a failure, restore from the backup when one exists and remove a hook only when it did not exist before; remove every temporary file.
2. **Handle `core.commentChar=auto`**: read the character from the comment block git inserted (its "Please enter the commit message" or scissors line), never from the literal word `auto`.
3. **Add the tests the record names**: custom `core.commentChar`, `auto`, the scissors block, and the restore helper keeping/removing/restoring the right hooks.
4. **Restore the `gates.json` layout** so its diff is only the new gate, and correct the REPORT line that still said `commit-order` fails.

## Cancel probe (run first)

The round-3 CHANGES record is counted at `ec3cf5805`, so the repair is needed. Reproduced: the restore loop takes the no-backup branch for a hook that was already current, and `comment_char` with `auto` returns `a`. Class: instrument, fixed here.

## Files in scope

- `crates/uor-r4-harness/src/githelp.rs` (and `version.rs` readers only).
- `rules/*-hook.sh`.
- `crates/uor-r4-harness/tests/git_hooks*.rs`.
- `docs/harness/README.md`, `docs/harness/gates.json`.
- The item's `status:` line, `docs/roadmap/README.md`, and `docs/loop/tasks/infra-loop-harness/deepseek-6/**`.

## Done when

- An install where every hook is already current writes no temporary file, makes no backup, and says `unchanged`.
- A failure during the renames restores a replaced hook from its backup, keeps a hook that was already current, removes only a freshly installed one, and leaves no temporary file.
- `core.commentChar` is honoured for a custom character, for `auto` (read from git's inserted block) and with a scissors block; a message with trailers above the scissors passes and fake trailers below it do not count.
- `gates.json` has no reformatting churn and its `git-hooks` break still turns the gate red with the trailer test's own text.
- `loop gates --full` GREEN on five, `--break all` CAUGHT on five, the full suite and clippy green, `loop pr-check --local` PASS.

## Stop and ask if

- A fix would change a hold, a cap or an owner path; a file outside scope must change; a new dependency is needed.

Size estimate: about 180 lines.
