# REPORT: infra-loop-harness, claude-25 (feed the bench from every lab's work)

Lab: claude. Base: `a8ff5690e`. Order: [ORDER.md](ORDER.md), committed first (`d4a4437eb`). The cancel probe found that no harness command wrote bench-shaped files. `loop board` printed text or JSON only.

## What did not work, first

- **The first feed called AGENTS.md's whole opening paragraph "the project in one sentence".** On the real repository that pulled in three lines. It now takes the paragraph's first line (`d3c356de2`). The test that pinned the joined form was updated to the specified behaviour and now also asserts that the second line is absent (`072076e25`). I committed `d3c356de2` before that test's failure stopped me, and caught it on the next line of output.
- **The first column names and file lists matched the kits.** "Cancel if" and "Why / What was rejected" matched LoopKit, and the four page names in order matched CorvusWing: 3 and 2 eight-word runs. D44 keeps kit text out of the repository, so they are reworded (`272295b2e`). The bench finds a column by its first word (`cancel`, `reversal`), so nothing it reads changed.
- **The feed writes nothing from an owner checkout without `docs/harness/labs.json`.** It refuses at the lab check, so the live run used the claude-25 worktree as `--repo`.
- **The builder extended `doccheck.rs`'s flag list** (`--bench`, `--every`), a shared file it had not been handed. The ported-rules test required it.
- **The ORDER was edited after it was committed.** `272295b2e` rewords its goal sentence for D44. Nothing else in it changed.
- **The first review, with two lenses at `c2f6672bb`, found nine faults.** They were fixed in three commits:
  - The lab registry was read from the working tree, not `origin/main`.
  - One unreadable PR failed the whole refresh, which under `--every` would freeze every panel.
  - The review cell showed any APPROVE at the head, including a same-vendor self-review that the merge gate would not count.
  - A lab PR with no ORDER got no row.
  - `gh pr view` ran twice per PR.
  - A failed write left its temp file behind.
  - The refusal tests checked only the exit code.
  - No test read the `Reversal cost` column or the gate fields.
  - The live numbers in this REPORT came from a build older than the D44 rewording.
- **The first fix builder stopped at its context limit after one fix.** The rest went to two fresh agents, one after the other.

## What changed

| Commit | What |
|---|---|
| `b17ea7e71` | `loopbench.rs` writes the five files atomically into DIR, from roadmap items, open lab PRs with their ORDER and last review record at the head, lab presence, `docs/integration/DECISIONS.md`, committed receipts and local gate ledgers. A DIR inside the tracked tree is refused (exit 2), and `--every` below 30 is refused. `INBOX.md` is never touched. GitHub is read through a read-only `BenchGh`, with a fake for tests. `loopgates.rs` sends `loop board --bench` here, and `loopboard.rs` is not edited. Six tests are in `tests/loop_bench.rs`. |
| `8f628cc02` | The harness README's `loop board` paragraph. |
| `9cbbedac9` | Merges `origin/main`, which carries claude-26's scope lines. |
| `d3c356de2`, `072076e25`, `272295b2e` | The fixes listed above. |
| `63169ba3e` | The lab registry is read from `origin/main` (`show` and `labs::parse_labs`). |
| `c9be92579` | One unreadable PR becomes a row reading `ORDER unreadable: <reason>`, and the refresh goes on. A lab PR with no ORDER gets a `(no ORDER)` row. There is one `gh pr view` per PR. A failed write removes its temp file. The refusal tests assert the reason, an absolute in-repo DIR is refused, and the tests read `Reversal cost` and the gate fields. |
| `bc0accf5f` | The review cell shows only what the merge gate counts: `loopreview::main_context` once per refresh, a `GateInput` built as in `loopnext.rs`, then `loopreview::counted`. A record at the head that the gate does not count reads `review not counted (<lab>)`. |

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean, and whole-crate clippy with `-D warnings` is clean. `loop_bench` 10 passed. `loop_next` and `loop_review` 27 passed each. The library tests passed 662, with 1 ignored.
- **Negative gates.** Each fails with its fix set aside and passes, with a clean tree, once restored:
  - Lanes, run at `b17ea7e71`: with `item_status` forced to `ready`, `bench_lanes_follow_item_and_pr_state` failed at line 290 of that commit.
  - Registry: with the registry read set back to the working tree, `bench_reads_the_lab_registry_from_origin_main` fails.
  - Resilience: with the degrade arm set back to `return Err(e)`, `one_unreadable_pr_does_not_stop_the_feed` fails at line 440.
  - Counting: with the raw last record's verdict restored, `a_self_review_the_gate_would_not_count_is_not_shown_as_approve` fails at line 507.
- **Live, on the real repository, at `bc0accf5f`.** `--bench /Users/casey.allard/uor-r4/local/bench` writes five files (2 open PRs, 88 items):
  - Next: one row per lab PR, with its ORDER's done-when and stop bullets, and both reading `needs review`. Neither has a counted record at its head.
  - Blocked: the 4 owner decisions.
  - Tasks: dependency waits named.
  - State: every lab's presence.
  - Checks: the gate lines.

  `--every 120` runs on this build. A reviewer ran the bench's own `--check` on the earlier build's output and got 6 of 6 panels FILLED.
- Full harness suite at `072076e25` and at `272295b2e`: 1,010 passed, 0 failed each. At `bc0accf5f`: 1,014 passed, 0 failed.
- Claim-wording finds nothing in the changed files. The copyright check finds 0 LoopKit and 0 CorvusWing eight-word runs.

## Skills used

solution-integrity: the feed is generated from the sources the loop already keeps, so the bench cannot disagree with the roadmap, and no hand-kept copy of the state exists.

## Still open

- The bench's Next panel shows four rows, so with five labs one lab's current task appears only in the count.
- The real GitHub path (`gh pr view` and the contents API) has run once, live. Only the fake is under test.
- `infra-loop-pr03-render` already scopes `STATE.md` and `HANDOFF.md` as rendered views. Whether the bench feed should become that renderer is a call for Kimi.

## Plain summary

The bench Casey installed now shows every lab's work: what each one is doing, every roadmap task in its lane, the decisions waiting on him and the latest checks. It is all generated from the project's own records every two minutes, so it stays current without anyone keeping notes by hand.
