# The panel grader is now guarded by a check, not by text

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the follow-up named in `docs/labs/prep-panel-d28-2026-10-13/README.md`

## Why the previous fix was not enough

Defect 6 was fixed by correcting the `grader` field in `docs/harness/panels/v5-reply.json`. **That
corrected the text; it did not guard anything.** A future edit could put a neural grader back and **no
rule would fail** — and the field is not decorative: **it is what a pre-registration sheet prints**, so
a prohibited grader there propagates into every sheet citing the panel, with `frozen: yes` beside it
reading as approved.

I said at the time that *"the guard is text, and text is weaker than a check."* This is the check.

## The rule

**`prep.panel_grader_permitted`, error severity** — no panel's `grader` field may name a neural grader.

It reads every `docs/harness/panels/*.json`, and for each panel's `grader` field:

- **skips a grader that says it is BLOCKED** (or names D28 / "not run" / "prohibited") — **a grader
  that states the prohibition is honouring it, not violating it**, and failing there would punish the
  correct state;
- **fails on a neural grader** — matching `qwen*`, `llama*`, `gpt-*`, `claude-*`, and the phrases
  `neural grader` / `llm judge` / `model grader`.

The patterns are deliberately narrow for the same reason `_PROHIBITED_TEACHER`'s are: they target a
**grader field naming a model**, not the word appearing in a note about why the grader is blocked.

## Verified in both directions

That is the whole point of the change, so it was tested as a pair:

```
on the fixed tree                -> harness check: 1 rules, all pass
with qwen2.5:7b restored         -> FAIL prep.panel_grader_permitted: panel 'v5-reply' names a
                                    neural grader ('qwen2.5:7b') as its grader...
                                    fix: record the grader as BLOCKED with the D28 reason...
```

**The probe restored the exact string that caused Defect 6**, not a synthetic one. The rule catches
it and names the fix.

## Verification

| check | result |
|---|---|
| `prep.panel_grader_permitted` on the fixed tree | all pass |
| the same rule with the banned grader restored | **fails**, naming the grader and the fix |
| `python3 -m unittest discover -s tests/harness -t .` | **319 passed** |
| `uor-harness check` | **0 errors** |
| `parity.py` (after building in this worktree) | all comparisons ok, exit 0 |

## Limitations, stated

- **This is a Python rule.** It lives in `scripts/harness/prep.py`, so it protects the Python `check`
  that labs run today. **The Rust `check` does not have it** — when `check` is ported, this rule must
  come with it or the port silently drops a guard.
- **It is a pattern match, not a semantic check.** A grader could name a neural model in a way these
  patterns miss — a codename, an API alias, a model behind a wrapper. **This raises the cost of the
  mistake; it does not make it impossible**, and saying otherwise would be the same overclaim the rule
  exists to prevent.
- **Only the `grader` field is checked.** Teachers, comparators and control arms named elsewhere in a
  panel are covered by `prep.no_prohibited_teacher`, which is a different rule with different patterns;
  the two do not cover each other's gaps by design, but neither is complete.
- **It found no other violations.** `v5-reply` was the only panel naming a neural grader — worth
  stating, because a new rule that reports nothing is easy to mistake for a rule that works, which is
  why the probe above exists.
