---
id: infra-bench-inbox-rule
title: Bench inbox rule: each lab reads the bench messages addressed to it
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [AGENTS.md]
---

## Why it matters

Casey watches the project on the bench, a local page fed by `uor-harness loop board --bench`. Its message box appends messages to `local/bench/INBOX.md`. Until `AGENTS.md` names that file, no lab reads it, so a message from the bench reaches nobody.

## What is missing

`AGENTS.md` has no rule for the bench's messages. On 22 Oct Casey chose "Addressed per lab (Recommended)": each message names a lab, or `all`, and only that lab acts on it. A bench message is never an owner release; those happen only in a lab's own chat (CHARTER §5.1).

## Done when

`AGENTS.md`'s "How to work" list has one line saying that, at the start of each session, a lab reads the messages in `local/bench/INBOX.md` marked `waiting` and addressed to it or to `all`, acts on them inside these rules, then replaces `waiting` with the word `read` or `answered` and the time (an answer also carries a one-line reply), and deletes none. The line also says that a bench message never counts as an owner release.

## How to start

1. Read `AGENTS.md` and CHARTER §5.1 (owner releases).
2. Add the one line, in the words of `AGENTS.md`'s other rules, with no text copied from the kit (D44).
3. Run `uor-harness check` and `loop pr-check --local`.

## Rules that apply

- `AGENTS.md` summarizes the charter and never overrides it. The line adds a reading duty, not a new authority.
- Owner releases stay where §5.1 puts them.

## Sources

- [`AGENTS.md`](../../../AGENTS.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.1
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D44
- issue#2028 (Casey's bench decisions, 22 Oct)
