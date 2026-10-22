# ORDER: infra-loop-harness, claude-31 (no idle stops)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `4bb1feede`.

## Goal

Labs park with nothing to wake them. Since the joins on 21 Oct, the longest silences on #2028 were 5.5 h for Kimi, 4.3 h for Claude, 2.8 h for Antigravity and 2.3 h for DeepSeek.

- `loop drive` exits on any STOP (`loopdrive.rs:676-678`).
- `loop next` has no WAIT state. A lab whose PRs all await review hits `open_pr_limit`, and gets `STOP held work` or `STOP empty queue` with no re-check time.
- The docs park a whole lab on a pending owner question.

The planted fault: **a lab whose open PRs all await review is told to STOP, with no time to look again.**

## Cancel probe (run first)

At `4bb1feede`, does `loop next` print anything but GO or STOP, or does `loop drive` re-check after a STOP? If either holds, the task changes. Class: object.

## Calls

- **`loop next`:**
  - STOP only for the kill switch.
  - Every other no-work case prints `WAIT <reason> — check again in <n>s` and exits 3, naming what is pending: own PRs awaiting review at `open_pr_limit` (by number), held items, dependency waits, or an empty queue. The re-check defaults to 600 s.
- **`loop next --wait [--poll SECS] [--max SECS]`:** asks again every `--poll` seconds until GO (0), STOP (1) or `--max` (3). Three read failures in a row give 2.
- **`loop drive`:** on WAIT it pauses `--poll` and asks again. STOP still ends it. The default prompt ends only on WAIT or STOP.
- **Docs:**
  - The loop page's stops section: only the kill switch stops, and a lab never ends a session on WAIT without a timer.
  - `AGENTS.md:70`: a pending owner question parks only its own item.
- **Gate** `loop-next-wait`, modelled on `loop-drive`. Its planted fault turns the WAIT back into STOP.

## Owner approval

`AGENTS.md` is an owner path. Casey's direction in Claude's chat at 2026-10-22T13:50:03Z is recorded verbatim on #2028 (comment 6342668364):

> "can you fix the loop harness errors because they are causing the labs to stop their loops and wait without timers to resume so they become stale... I don't want everything to get piled up behind a single review"

The one `AGENTS.md` line changed is the rule that parks a whole lab on a pending owner question. Its other half stays: never act on your own reading of it.

## Files in scope

- `loopnext.rs` and `loopdrive.rs` under `crates/uor-r4-harness/src/`, with their `tests/loop_next*.rs` and `tests/loop_drive*.rs`
- `AGENTS.md`
- Shared files
- This task folder

## Done when

- A test shows a lab at `open_pr_limit` gets WAIT naming its PRs, with exit 3, and fails with the fix set aside.
- The `--wait` tests run without real sleeping, and a drive test shows re-asking after WAIT and exiting on the kill switch.
- Clippy `-D warnings`, the whole harness suite, `loop gates --full` with `--break all` CAUGHT, `report --check` and `pr-check --local` all pass.

## Stop and ask if

- Exit code 3 is already used by `loop next` or a caller.
- A client's command limit makes `--wait` unusable for some lab.
