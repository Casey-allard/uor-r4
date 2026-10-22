# ORDER: infra-loop-pr12-git-hooks, deepseek-5 (loop PR 12, round-2 repair: the installer cannot refuse LFS's own hook, and an editor message loses its trailers)

Lab: deepseek. Item: [infra-loop-pr12-git-hooks](../../../../roadmap/items/infra-loop-pr12-git-hooks.md) (#2917). Setup row: 12. Base: `3994731aa`.

## Done in deepseek-3 and deepseek-4 (history)

- deepseek-3 ORDER `b37bde663`: three managed hooks, `check-message`, `push-check`, the `git-hooks` gate; code `a5f541b66`; REPORT, item, roadmap and receipts.
- Round-1 CHANGES at `77fd684db`; deepseek-4 ORDER `74c7b345c`: branch-gated trailers, LFS chain, owner-only dry-run installer, all-or-none with backups, push-check column fix, a real planted break; repairs `df46c365a`.
- Round-2 CHANGES at `d65aba83a`: the LFS stock hook is refused as foreign, and an editor message with correct trailers is refused because git's comment block is last. claude-22 (`3994731aa`) is merged and the chain is `deepseek-3` → `deepseek-5`.

## Goal

Answer the round-2 CHANGES record at `d65aba83a`:
1. **Recognise Git LFS's stock `pre-push` as replaceable**, back it up, and install the chained hook; add a test that starts from the real stock body.
2. **Strip git's comment block and scissors before reading the final paragraph**, honouring `core.commentChar`, so a commit typed in an editor with correct trailers passes; test editor messages both ways.
3. **Keep all-or-none while writing**: stage temporary files and rename them, restoring replaced hooks on error; create the hooks directory first.
4. **Fix the README's arrive sentence** (arrive checks only `pre-commit` until `looparrive.rs` is in pr12's scope).
5. **Close the LOWs**: `Lab:` must equal `UOR_LAB`; only git's generated forms skip; back up under a pid-suffixed directory; note that lab-session detection is `UOR_LAB` only; the pre-commit hook's bypass line is out of this PR's scope and stays for a later change.

## Cancel probe (run first)

The round-2 CHANGES record is counted at `d65aba83a` (the last counted record), so the repair is needed. Both HIGHs reproduce on this branch: `~/uor-r4/.git/hooks/pre-push` is LFS's stock body and `git install-hooks --install` exits 2 on it; a valid-trailer message followed by git's `#` block is refused by `check-message`. Class: instrument, fixed here.

## Files in scope

- `crates/uor-r4-harness/src/githelp.rs` (and `version.rs` readers only).
- `rules/*-hook.sh` (the two new hook scripts at the scoped root).
- `crates/uor-r4-harness/tests/git_hooks*.rs`.
- `docs/harness/README.md`, `docs/harness/gates.json`.
- The item's `status:` line, `docs/roadmap/README.md`, and `docs/loop/tasks/infra-loop-harness/deepseek-5/**`.

## Done when

- A stock LFS `pre-push` is replaced and backed up by `install-hooks --install`, and the installed hook chains LFS first on the same stdin; a missing `git-lfs` exits 2.
- An editor-typed message with correct trailers passes and one without trailers is refused.
- `Lab:` must equal `UOR_LAB`; a hand-written `Merge ...` subject is not exempt; only git's own generated subjects are.
- Writing is all-or-none: temporary files are staged and renamed, and a rename failure restores the replaced hooks; the hooks directory is created first.
- The README says arrive checks only `pre-commit`, names the backups under `local/loop/install-backups/<seconds>-<pid>/`, the LFS stock replacement, the comment stripping and the `UOR_LAB`-only detection.
- `loop pr-check --local` passes everything except the structural `commit-order` of a repaired branch; gates --full is GREEN on five and --break all CAUGHT on five.

## Stop and ask if

- A fix would touch a hold, a cap or an owner path; a file outside scope must change; a new dependency is needed.

Size estimate: about 300 lines.
