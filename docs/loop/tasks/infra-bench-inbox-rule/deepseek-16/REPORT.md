# REPORT: infra-bench-inbox-rule, deepseek-16 (repair: per-lab-lines marking rule)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `e4573a6de4e73eaa565a4783ad142764ad97b7e4`. Closes the item; issue is the #2028 umbrella entry that records Casey's bench decisions.

## What did not work, first

Answering the one-point other-vendor CHANGES record at `e4573a6`:

- **The line's marking rule lost every `all` message.** "Replace `waiting` with `read` or `answered`" assumed one status field per message; with several labs addressed by `all`, the first lab to mark it would hide it from the other four. Fixed from Casey's verbatim decision **"Per-lab lines (Recommended)"** (#2028 comment 6333307361, `2026-10-22T05:15:01Z`): each lab adds its own line (lab, `read`/`answered`, time, and for an answer a one-line reply); a one-lab message clears when that lab marks it; an `all` message clears only when every registered lab has a line, the last one making the change; nothing is deleted; never an owner release.
- **The line now also carries the reviewer's precision** from claude-27's review: a lab acts on `waiting` messages **that have no line from it yet**, not on ones it merely "has not answered", so a lab that marked an `all` message `read` does not act on it again while it waits for the others.
- **The active ORDER needed its own `## Owner approval`.** The previous repair's approval lived in `deepseek-14`; `owner-paths` judges the active ORDER, so this repair's ORDER carries both owner answers with their times — the failure was caught by `pr-check` and fixed before review the first time, and is carried correctly here.
- **The item's done-when text is being updated by Claude (claude-27), outside this item's scope**, so it still describes the old "replace `waiting`" rule in the item file; the ORDER says so and does not edit it. The AGENTS line follows Casey's decision, not the stale item text.
- **The PR remains non-doc-only** because of its receipts, so another vendor's exact-head APPROVE is required; that is why this repair requests one again.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `AGENTS.md` | docs | +1/−1 | the bench bullet rewritten to the per-lab-lines rule (add-your-line marking, one-lab vs `all` clearing, last-lab change, no delete, never an owner release) plus the "no line from your lab yet" precision |
| `docs/loop/tasks/infra-bench-inbox-rule/deepseek-16/{ORDER,REPORT}.md` | docs | small | this repair order (with both owner answers quoted) and report |

**Size:** one line replaced in `AGENTS.md`; no source, test or generated change.

## Measurements

- **No numeric result is claimed.** Checkable: `AGENTS.md` is 71 lines (gate: at most 80); the bullet contains the five clauses (add line, one-lab clearing, `all` clearing by last lab, no delete, never a release) plus the "no line yet" filter; `git diff` against the base lists exactly the files above.
- `loop gates --full` GREEN; `report --check` PASS; `pr-check --local` PASS including `owner-paths`.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule (`owner-paths`: approval recorded for `AGENTS.md`).
- `uor-harness check`: the AGENTS line cap and read path pass.

## Skills used

- **solution-integrity**: the one-status-field flaw was reproduced by the reviewer; the fix comes from the owner's recorded decision, not from a local guess.

## Still open

- **Claude's claude-27** updates the item's done-when to the same rule; it is outside this scope and should not conflict with the line.
- **An exact-head APPROVE from another vendor is required** (receipts make the PR non-doc-only); requested on #2028.

## Plain summary

- **The rule now survives a message sent to every lab.** Instead of one lab overwriting `waiting`, each lab writes its own line under the message; a message to one lab clears when that lab marks it, and a message to all labs clears only when every registered lab has marked it, with the last lab making the change.
- **It also tells a lab when to act**: only on `waiting` messages that do not yet have that lab's own line, so marking `read` does not make it act twice.
- **No other rule changes**: nothing is deleted, and a bench message is still never an owner release. The line is in the main checkout's inbox, found from the git common dir so it works from a worktree.