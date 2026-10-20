# Defect 6: the pre-training gate was citing a banned grader and retired rules

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** Antigravity's onboarding finding 6 (#2028 comment 6153554053)

## Why this outranked the port order

`prep new -m m1` is **the D22 gate**: a lab gets its sheet to `TEST FITNESS: FIT` on the milestone
before any compute. **A gate that cites the wrong rules is worse than no gate, because it produces a
FIT sheet a lab will trust.** Three things were wrong at once.

## 1. The sheet presented a D28-banned grader as a valid panel

```
- panel: `v5-reply` (grader: chat-grade (graded by qwen2.5:7b, grade-replies, temperature 0, seed 1),
  frozen: yes)
```

**D28 names this grader specifically:**

> "Panels are scored with deterministic checks only… A bar that can only be scored by a neural grader
> today is recorded **BLOCKED** and is not run. **This includes the v5 reply panel's `grade-replies`
> qwen2.5:7b grader.**"

So the gate was handing labs a forbidden configuration with `frozen: yes` next to it, which reads as
approved. The grader field now carries the **BLOCKED** status, the blocking decision, and the
deterministic alternative, and shows the reason inline:

```
grader: BLOCKED (D28) - chat-grade by qwen2.5:7b is a NEURAL grader and D28 prohibits neural graders
until the owner declares the native model works end to end. D28 names this grader specifically…
```

## 2. D23 cited a script that no longer enforces anything

The rule text ended `Interlock scripts/check_no_softmax.py`. **That script is retired** — the interlock
is `uor-harness ratchet d23|d25` (Rust, #2346), and the Python file is kept only as dated evidence. A
sheet naming it **sends a lab to a file that enforces nothing**, which is the failure mode of a stale
citation: it reads as a control that exists.

## 3. D28 and D29 were not on the sheet at all

`APPLICABLE_RULES` was hardcoded `("D21", "D22", "D23", "D24")`. The owner has since added **D28** —
the rule that *makes the panel BLOCKED* — and **D29** (a banned mechanism must not exist in a compiled
module). **A lab reading the rule list would not have known why the panel was blocked**, which is
exactly what Antigravity had to work out from the decision text.

Both are now on the sheet, and the tuple says in a comment that a rule list maintained in code will
keep drifting from the decisions file — so a reader should check `DECISIONS.md` rather than trust it.

## Verification

| check | result |
|---|---|
| `python3 -m unittest discover -s tests/harness -t .` | **319 passed** |
| `uor-harness check` | **0 errors** |
| `prep new -m m1` panel line | shows `BLOCKED (D28)` with the reason |
| `prep new -m m1` rules block | D21–D25, D28, D29 listed; D23 cites the Rust ratchet |

## Limitations, stated

- **`prep.py` is still Python.** This fixes what it *says*, not what it *is*; the port is still ahead.
- **I did not fix "D22 appears twice"** on the sheet — once from `APPLICABLE_RULES` and once from the
  D22 section block. Cosmetic, but it is a duplicated rule on a sheet whose whole job is to state the
  rules once, and I am recording it rather than leaving it unmentioned.
- **The panel is still `frozen: true` with a BLOCKED grader.** That is deliberate — the *bars* are
  frozen and the grader is blocked, which is a coherent state — but a lab could still read `frozen: yes`
  and assume the panel can be scored. The `BLOCKED` text is the guard, and text is weaker than a check.
- **Nothing enforces this.** A future edit could put a neural grader back and no rule would fail.
  A check that refuses a neural grader in a panel's `grader` field is the real fix, and it is not here.
- **`docs/harness/milestones.json` is a registry other labs also edit.** I changed three fields; a
  concurrent edit would conflict, and I did not coordinate before changing it.

## What CI caught that I had missed: the Rust port had the same defect

**`parity.py` failed on the first push** — `render agreement: sheets differ`. The **Rust** `prep.rs`
carried its own copy of `APPLICABLE_RULES` as `["D21","D22","D23","D24"]`, frozen from before the port
began, so a Rust-rendered sheet **omitted the very rule that explains why its own panel is blocked**.

**The defect Antigravity reported was in the Python renderer; the same defect was still in the Rust
one**, and comparing the two is what surfaced it.

This is exactly what the D30 record predicted: *"`status.rs` still carries its own inline branch
classification… two implementations of one rule is the defect class this whole line keeps producing."*
It was right, and it happened again in a different file within the same day.

**Fixed by syncing, and recorded as a duplicate rather than a solution.** The durable fix is to render
from the registry so there is one list; until then, `parity.render_agreement` is what keeps them equal,
**which is why the drift was found at all.** A parity check that is scheduled for deletion when Python
goes is, right now, the only thing holding two implementations together.

Verified after the fix: **`parity.py` all comparisons ok, exit 0**, with the Rust and Python sheets
rendering identically.

