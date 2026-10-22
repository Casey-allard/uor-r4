# ORDER: infra-bench-inbox-rule, deepseek-15 (repair: main-checkout path and record claims)

Lab: deepseek. Item: [infra-bench-inbox-rule](../../../../roadmap/items/infra-bench-inbox-rule.md) (#2028). Base: `e491d1162dac93bb208f5081056ef53bfa3b857d`.

## Goal

Answer the counted other-vendor CHANGES record on PR #2943 at `e491d1162`: (1) the bench line names `local/bench/INBOX.md`, which does not resolve from a lab worktree because `local/` is per checkout — name **the main checkout's** `local/bench/INBOX.md` (the folder above `git rev-parse --git-common-dir`) and restore "inside these rules"; (2) the ORDER/REPORT claims must list every changed file (`receipts.jsonl`, `docs/roadmap/README.md`) and record that the bench feature merged (#2942 → `d00431587`), not "open"; (3) re-run the gates at the repaired head so the receipts bind it. **The `all`-marking wording is held for Casey** (a single status field cannot let four labs mark one `all` message; being asked on #2028) — do not change the marking sentence in this repair.

## History

- deepseek-14 `84ee4646f` added the AGENTS.md line, the item `status: done`, the ORDER/REPORT and receipts; the docs CI required a roadmap render; Claude reviewed at `e491d1162` and returned CHANGES with the path defect, two record updates and the gate-receipt re-run, and held the marking wording for Casey.

## Cancel probe (run first)

Class **instrument** (docs). The CHANGES record is counted at the current head `e491d1162`; the path defect is real (`local/` is git-ignored and per checkout, so a lab worktree's `local/bench/INBOX.md` is not the bench's file), and the record claims are checkable against `git diff --name-only`. The task stands.

## Files in scope

`AGENTS.md`, the item's `status:` line, and this task folder. Merging fresh `main` is required (it brings the merged bench feature `d00431587`).

## Done when

- The AGENTS bench line names the main checkout's `local/bench/INBOX.md` via `git rev-parse --git-common-dir` and says the messages are acted on inside these rules.
- Every changed file is listed in ORDER-15/REPORT-15 (AGENTS.md, item status, docs/roadmap/README.md, task-folder files + receipts) and #2942 is recorded as merged at `d00431587`.
- Fresh `main` is merged into the branch; gates are re-run at the repaired head and the receipts bind it.
- The marking sentence is unchanged and the ORDER/REPORT say it is held for Casey's answer.
- `pr-check --local`, `report --check` and `gates --full` pass; `AGENTS.md` ≤80 lines.

## Owner approval

`AGENTS.md` is an owner path. Casey answered in Claude's chat at about 04:04Z on 22 October 2026; Claude recorded the answers verbatim on #2028 at 2026-10-22T04:05:13Z, decision 3:

> Inbox: "Addressed per lab (Recommended)". Each bench message names a lab, or `all`, and only that lab acts on it. A bench message is never an owner release; those still happen in each lab's own chat. This goes into `AGENTS.md` through a PR.

## Stop and ask if

- Casey's `all`-marking answer arrives and changes the rule (then a one-line repair follows).
- The bench feature's actual write path differs from the main checkout's `local/bench/INBOX.md`.
- A bar, status, constraint or spend change is needed.