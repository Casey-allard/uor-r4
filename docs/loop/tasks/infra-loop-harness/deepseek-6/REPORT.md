# REPORT: infra-loop-pr12-git-hooks, deepseek-6 (loop PR 12, round-3 repair)

Lab: deepseek. Order: [ORDER.md](ORDER.md); deepseek-3 and deepseek-5 are history inside it.
Base: `3994731aa`. Closes #2917. References #2904.

## What did not work, first

Round 3 of the CHANGES records. Round 2's two HIGHs were confirmed fixed on the built binary; this round answers the three items it left:

- **MEDIUM 1, restore-on-failure deleted a hook it should have kept.** A hook that was already current was staged and renamed like the others but had no backup, so if a later rename failed the restore loop removed it. Fixed: an `unchanged` hook is neither staged nor renamed; on a failure every hook that was renamed is restored from its backup, and a hook is removed only when it did not exist before the run; every remaining temporary file is removed. A test forces the failure path through the new `restore_replaced` helper (keep the current hook, restore a backup, remove a fresh one) and an install over three current hooks is tested to write no temporary file and make no backup.
- **MEDIUM 2, `core.commentChar=auto` was read as the letter `a`.** The value's first character was taken, so under `auto` the `#` block survived and lines starting with `a` were dropped. Fixed: on `auto`, the character is read from the comment block git inserted (the "Please enter the commit message" line or the scissors line, whichever appears); a configured single character still wins when it is not `auto`.
- **The record's missing tests exist now**: a custom `core.commentChar` (`;`), `auto` (a `#` block), the scissors block (trailers above it count, fake trailers below do not), and the restore helper's three cases. `git_hooks` is 24 tests and the `install_hooks` unit module is 9.
- **LOW, `gates.json` churn.** The file is back to `origin/main`'s layout and the diff against it is only the `git-hooks` gate (+24 lines); its planted break is still the trailer branch replaced in `githelp.rs`, and `--break all` is CAUGHT with `a message without trailers must be refused`.
- **LOW, the REPORT line** that still said `commit-order` fails is gone; this report records `pr-check --local` PASS.
- **Still deferred by the ORDER**: the `--no-verify` line in the pre-existing `rules/precommit-hook.sh` (outside this PR's scope) and `looparrive.rs`'s `install-hooks` advice (PR 11's scope). Both stay on the follow-up list.
- **The first full suite after the round-2 fixes had found two failures** (the unit tests read `out[0]`, now the backup line first; `committed_writes_refuse_the_shared_checkout` saw the owner guard where it requires R10's). The guard now sits behind the R10 refusal and only on a write, unchanged hooks are not backed up, and the tests find their action line anywhere in the plan.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/githelp.rs` | code | +80/−45 | an `unchanged` hook is not staged or renamed; `restore_replaced` keeps a current hook, restores a backup, removes a fresh one; `comment_char` reads `auto` from git's inserted block; temporary files are cleaned on failure |
| `crates/uor-r4-harness/tests/git_hooks.rs` | tests | +100 | custom commentChar, auto, scissors, and the restore-path behaviours |
| `crates/uor-r4-harness/src/githelp.rs` (unit tests) | tests | +85 | the restore helper's three cases and `comment_char`'s custom/auto/default forms, plus the no-temp/no-backup install |
| `docs/harness/gates.json` | data | +24 | only the `git-hooks` gate, in the base layout |
| `docs/harness/README.md` | docs | small | the comments and `auto` behaviour, the `restore` guarantee |
| `docs/loop/tasks/infra-loop-harness/deepseek-6/**` | docs | small | this ORDER and REPORT |

**Size:** about 210 changed lines against the ORDER's 180-line estimate, about 1.2.

**Casey's three decisions of 22 Oct 2026, recorded verbatim by Claude, and built in:**
- `Trailers: "Only lab branches (Recommended)"`
- `Pre-push: "Chain with LFS (Recommended)"`
- `Installer: "Only you (Recommended)"`

## Measurements

- **`loop gates --full`:** GREEN on five gates.
- **`loop gates --break all`:** CAUGHT on five; `git-hooks` red with `a message without trailers must be refused`.
- **Tests:** `cargo test -p uor-r4-harness --test git_hooks` 24 passed; the `githelp::install_hooks_tests` unit module 9 passed; the full suite green (run alone).
- **The hook behaviour end to end:** a real `git commit` without trailers is refused on a lab branch and allowed with them; the stock LFS `pre-push` is replaced after its backup; LFS runs first on the same stdin; no `git-lfs` exits 2; a new branch is checked and a delete skipped; an install over three current hooks says `unchanged` three times, with no temporary file and no backup.
- **`loop pr-check --local`:** PASS on every rule.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: every target green (run alone).
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on five gates.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.

## Skills used

- **gates**, read from its private local install: the planted-fault pattern.

## Still open

- **`looparrive.rs`'s hook check and its `install-hooks` advice** are PR 11's scope; extending arrive to the three hooks needs a scope delta and is a follow-up.
- **`rules/precommit-hook.sh`'s `--no-verify` line** is pre-existing and outside this PR's scope; the two new hooks never suggest a bypass.
- **The binary warning path has no direct test** (it needs a build inside the checkout).
- **The LFS chain is tested with a fake `git-lfs`**, not a real transfer.

## Plain summary

- **A failed install cannot eat a working hook.** Hooks that are already current are left completely alone, a failure restores every hook that was replaced, and only a freshly installed hook is removed again; no temporary file is left behind.
- **`core.commentChar=auto` works.** The character is read from the comment block git itself inserted, so an editor message keeps its trailers and body lines are not silently dropped.
- **The tests the review asked for exist**: custom comment characters, auto, scissors, and the restore helper's three outcomes.
- **The gate diff is exactly the new gate**, and the planted fault still turns the gate red with the trailer test's own refusal text.