# REPORT: infra-bench-inbox-rule, deepseek-15 (repair: main-checkout path and record claims)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `e491d1162dac93bb208f5081056ef53bfa3b857d`. Closes the item; issue is the #2028 umbrella entry that records Casey's bench decisions.

## What did not work, first

Answering the counted other-vendor CHANGES record at `e491d1162`:

- **The path in the line could not resolve from a lab's worktree.** `local/` is git-ignored and per checkout; the bench writes the main checkout's `/Users/casey.allard/uor-r4/local/bench/INBOX.md`, and every lab works in `.worktrees/<lab>-…`. A lab reading `local/bench/INBOX.md` from its own worktree would find no file and conclude there were no messages. Fixed: the line now names **the main checkout's `local/bench/INBOX.md` (the folder above `git rev-parse --git-common-dir`)** — the same resolution the harness uses — and restores "inside these rules", which the first line dropped.
- **The report and order under-listed the changed files.** They said "no other file was touched" while the PR also changes `docs/roadmap/README.md` (the docs CI requires a render when the item status changes) and carries `receipts.jsonl` gate evidence. Fixed in place in REPORT-14's changed-file list and in this report: `AGENTS.md`, the item file's status line, `docs/roadmap/README.md`, both task folders and their receipts.
- **The report called the bench feature "open".** claude-25 merged as `d00431587` at 05:05Z, so the repair merges that `main` and the records now say merged.
- **The receipts bound `b9177670`, not the reviewed head.** Re-run at the repaired head and committed there; the receipt lines name the new head.
- **The `all`-marking wording is held for Casey, by the reviewer's instruction.** Each message has one status field, so the first lab to mark an `all` message `read` would hide it from the other four; Claude asked Casey how `all` messages should be marked and asked me to hold that sentence. The repair does not change the marking rule; it will be a one-line repair when the answer lands.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `AGENTS.md` | docs | +1/−1 | the bench bullet: main checkout's `local/bench/INBOX.md` via `git rev-parse --git-common-dir`; act "inside these rules"; marking sentence unchanged and held |
| `docs/loop/tasks/infra-bench-inbox-rule/deepseek-14/REPORT.md` | docs | +5/−5 | changed-file list completed, bench feature recorded as merged `d00431587`, held marking noted |
| `docs/loop/tasks/infra-bench-inbox-rule/deepseek-15/{ORDER,REPORT}.md` | docs | small | this repair order and report |
| merge commit | generated | — | fresh `origin/main` (`d00431587`, claude-25 bench feature) merged into the branch |

**Size:** a one-line path correction plus record fixes; no source or test change.

## Measurements

- **No numeric result is claimed.** Checkable: `AGENTS.md` is 72 lines after the merge (gate: at most 80); `git diff --name-only` against the base lists exactly the files above; the receipts name the repaired head.
- `loop gates --full` GREEN; `report --check` PASS; `pr-check --local` PASS (owner-paths included).

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule; `owner-paths` passes with the ORDER's `## Owner approval` section quoting Casey's "Addressed per lab (Recommended)" and `2026-10-22T04:05:13Z`.
- `uor-harness check`: the AGENTS line cap and read path pass.

## Skills used

- **solution-integrity**: the reviewer reproduced that the path could not resolve from a worktree; the fix names the main checkout rather than a relative path that only works in one place.

## Still open

- **Casey's answer on how `all` messages are marked** (Claude asked on #2028); the marking sentence is held and a one-line repair follows.
- **An exact-head APPROVE from another vendor is required** (the PR is not doc-only because of the receipts); requested on #2028.

## Plain summary

- **The rule pointed at the wrong file for a lab at work.** Labs run in worktrees, and `local/` is per checkout, so the line now says to read the **main checkout's** inbox, found from `git rev-parse --git-common-dir`, exactly as the harness resolves it.
- **The records now list every file the PR changes** (including the roadmap render and the gate receipts) and say the bench feature merged as `d00431587`.
- **One wording question is deliberately parked for Casey**: how four labs should mark the same `all` message when there is one status field. The rule stands as written until he answers.