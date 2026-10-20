# #1596 — Let the chat wrapper run the LM phase from an initialised base

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T03:17:13Z
- **Updated:** 2026-10-09T13:17:08Z
- **Closed:** 2026-10-02T13:59:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1596
- **Merged:** 2026-10-02T13:59:58Z
- **Merge commit:** 2d589372c5714f8c993deedd54977ccfb8431d63
- **Base / Head:** main / UOR-Foundation:codex/chat-lm-from-init-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

`scripts/geometric-stack-chat-m1.sh` guarded its language-model phase on `[ -z "$INIT" ]`, so passing `INIT=` **skipped** the LM phase and went straight to response learning. That makes one specific arm impossible to express through the wrapper: **an initialised base plus a chat-v0 LM phase before the response phase**. Worse, it fails *silently* — the script runs the wrong arm rather than refusing.

That arm is the one this lane needs next. Two prior arms on the same 161-response panel win on different axes:

| Arm | Pooled response NLL | First-four NLL |
|---|---:|---:|
| S2 (from scratch + Phase A) | **2.520916** | 2.031832 |
| chat-from-prose-1 (prose init, **no** Phase A) | 2.649143 | **1.676303** |
| chat-v0-phase-a-1 (**Phase A**, no prose init) | 2.530589 | 1.994121 |

The prose base buys short-range fluency; the chat-v0 LM phase buys the response distribution. Combining them is the only configuration that could beat S2 on both, and the wrapper could not launch it.

## Change

The phase now runs whenever `LM_STEPS` is nonzero, starting from `INIT` when one is given. **The two existing meanings are preserved and are the only ones that change:**

| Invocation | Before | After |
|---|---|---|
| `INIT` unset | train new stack for `LM_STEPS` | **identical** |
| `INIT` set, `LM_STEPS=0` | response learning from `INIT` | **identical** |
| `INIT` set, `LM_STEPS>0` | LM phase **skipped** (silent) | **NEW** — LM phase from `INIT`, then response learning |

`lm_init` is an array, so **no `init=` argument is passed when `INIT` is unset** and the from-scratch command line stays byte-identical. Header stages updated.

## Exact-head evidence

**Reviewed head:** `1b39690f25d779cc632444c20767b2f1614e6a92` (one commit on `origin/main`).

- **`bash -n scripts/geometric-stack-chat-m1.sh` → OK.**
- **Executed end-to-end**, `INIT=<main-10000/geometric-s1/model> LM_STEPS=1 DIALOGUE_STEPS=1` (the new third row of the table):
  - `lm.log` **exists** and starts at step 0 — before this change the LM phase was skipped entirely and no `lm.log` was written;
  - `lm/attempt.json` records **`init=/Users/casey.allard/uor-r4-worktrees/stack-prose-reports/main-10000/geometric-s1/model`**, so the init was genuinely applied rather than ignored;
  - all later stages then completed: `dialogue/`, `integer/`, `summary.txt`, and the script's own next-command line printed. Exit 0.

## Scope

Script-only; **no Rust is changed**, so no compile or test-suite claim is made or needed. `LM_STEPS=0` and unset-`INIT` paths are unchanged by construction (the guard is the only edit and `lm_init` is empty when `INIT` is unset).

References #1512.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

