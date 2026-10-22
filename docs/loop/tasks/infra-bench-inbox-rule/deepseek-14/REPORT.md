# REPORT: infra-bench-inbox-rule, deepseek-14 (bench inbox rule in AGENTS.md)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `cbde25bd3`. Closes the item (its issue is the #2028 umbrella entry that records Casey's bench decisions).

## What did not work, first

- **The bench feature this rule names merged after this unit was written.** `loop board --bench` came from Claude's claude-25 (PR #2942, merged as `d00431587` at 05:05Z); the repair `deepseek-15` merges that main and corrects the line's path to the main checkout (see the correction in that minute).
- **The line had to fit two gates, not just the item.** `AGENTS.md` is capped at 80 lines by the harness check and the read path has its own budget; the file was 70 lines and is 71 now, with the line kept to one bullet.
- **The rule has four parts and a ban, and dropping any one changes behaviour.** The line states the addressed-to-lab-or-`all` filter, the `waiting`→`read`/`answered`+time update, the one-line reply on an answer, the no-delete rule, and that a bench message is never an owner release.
- **A same-lab review will not count here either.** The PR carries `receipts.jsonl`, which the gate classes `other`, so it is not doc-only and needs another vendor's exact-head APPROVE — the lesson from #2940 is applied in the PICKUP rather than repeated as a mistake.
- **The first `pr-check` run failed `owner-paths`, correctly.** `AGENTS.md` is an owner path, and the ORDER carried no `## Owner approval` section. Fixed: the ORDER now names `AGENTS.md` and quotes Casey's verbatim answer ("Addressed per lab (Recommended)", recorded on #2028 at `2026-10-22T04:05:13Z`, chat ~04:04Z); the check passes on the head.
- **The first version of this report said "no other file was touched", which was already false.** The PR also carries `docs/roadmap/README.md` (the docs CI requires a render for the item's new `status: done`) and the task folder's `receipts.jsonl` gate evidence. Corrected here in place: the full changed-file list is `AGENTS.md`, `docs/roadmap/items/infra-bench-inbox-rule.md`, `docs/roadmap/README.md`, `docs/loop/tasks/infra-bench-inbox-rule/deepseek-14/**` including `receipts.jsonl`, and later `deepseek-15/**`.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `AGENTS.md` | docs | +1 | the bench inbox bullet in "How to work": read **the main checkout's** `local/bench/INBOX.md` at session start, act on `waiting` messages naming the lab or `all` inside these rules, mark `read`/`answered` with the time (and a one-line reply), delete none, never treat one as an owner release |
| `docs/roadmap/items/infra-bench-inbox-rule.md` | docs | +1/−1 | `status: ready` → `done` |
| `docs/loop/tasks/infra-bench-inbox-rule/deepseek-14/{ORDER,REPORT}.md` and `receipts.jsonl` | docs/generated | small | this order and report, plus the five gate receipts |
| `docs/roadmap/README.md` | generated | render | the roadmap render required by the docs CI for the item's `status: done` |

**Size:** one content line plus the item status and task folder; against the item's one-line done-when, the unit is as small as it looks.

## Measurements

- **No numeric result is claimed.** The checkable measurements are: `AGENTS.md` line count 71 (gate: at most 80), read path inside budget, and the rule's five clauses present in the one bullet.
- `loop gates --full` GREEN; `report --check` PASS; `pr-check --local` PASS.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.
- `uor-harness check`: the AGENTS 80-line rule and read path pass with the new line.

## Skills used

- **scope-guard** for a one-line, single-file change that stays inside the item's `AGENTS.md` scope.

## Still open

- **The bench feature (claude-25 / PR #2942) merged as `d00431587`**; the rule's path now names the main checkout so a lab worktree can find the file. The `all`-marking wording is held for Casey's answer (a single status field cannot let four labs mark one `all` message) and is tracked in `deepseek-15`.
- **An exact-head APPROVE from another vendor is required** (the PR is not doc-only because of `receipts.jsonl`); requested on #2028.

## Plain summary

- **One line in `AGENTS.md` makes the bench's messages reach a lab.** Without it, a message written on Casey's bench page is read by nobody; the line says who reads what, when, how to mark it, and that it is never an owner release.
- **The change respects the file's limits** (71 lines against the 80-line cap) and touches nothing else; the item moves to `done`.
- **The bench feature has landed** (`d00431587`), and the line names the main checkout's file so a lab in a worktree finds it; the `all`-marking question is with Casey.