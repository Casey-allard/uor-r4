# ORDER: infra-bench-inbox-rule, deepseek-16 (repair: the per-lab-lines marking rule)

Lab: deepseek. Item: [infra-bench-inbox-rule](../../../../roadmap/items/infra-bench-inbox-rule.md) (#2028). Base: `e4573a6de4e73eaa565a4783ad142764ad97b7e4`.

## Goal

Answer the one-point CHANGES record on PR #2943 at `e4573a6`: write the marking rule from Casey's decision. The owner answered in Claude's chat and it is recorded verbatim on #2028 (comment 6333307361, 2026-10-22T05:15:01Z): **"Per-lab lines (Recommended)"** — each lab adds its own line under a message (its lab, `read` or `answered`, the time, and for an answer a one-line reply); a one-lab message leaves `waiting` when that lab adds its line; an `all` message leaves `waiting` only when every registered lab has its line, the lab adding the last line making the change; nothing is deleted; a bench message is never an owner release. The previous "replace `waiting`" wording loses `all` messages and is replaced in the reviewer's own-words requirement (D44).

## History

- deepseek-14 `84ee4646f` added the AGENTS line and item status; deepseek-15 fixed the path to the main checkout and the record claims, with the marking wording deliberately held for Casey's answer. This repair writes that answer in.

## Cancel probe (run first)

Class **instrument** (docs). The decision is recorded at #2028 `6333307361` (`2026-10-22T05:15:01Z`) and the current `AGENTS.md` line still says to replace `waiting`, which would hide an `all` message from the other labs. The one status-field flaw is real; the task stands.

## Files in scope

`AGENTS.md`, the item's `status:` line, and this task folder. The item's done-when text is Claude's claude-27 (outside this scope) and is not edited.

## Done when

- `AGENTS.md` carries the per-lab-lines rule in our own words: add a line per lab, one-lab message leaves `waiting` when that lab marks it, `all` only when every registered lab has a line (last lab makes the change), nothing deleted, never an owner release.
- The `## Owner approval` section in this ORDER names `AGENTS.md`, quotes both owner answers ("Addressed per lab (Recommended)" from 2026-10-22T04:05:13Z and "Per-lab lines (Recommended)" from 2026-10-22T05:15:01Z) and keeps `AGENTS.md` at most 80 lines.
- `pr-check --local` PASS including `owner-paths`; `report --check` PASS; `gates --full` GREEN; receipts re-run at the repaired head; CI green.

## Owner approval

`AGENTS.md` is an owner path. Casey answered in Claude's chat on 22 October 2026; Claude recorded both answers verbatim on #2028:

- Inbox addressing (comment `2026-10-22T04:05:13Z`): "Addressed per lab (Recommended)".
- `all` marking (comment `2026-10-22T05:15:01Z`): "Per-lab lines (Recommended)".

## Stop and ask if

- The decision's wording is ambiguous for a mixed one-lab-and-`all` message (it is not: one line per lab; the addressed set decides when `waiting` clears).
- A bar, status, constraint or spend change is needed.