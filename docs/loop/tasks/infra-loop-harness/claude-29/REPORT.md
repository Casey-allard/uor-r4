# REPORT: infra-loop-harness, claude-29 (scope for ending D59 and for the no-idle fix)

Lab: claude. Base: `4bb1feede`. Order: [ORDER.md](ORDER.md), committed first (`ac84e726a`).

The cancel probe found no item whose `scope:` lists `docs/harness/d59.json` or `docs/harness/owner-paths.json`.

## What did not work, first

- **The loop parks labs with nothing to wake them, and three paths do it:**
  - `loop drive` exits on any STOP (`loopdrive.rs:676-678`). The page's promised 10-minute poll on an empty queue was never built.
  - `loop next` has no WAIT state. A lab whose open PRs all await review hits `open_pr_limit` (3) and gets `STOP held work` or `STOP empty queue`, with no re-check time.
  - Setup PRs count only Claude's record (D59), with no timeout.
  - Since the joins on 21 Oct, the longest silences on #2028 were Kimi 5.5 h (01:19→06:47Z), Claude 4.3 h, Antigravity 2.8 h and DeepSeek 2.3 h.
- **My own share of it:** an owner question I asked blocked my session from 08:24Z to 12:35Z, while two of Kimi's PM PRs waited on my D59 review.
- **I first wrote this ORDER in the owner checkout.** The shell ran from `~/uor-r4` instead of this worktree. My commit there swept in 22 staged files that were not mine, as `43013aa47` on local `main`. Nothing was pushed. I undid it with `reset --soft`, removed only my file, and checked that the staged diff was byte-identical to before. Every writing command now `cd`s to the worktree's absolute path and checks the toplevel and branch first.

## What changed

| Commit | What |
|---|---|
| `ac84e726a` | This ORDER, alone. |
| `fe185c1f5` | `infra-loop-harness`'s `scope:` gains `docs/harness/d59.json`, `docs/harness/owner-paths.json`, `docs/integration/DECISIONS.md`, `AGENTS.md`, `crates/uor-r4-harness/src/loopdrive.rs` and `crates/uor-r4-harness/tests/loop_drive*.rs`; nothing else in the item changes. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is a roadmap record, with no model run.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK (ready 39, held 26, decision 4, done 19). The render leaves `docs/roadmap/README.md` unchanged.
- Claim-wording, links, the copyright check, `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one item line and this task folder. The owner-path files themselves change only in claude-30, under the owner's quoted approval.

## Still open

- **claude-30:** end D59. It adds `docs/harness/d59.json` (`"d59": "ended"`), lists it in `owner-paths.json`, marks D59 ended in `DECISIONS.md`, and updates the D59 lines in `AGENTS.md` and on the loop page.
- **claude-31:** no idle stops. `loop next` gets a WAIT outcome and `--wait`, and `loop drive` re-polls instead of exiting.
- Kimi checks this PM edit.

## Plain summary

The loop could leave a lab sitting idle with nothing to wake it. Casey asked for that to stop, and for D59's single-reviewer rule to end. This one-line roadmap change lets the next two PRs touch the files that do both.
