# REPORT: infra-loop-harness, claude-35 (a counted CHANGES blocks until a new head or the same lab approves)

Lab: claude. Base: `fb6df367b`. Order: [ORDER.md](ORDER.md), committed first (`fe80b378c`). The cancel probe found that at `fb6df367b`, records CHANGES (codex) then APPROVE (kimi) at one head **pass** `loopreview::gate`: it read only the last counted record (`loopreview.rs:447`). So the task stands.

## What did not work, first

- **Another lab's APPROVE could clear a CHANGES.** The gate passed whenever the last counted record was an APPROVE, so one lab's open findings could be cleared by a second lab's later APPROVE at the same head. Casey ruled this out at 17:51Z ("No: CHANGES blocks").
- **`loop next` would have stalled the author.** Its `repairs` source looked only at the last counted record. Had the gate alone changed, a PR with a standing CHANGES followed by another lab's APPROVE would be blocked by the gate, while `loop next` told the author there was nothing to repair. That is the stale-loop pattern this week's fixes remove, so both now read one function.
- **My first negative-gate command reverted the new `loop_review` tests** along with the set-aside, which meant the first run proved nothing for that file. I restored the tests, then planted the fault as the gate's own one-line break instead (below).

## What changed

| Commit | What |
|---|---|
| `fe80b378c` | This ORDER, alone. |
| `f612d6296` | See the list below. |
| this REPORT's commit | This REPORT. |

What `f612d6296` changes:
- **`loopreview.rs`:** `standing_changes(input)` lists the labs whose last counted record at the head is CHANGES. `gate` passes when a record counts and none stands, and its fail line names the labs. A new head already clears everything, because a record counts only at the current head.
- **`loopnext.rs`:** `repairs` sends the author to repair when any CHANGES stands.
- **`loopprcheck.rs`:** the doc comment only.
- **Tests:**
  - new `planted_fault_another_lab_s_approve_does_not_clear_a_counted_changes` (`loop_review`);
  - a new case in `an_own_pr_with_a_counted_request_changes_is_a_repair` (`loop_next`), whose labs fixture gains `kimi` as a third vendor;
  - two expected fail lines updated to the new wording, because the specified behaviour changed.
- **New gate** `loop-review-changes-blocks`.
- **`docs/harness/README.md`** states the rule (two sentences).

## Measurements

None. This is harness behaviour, with no model run.

## Checks run at the head

- **Tests:**

  | Test | Passed |
  |---|---|
  | `--test loop_review` | 28 |
  | `--test loop_next` | 29 |
  | `--test loop_next_sources` | 9 |
  | `--test loop_prcheck` | 28 |
  | `--test loop_queue` | 20 |

  `loop queue` reads the gate through `pr-check`, so it needed no change.
- **Negative gates:** with the fix set aside or the fault planted, each new test fails, and passes again once restored.

  | Fault | Failing test |
  |---|---|
  | `loopreview.rs` and `loopnext.rs` changes set aside | the `loop_next` case, at `loop_next.rs:271` |
  | the gate's break (`last.clear()` before each insert, which restores last-record-wins) | the `loop_review` planted-fault test, at `loop_review.rs:306` |

- **Static checks:** clippy `--all-targets -D warnings`, exit 0. fmt: clean.
- **`loop gates --full`:** 10 GREEN, 0 RED. `--break all`: 10 CAUGHT.
- **Still to run:** `loop report --check` and `loop pr-check --local`. See the PR body.

## Skills used

scope-guard: one rule in `loopreview.rs`, the matching condition in `loopnext.rs`, their tests, one gate, and two sentences of the shared README.

## Still open

- **A lab that posts CHANGES and then goes quiet** blocks the PR until the author pushes a new head. That is the exit Casey named. Any other exit, such as a PM override, is a new owner call.
- **The merge queue's required checks do not run the review gate,** so a plain `gh pr merge` still bypasses it. Only `loop queue` enforces it. Making the gate a required check is an owner or CI decision.

## Plain summary

If one lab asks for changes on a PR's current version, the PR cannot merge just because another lab approved. It waits for the author's repair (a new version) or for the same lab to approve.
