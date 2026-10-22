# REPORT: infra-loop-pr12-git-hooks, deepseek-3 (loop PR 12: git hooks, pr-check before push, Lab and Order trailers)

Lab: deepseek. Order: folded into [deepseek-4/ORDER.md](../deepseek-5/ORDER.md) as history; the original ORDER.md was removed so the PR adds exactly one ORDER.
Base: `0e0ff6ce7` (fresh `origin/main`, with PR 11 on it). Closes #2917. References #2904.

## What did not work, first

- **The item scope still does not hold the files the done-when needs.** `kimi-4` (#2933) adds the two hook scripts and `tests/git_hooks*.rs`, and the owner's shared-nine decision exempts `main.rs`, `doccheck.rs`, `docs/harness/README.md` and `docs/harness/gates.json`; until both land on `main`, `loop pr-check` reports `scope` FAIL for `githelp.rs` (which IS in this item's scope) only because the base scope is stale. This REPORT does not touch the scope; it names the gap for the PM.
- **The first ORDER commit carried the claim line**, so `commit-order` failed: the rules require the ORDER commit to change nothing outside its own folder. The branch is unpushed and was rebuilt before this REPORT so the first commit adds only `deepseek-3/ORDER.md`; `docs/harness/claims.json` moved to the code commit.
- **Two full-suite runs showed phantom failures** when a second cargo test job shared `CARGO_TARGET_DIR` in another worktree: the test binary is one path under the shared target dir, so two worktrees can run each other's build. Every suite in this REPORT ran alone; the phantom failures are recorded here as instrument, not mechanism.
- **`check-message`'s binary-current check is weaker than it reads.** It uses `version::placement_line`, which compares the running copy's age with the newest build in THIS checkout; a lab with an external `CARGO_TARGET_DIR` (this one) has no build in the checkout, so a stale PATH copy is not seen. The hook still refuses a binary the harness can prove older; the gap is named in Still open.
- **The pre-push hook will currently block lab pushes** whenever `loop pr-check --local` fails, which it does on any repaired branch (the repair-order tool gap) and on the stale scope above. The hook is correct; installing it is opt-in, and no lab may use `--no-verify`, so it stays opt-in until the tooling settles.
- **Merge messages are exempt from the trailer check**, because git generates them; every other commit must name its lab and order. The exemption is deliberate and tested.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/githelp.rs` | code | +307/−80 | three managed hooks (pre-commit, commit-msg, pre-push); `install_one_hook` keeps the foreign/symlink/unreadable refusals for each; `git check-message` (trailers plus binary currency) and `git push-check` (`pr-check --local` per pushed lab branch) |
| `crates/uor-r4-harness/rules/commit-msg-hook.sh`, `rules/prepush-hook.sh` (new) | data | +44 | thin hook scripts that call the two new actions and let the commit or push through without the harness |
| `crates/uor-r4-harness/src/main.rs` | code | +66/−2 | `git check-message` and `git push-check` dispatch and the command table |
| `crates/uor-r4-harness/src/doccheck.rs` | code | +2 | the two new documented actions |
| `crates/uor-r4-harness/src/looparrive.rs` | code | +50/−35 | the `hooks` check now verifies all three managed hooks |
| `crates/uor-r4-harness/tests/git_hooks.rs` (new) | tests | +335 | install and refresh, foreign refusal, both new actions, and both hook scripts end to end |
| `crates/uor-r4-harness/tests/loop_arrive.rs` | tests | +8 | the fixture installs the three hooks |
| `docs/harness/README.md`, `docs/harness/gates.json` | docs/data | +33/−15 | the three hooks, what each blocks, the `git-hooks` gate and its planted break |
| the item's `status:` line, `docs/roadmap/README.md`, `docs/harness/claims.json` | generated/data | small | the item is done; the roadmap re-rendered; the claim line |
| `docs/loop/tasks/infra-loop-harness/deepseek-3/**` | docs | +120 | this ORDER and REPORT |

**Size:** about 800 insertions against the ORDER's 900-line estimate, about 0.9.

## Measurements

- **`loop gates --full` at the head:** GREEN on all five gates - fmt-claim-wording 0.6 s, fmt-harness 1.9 s, claim-wording 35.6 s, loop-gates-watch 3.5 s, git-hooks 8.4 s.
- **`loop gates --break all` at the head:** CAUGHT on all five; `git-hooks` red at exit 101 with `planted_break_hooks`, `loop-gates-watch` red with `the window never gave up`.
- **Tests at the head:** the full `cargo test -p uor-r4-harness --no-fail-fast` suite is green; the loop targets ran 15 (watch), 18 (arrive), 30+ (run rules), 28 (pr-check), and the new `git_hooks` target ran 20 tests.
- **`loop arrive` on this branch:** PASS checkout (ahead of origin/main), PASS binary, WARN hooks with the opt-in reason, arrive: OK - the three-hook check runs and does not fail a session.
- **The hooks end to end** (`tests/git_hooks.rs`): a message with no trailers is refused by the installed commit-msg hook and a message with them is allowed; a merge message is allowed; a pushed `deepseek/topic` ref makes push-check run `loop pr-check --local` and block, while a `main` ref is skipped; a foreign commit-msg hook is refused and left byte-identical.
- No research compute, no GPU, no pod and no store; the only network was `git fetch`.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: every target green (run alone).
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on five gates.
- `UOR_LAB=deepseek uor-harness loop arrive`: OK (checkout PASS, hooks WARN).
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: `order`, `report`, `owner-paths`, `rule-sources` and `read-path` PASS; `scope` and `commit-order` are the two named structural gaps above.

## Skills used

- **gates**, read from its private local install: the gate and break pattern covers the new `git-hooks` gate.

## Still open

- **`pr-check` scope** on this PR: `kimi-4` (#2933, merged as `b40b82e0d`) adds the hook scripts and `tests/git_hooks*.rs`, and the shared-nine exemption covers `main.rs`, `doccheck.rs`, `docs/harness/README.md` and `docs/harness/gates.json` once claude-22 lands its tool side. **`looparrive.rs` is the remaining gap**: PR 11's scope owns it and it is not shared, but this PR changes one function there so arrive checks all three hooks. A scope delta for pr12 (or the shared list) is needed; the alternative is a wrong arrive. Raised on #2028. commit-order is fixed in this branch by the rebuilt single-folder ORDER commit.
- **The binary-current check cannot see a stale PATH copy when the checkout has no build of its own** (external `CARGO_TARGET_DIR`); a content or commit identity would close it, which is version.rs work.
- **The pre-push hook blocks repaired branches** until the repair-order tool gap is fixed (claude-22); it is opt-in, and no lab may use `--no-verify`.
- **`loop arrive` reports a missing hook as WARN, not FAIL**: the shared hook stays opt-in until the hook tooling is settled, and a permanently red check teaches labs to ignore FAIL.

## Plain summary

- **Three git hooks now guard a commit and a push.** `pre-commit` still runs the staged checks. `commit-msg` refuses a message that does not name its lab (`Lab:`) and its ORDER (`Order:`) and refuses a binary the harness can prove is older than the checkout's build. `pre-push` runs the loop's own policy check on the pushed head for a `<lab>/<topic>` branch, and skips other refs.
- **The decisions are Rust, not shell.** The hook scripts only find the repository, call `uor-harness git check-message` or `git push-check`, and relay their exit code; all the parsing, the trailer rule, the lab-branch rule and the policy call are in the tested binary.
- **The installer manages all three hooks the same way**: it refuses to overwrite a hook it did not write, refuses a symlink, and a re-run says `unchanged`.
- **`loop arrive` now checks all three and still only warns**, because installing the shared hook is opt-in during setup.
- **The new `git-hooks` gate plants a real fault**: it appends a failing test, and `--break all` catches it, so a change that breaks the hooks - or the gate itself - turns the suite red.