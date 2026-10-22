# REPORT: infra-loop-harness, claude-31 (no idle stops: WAIT, a polling `loop next --wait`, and a drive that re-polls)

Lab: claude. Base: `4bb1feede`, merged forward to `origin/main` (with #2969, #2971 and #2973). Order: [ORDER.md](ORDER.md), committed first (`76558b5fb`).

The cancel probe at `4bb1feede` found:
- `loop next` printed only `GO …` or `STOP …`;
- `loop drive` returned on any STOP (`loopdrive.rs:676-678`).

So the task stood. Class: object.

## What did not work, first

- **Labs parked with no way to wake.** Since the joins on 21 Oct, the longest silences on #2028 were Kimi 5.5 h, Claude 4.3 h, Antigravity 2.8 h and DeepSeek 2.3 h. Three paths caused it:
  - `drive` exited on any STOP; the page's "polls every 10 minutes" was never built.
  - A lab at `open_pr_limit` with all its PRs awaiting review got `STOP held work` or `STOP empty queue`, with the reason hidden in a detail line.
  - The docs said a pending owner question means wait, and only the PM resumed a stopped lab.
- **It took three builders.** Each stopped at its context mark after one item: WAIT (`0ebe3e718`), then `--wait` (`03e202c66`), then drive (`8bae4a49d`). I added the gate and the docs.
- **The drive builder's own negative gate failed only at compile time**, because the hook API changed. So I ran a behavioural one: replacing the WAIT arm's `(hooks.pause)(spec.poll)` with `return Ok(0)` makes `a_wait_pauses_by_the_poll_and_asks_again_then_stop_ends_cleanly` (`loop_drive.rs:748`) and `a_wait_then_go_starts_exactly_one_session` (`:807`) fail. Restored, all 18 pass.
- **Merging `main` brought DeepSeek's #2969** (lane and premise-probe sources). `gates.json` conflicted, because both PRs added a gate after `loop-drive`; both are kept. One #2969 test asserted the old `STOP empty queue` / exit 1, which this ORDER changes to WAIT / exit 3, so its two assertions now say so.
- **A wording slip from claude-30 is fixed here.** The loop page and `loopreview.rs:23` said `loop review` reads the D59 switch at the merge base. It reads `origin/main`; only `pr-check`'s review-record reads the merge base.

## What changed

| Commit | What |
|---|---|
| `76558b5fb` | This ORDER, alone, with the owner approval for `AGENTS.md`. |
| `0ebe3e718` | `loop next`: `Decision::Wait` and `RECHECK_SECS = 600`. `WAIT <reason> — check again in 600s` exits 3 (JSON: `"wait": true, "recheck_secs"`). An `open_pr_limit` refusal names the lab's own PRs. Only the kill switch prints STOP. New test `a_lab_at_its_open_pr_limit_waits_naming_its_prs_awaiting_review`; five existing STOP assertions became WAIT. |
| `03e202c66` | `loop next --wait [--poll SECS] [--max SECS]`: `wait_loop` with an injected sleeper and clock. It ends on GO (0) or STOP (1); `--max` gives 3; three read failures in a row give 2. Four unit tests, none sleeping. The flags are listed in `doccheck.rs`. |
| `8bae4a49d` | `loop drive`: `DriveHooks.decision` returns `Next::{Go, Wait, Stop}`, and `run_sessions` pauses `--poll` (default 600) on WAIT and asks again, without using a session slot. Two new tests. The module doc and the default prompt's last line are updated. |
| `12b3c59c1` | Gate `loop-next-wait`. Its planted fault turns the no-work WAIT back into STOP and is CAUGHT with "a lab at its open_pr_limit must WAIT, not STOP". |
| `32ed437dc` | Merged `origin/main`: #2969, #2971 and #2973. `gates.json` keeps both new gates. |
| `bf9453fe7` | Docs. Loop page step 1 and "Stops and waits"; `AGENTS.md`'s owner-question line (owner path, approved in the ORDER); the D59 switch wording; `loop_next_sources`'s empty-queue assertions. |
| this REPORT's commit | This REPORT. |
| `9d60929ff` | The ORDER trimmed to 3135 B. `pr-check`'s `read-path` rule caps an ORDER under 4096 B, and the first version was 5313 B. Same calls, done-when and owner approval. |

## Measurements

None. This is harness behaviour, with no model run.

## Checks run at the head

- `cargo test -p uor-r4-harness --no-fail-fast`: 32 test binaries, **1053 passed, 0 failed**. An earlier local run by another reviewer showed `git_hooks` failures; they do not reproduce here.
- Focused runs:
  - `--test loop_next`: 28 passed;
  - `--test loop_next_sources`: 3 passed;
  - `--test loop_drive`: 18 passed;
  - `--test loop_review`: 27 passed;
  - `--lib loopnext`: 6 passed.
- `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings` exits 0, and `cargo fmt --check` passes.
- `loop gates --full`: 8 GREEN (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `loop-next-sources`, `loop-next-wait`, `git-hooks`).
- `--break all`: 8 CAUGHT. `loop-next-wait` goes red with "a lab at its open_pr_limit must WAIT, not STOP".
- `loop report --check`, `loop pr-check --local`, claim-wording, links and the copyright check: see the PR body.

## Skills used

- **solution-integrity:** each fix is pinned by a test that fails without it. The negative gates were G1 for WAIT (it saw STOP, exit 1) and the behavioural drive gate above.
- **scope-guard:** `loopnext.rs`, `loopdrive.rs`, their tests, the shared registration files, `AGENTS.md` (owner-approved) and this task folder.

## Still open

- **Each client's own command limit.** A lab holds `loop next --wait --max <limit>`. The right `--max` per client (Claude Code's 10-minute foreground limit, the others' limits) belongs in each client's onboarding notes. This PR does not set it.
- **`--poll`'s CLI rejection of 0 and non-numbers has no test;** only the hook path is tested.
- **Since D59 has ended,** this setup PR needs another vendor's fresh-context review.

## Plain summary

A lab with nothing to do used to stop with nothing to wake it, so the whole loop waited on whoever was slowest. Now it waits with a timer and checks again on its own. Only the kill switch truly stops a lab.
