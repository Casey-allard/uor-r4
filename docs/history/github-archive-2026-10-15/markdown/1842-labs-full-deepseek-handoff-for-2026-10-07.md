# #1842 — labs: full DeepSeek handoff for 2026-10-07

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T14:55:21Z
- **Updated:** 2026-10-09T13:20:25Z
- **Closed:** 2026-10-07T14:56:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1842
- **Merged:** 2026-10-07T14:56:58Z
- **Merge commit:** 5e7dcf1e9792ab7eebc8b48ded4655e48ff8c00c
- **Base / Head:** main / UOR-Foundation:deepseek/handoff-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A standalone handoff for a session with **zero prior context**.

## Contents

- **§0–1** the state of the project in one paragraph, and a bootstrap that gets a new session read-in and running in 20 minutes
- **§2 the immediate task** — *make prediction produce correct output* — with the diagnosis, the evidence, and the ~20-minute experiment that answers it
- **§3** everything completed this session, with PR numbers, so it is not redone
- **§4** infrastructure traps, each with symptom, cause and fix
- **§5** method rules, each learned by violating it
- **§6** open threads and owners
- **§7** the one thing to do first

## The headline

**The response-entry fit's own test suite passes 12/12**, including assertions that `final_entry_correct = 1` and `final_exact_responses = 1` — that test trains entry, exports, re-derives the decision post-export, and generates the full response byte-for-byte.

**So prediction works when the entry objective is the whole objective.** At 512 construction replies the entry is **1,024 of 13,328 supervised positions (7.7%)** at **weight 1/24**, sharing all four parameter groups with the 12,304 later positions that dominate the gradient.

**Falling loss with zero correctness is a supervision problem, not a capability one** — answerable in ~20 minutes rather than 12 hours.

## Why the operational sections are long

This session lost four hours to a build flag, OOM'd twice on runs launched without pre-flight validation, and discovered every failure by asking rather than by being told. Those traps are written down so the next session does not pay for them again.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

