# #2360 — Defect 6: the pre-training gate was citing a banned grader and retired rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T19:47:48Z
- **Updated:** 2026-10-13T19:54:48Z
- **Closed:** 2026-10-13T19:54:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2360
- **Merged:** 2026-10-13T19:54:33Z
- **Merge commit:** ff8bef6ed8a4bdcd4902ca64466217f0725f4237
- **Base / Head:** main / UOR-Foundation:deepseek/prep-panel-defect
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Decision: **KEEP** (tooling). Fixes Antigravity's onboarding finding 6.

## Why this outranked the port order

`prep new -m m1` **is the D22 gate** — a lab gets its sheet to `TEST FITNESS: FIT` before any compute. **A gate that cites the wrong rules is worse than no gate, because it produces a FIT sheet a lab will trust.** Three things were wrong at once.

### 1. The sheet presented a D28-banned grader as a valid panel

```
- panel: `v5-reply` (grader: chat-grade (graded by qwen2.5:7b, grade-replies, temperature 0, seed 1),
  frozen: yes)
```

**`frozen: yes` beside a banned grader reads as approved.** D28 names this grader specifically:

> "Panels are scored with deterministic checks only… A bar that can only be scored by a neural grader today is recorded **BLOCKED** and is not run. **This includes the v5 reply panel's `grade-replies` qwen2.5:7b grader.**"

The grader field now carries the **BLOCKED** status, the blocking decision, and the deterministic alternative — with the reason inline.

### 2. D23 cited a script that no longer enforces anything

The text ended `Interlock scripts/check_no_softmax.py`. **That script is retired** — the interlock is `uor-harness ratchet d23|d25` (Rust, #2346). **A stale citation reads as a control that exists.**

### 3. D28 and D29 were not on the sheet at all

`APPLICABLE_RULES` was hardcoded to D21–D24. **The rule that makes the panel BLOCKED was absent**, so a lab could not tell why. Both are now listed, with a comment that a rule list maintained in code will keep drifting from the decisions file.

```
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
prep new -m m1 panel line                       -> shows BLOCKED (D28) with the reason
prep new -m m1 rules block                     -> D21-D25, D28, D29; D23 cites the Rust ratchet
```

Record: `docs/labs/prep-panel-d28-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling and registry text only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`prep.py` is still Python.** This fixes what it *says*, not what it *is*.
- **D22 still appears twice** on the sheet — once from the rule list, once from its own section. Cosmetic, but it is a duplicated rule on a sheet whose job is to state them once.
- **The panel is still `frozen: true` with a BLOCKED grader.** Coherent, but a lab could still read `frozen: yes` and assume it is scorable. **The guard is text, and text is weaker than a check.**
- **Nothing enforces this.** A future edit could restore a neural grader and no rule would fail. **A `check` rule that refuses a neural grader in a panel's `grader` field is the real fix, and it is not here.**
- **`docs/harness/milestones.json` is a registry other labs edit.** I changed three fields without coordinating; a concurrent edit would conflict.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

