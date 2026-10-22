# ORDER: infra-loop-harness, claude-25 (feed the bench from every lab's work)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c. Base: `a8ff5690e`.

## Goal

Casey installed the CorvusWing bench, a local page that reads five files from one folder (`.gates.jsonl` and four markdown pages: decisions, plan, handoff and state). He wants every lab's tasks, objectives, decisions and checks on it, side by side in the bench's layout.

Casey's decision, in Claude's chat on 22 Oct: "Generated feed (Recommended)". A new `uor-harness loop board --bench <dir>` writes the five files into the git-ignored `local/bench/` from the real sources, and nobody keeps top-level STATE, HANDOFF, PLAN or DECISIONS files by hand.

The planted fault: **every lab's open task appears in the bench's Next and Tasks panels in a shape the bench reads.**

## Cancel probe (run first)

At `a8ff5690e`, does any harness command write bench-shaped files? No. `loop board` prints text or JSON only. The task stands. Class: instrument.

## Calls

- `loop board --bench DIR [--every SECONDS]` writes the five files atomically into DIR, which must be outside the repository's tracked tree (under `local/` or outside the repo). It never writes `INBOX.md`. With `--every` it rewrites them on a timer.
- The sources, all read only:
  - roadmap items on `origin/main`;
  - each lab's open PRs, with their ORDER and the last review record at the head;
  - the lab presence `loop board` already reads;
  - `docs/integration/DECISIONS.md`;
  - committed `receipts.jsonl` and each lab's `local/<lab>/gates.jsonl`.
- The tables use the bench's column names. A Status cell starts with a word that puts the row in the right lane.
- The feed is new code in `loopbench.rs`. `loopboard.rs` is not edited, and the `--bench` routing goes in the `loopgates.rs` dispatcher.

## Files in scope

- `crates/uor-r4-harness/src/loopbench.rs` and `crates/uor-r4-harness/tests/loop_bench*.rs` (once Kimi adds them to the item's scope)
- The shared files: `loopgates.rs`, `main.rs` and `lib.rs` for the routing, and `docs/harness/README.md` for the `loop board` paragraph
- `docs/loop/tasks/infra-loop-harness/claude-25/**`

## Done when

- On a fixture with two labs, open PRs, items, decisions and receipts, the five files hold one Next row per open lab PR, one Tasks row per item in the right lane, the decisions, and one gate line per check.
- A DIR inside the tracked tree is refused, and `INBOX.md` is never written.
- The negative gate: with the feed's lane mapping set aside, the lane test fails.
- The full harness suite passes, as do `pr-check --local`, claim-wording, links and the copyright check.

## Stop and ask if

- `loopboard.rs` or any file outside scope must change.
- A source needs a GitHub write.
