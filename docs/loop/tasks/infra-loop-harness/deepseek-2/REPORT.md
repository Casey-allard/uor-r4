# REPORT: infra-loop-pr11-progress-window, deepseek-2 (loop PR 11 repair: answer the CHANGES review)

Lab: deepseek. Order: [ORDER.md](ORDER.md), committed as `7eff28935` before any repair.
Base: `a8bec0a` (the reviewed head). References #2904. Closes #2916.

## What did not work, first

- **HIGH 1a, `loop arrive` failed for every lab every session.** The raised `hooks` warning told every lab to install the shared hook, which the harness README says not to do until it is sanctioned. It is **WARN again**, with the opt-in reason in the line; loop PR 12 is what sanctions installing it.
- **HIGH 1b, `loop arrive` failed on every task branch.** `Staleness::Unknown` covers both "HEAD is not in origin/main" - every task branch and repaired PR worktree - and "cannot read HEAD or origin/main". Only the second FAILs now; a branch ahead of main PASSes. A test pins it.
- **HIGH 2, the planted fault was not pinned.** Deleting the `else if changed_at.elapsed() >= give_up` branch left all 12 window tests green. The window loop is now `watch_loop(read, out, last, give_up, wait)`: the file reader, the writer, the poll and the give-up duration are injected. I removed the give-up branch by hand and the bounded normal test failed with "the window never gave up; its give-up branch is missing"; the branch restored byte-identical.
- **The appended break could not plant this fault.** The first re-aimed break still went **DEFECT**, because an appended test can only add a passing case. `docs/harness/gates.json` breaks now may declare one literal `replace` (`from` must occur exactly once) instead of appended text, so `loop-gates-watch` deletes the give-up branch in the scratch worktree: `--break` now reports **CAUGHT** with "the window never gave up". A replacement whose `from` is absent is REFUSED, and both cases have tests.
- **MEDIUM: UNKNOWN ledger rows zeroed the estimate.** Rows written for gates skipped after a red carry `seconds: 0.0`; `last_seconds` now skips them, and a test writes a GREEN row then a newer UNKNOWN row and asserts the real timing survives.
- **MEDIUM: the README contradicted arrive.** The "Before you install the hook" section now says `loop arrive` reports a missing or different hook as WARN until loop PR 12 sanctions installing it.
- **LOW: the claim record was in the diff.** `docs/harness/claims.json` is restored to `origin/main` and no longer appears in this PR.
- **LOW: the test fixture stripped two `GIT_*` variables.** It now strips every one.
- **LOW: the five-minute close on a long gate is now named** in the harness README's window paragraph.
- **Still expected, and not a defect of this repair:** with a repair ORDER present, `loop pr-check --local` reports "the PR adds 2 ORDERs" for `order`, `scope`, `commit-order` and `report`; the rule has no repair-order case yet. `scope` already fails every PR until items carry `scope:` lines.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopgates.rs` | code | +120/−45 | `watch_loop` with injected reader, writer, give-up and wait; `last_seconds` skips UNKNOWN rows; a `replace` break kind |
| `crates/uor-r4-harness/src/looparrive.rs` | code | +25/−20 | `hooks` back to WARN with the opt-in reason; `checkout` FAILs only when behind or unreadable, PASSes ahead of main |
| `crates/uor-r4-harness/tests/loop_gates_watch.rs` | tests | +90/−10 | the giving-up, closed and red-as-it-lands tests; bounded waits; every `GIT_*` stripped |
| `crates/uor-r4-harness/tests/loop_arrive.rs` | tests | +30/−20 | the WARN hook cases and the branch-ahead PASS |
| `crates/uor-r4-harness/tests/loop_gates_run_rules.rs` | tests | +55 | a `replace` break is CAUGHT; an absent `from` is REFUSED |
| `docs/harness/gates.json` | data | +6/−2 | `loop-gates-watch`'s break is the `replace` that deletes the give-up branch |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +8/−3 | the break kinds, the long-gate close, the hook WARN rationale |
| `docs/harness/claims.json` | generated | restored | back to `origin/main` |

**Size:** this repair adds about 250 lines against the ORDER's 250-line estimate, about 1.0.

## Measurements

- **The negative gate:** with the give-up branch removed, `cargo test --test loop_gates_watch the_window_gives_up` failed in 0.26 s with "the window never gave up; its give-up branch is missing"; the file was then restored byte-identical.
- **`loop gates --full` at the head:** GREEN on all four gates (fmt-claim-wording 0.3 s, fmt-harness 0.7 s, claim-wording 2.0 s, loop-gates-watch 0.8 s).
- **`loop gates --break all` at the head:** CAUGHT on all four; `loop-gates-watch` red at exit 101 with "the window never gave up".
- **Tests at the head:** `loop_gates_watch` 15 passed, `loop_arrive` 18 passed, `loop_gates_run_rules` 31 passed, `loop_prcheck` 28 passed; the full suite is recorded below.
- **pr-check:** only the repair-order and scope failures above; `read-path`, `owner-paths` and `rule-sources` PASS.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 658 tests in the library target and every integration target passed, including `loop_gates_watch` 15, `loop_arrive` 18, `loop_gates_run_rules` 31 and `loop_prcheck` 28.
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on four gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on four gates.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: the expected repair-order and scope failures, named above.

## Skills used

- **gates**, read from its private local install: its progress-window design is the model for the injected-loop tests.

## Still open

- **`loop pr-check` has no repair-order case**, so a repaired PR shows "adds 2 ORDERs" for four rules until it is taught. That is loop PR 5/7c territory, not the watch's.
- **A single gate longer than five minutes closes the window**; the run continues and the result is unaffected. That is the specified behaviour.
- **No pty test**: whether the window opens in a real terminal is decided by the tested decision function, not an allocated terminal.
- **The `--break` schema now has two kinds**; a break that uses both refuses to load, and a `replace` whose text is absent is REFUSED.
- **A branch both ahead of and behind `main` still PASSes `checkout`** with no note of the gap: the check now keys on `version.rs`'s "not in origin/main" message to tell the two apart, and a `Staleness::Ahead` variant would be safer. That is version.rs work for a later PR.
- **The give-up line hardcodes "5 minutes"**, so an injected shorter duration (tests) prints the production wording. That is deliberate today; a parameterised message would need a test update.

## Plain summary

- **The window's fault is now really planted.** The progress window's give-up branch can be deleted from a throwaway worktree, and the gate suite turns red because the tests notice it is missing - not because a generic panic was appended.
- **A branch ahead of `origin/main` no longer fails arrival.** Arriving in your own task branch PASSes; only a checkout that cannot be read or is behind main fails.
- **The missing shared hook is a warning, not a failure**, until loop PR 12 sanctions installing it, so `loop arrive` stops being permanently red.
- **The estimate no longer collapses to zero** after a run stopped at a red gate.
- **The claim record and stray fixture details are out of the PR.**
