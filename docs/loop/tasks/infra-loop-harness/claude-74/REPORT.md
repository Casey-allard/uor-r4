# REPORT: infra-loop-harness, claude-74

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- One PR for all of phase 0 could not pass. No item scoped `CHARTER.md` or `docs/rules/changes.md`, so pr-check's scope rule would fail, and `loop queue` refuses on any pr-check failure. claude-73 (#3159, merged as `6ecb40c45`) added both paths to this item's scope first, and this branch was merged forward onto it.
- The first ORDER quoted all of the owner's answers and came to 5,437 B, over the 4,096 B read-path budget (pr-check `read-path` FAIL). The ORDER now quotes only the approval of this record. The answers themselves are quoted verbatim in the change-log entry.
- The first in-unit review (fresh context, at `5b1071e59`) returned CHANGES. It found five problems:
  - D67 misquoted the read-back option: lowercased, shortened, and with "Accurate" for "Accurate (Recommended)".
  - §5.10 still said "stop and ask".
  - D69's interim `owner log:` form was not marked as Claude's choice.
  - The §4.1 and §5.2 conflicts were not named.
  - The D41 wording implied the owner chose to keep that gate.

  All five are fixed at the next head. Its sixth note, that the review gate still refuses a same-lab record on a code PR, is under Still open.
- Claude's read-back to the owner had kept a 30-minute wait for another lab's review. The owner rejected it, and it is recorded only as a turned-down alternative (D65, D66).

## What changed

- `CHARTER.md`:
  - §5.1: the owner sets the charter, §3 with its exceptions, and the GPU caps; the single-lab rule; the pre-run and decision reviews; the owner's queue as a log.
  - §5.2: decision cards are approved by the lab's own decision review, and an unanswered PM ask becomes one.
  - §5.5: the review sits inside the unit, other labs audit after the merge, and the finishing PR writes `Closes #N`.
  - §5.10: sessions end on `loop next`; the supervisor; the content of the in-unit reviews; the order of work by unblock value.
  - §6: what retired on 24 Oct.
  - §3, §4 and the caps are unchanged. D41's donor pass still needs the owner's release at dispatch (§3.10).
- `docs/integration/DECISIONS.md`: rows D65–D71 appended, one decision each. D62–D64 are left to Kimi's open #3149.
- `AGENTS.md`: items 5 and 6, the decision-card line and the owner-only lines now match the charter. It is still 80 lines.
- `docs/rules/changes.md`: a 24 October entry with who decided, the owner's words verbatim and a side-effects table.

Size: docs 6 files (line counts in pr-check's size line at the top of the PR body); tests, code, generated and other 0. Against the ORDER's five files plus the task folder: 1.0.

## Measurements

Nothing was measured; this is a record change.

## Checks run at the head

- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: verdict PASS. Scope passes on the merged-forward base. Owner paths: approval recorded for all four. Read path: AGENTS.md is 80 lines, and the ORDER is under 4,096 B.
- `git diff --check origin/main...HEAD`: clean.
- `uor-harness loop gates --quick`: fmt-claim-wording, fmt-harness and claim-wording GREEN.
- `cargo run -q --locked -p xtask -- roadmap check`: 91 items OK.

## Skills used

None.

## Still open

The tooling still enforces the old waits. The next loop units, phases 1–3 of the stall review, bring it in line:

- `loop review` and the review gate still refuse a same-lab record on a code PR (D46). This is the first unit, since every lab's code PR needs it.
- `loop queue` runs that gate.
- `loop next` still offers WAIT and already-run premise probes, and does not rank work by unblock value.
- `loop close` still writes "Await the exact-head review" and only `References #N`.
- The owner's queue view and the restart-proof supervisor are not built.
- `docs/loop/README.md` still describes the old review steps, and the ORDER template still has a "Stop and ask if" heading. The charter wins where they disagree.
- §4.1 ("Stopping a mechanism needs the owner") and §5.2's owner as the PM seat's last fallback still conflict with D65 and D68. They are named in the change log's side-effects table for the next record unit.

## Plain summary

The project's labs are AI coding sessions from different vendors, each working the same research to-do list. Until today they were told to wait in three ways: for a review from a lab of another vendor before merging, for the owner's approval before any GPU training run, and for the project manager or owner to answer decision questions. On 23–24 October every lab ended up waiting on someone who was not looking, and the work stopped.

The owner ruled that no step may wait on anyone. A lab now reviews its own work with a fresh, adversarial reviewer before merging. Other labs check it afterwards. A lab decides its own GPU runs and research decisions after a review that must explain where the step fits in the whole project. The owner's role becomes reading a log he can reverse.

This change writes that ruling into the rulebook and the decision index. The tools that still enforce the old waits are changed next.
