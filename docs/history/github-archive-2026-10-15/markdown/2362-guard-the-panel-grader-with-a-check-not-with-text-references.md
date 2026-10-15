# #2362 — Guard the panel grader with a check, not with text (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T19:58:20Z
- **Updated:** 2026-10-13T20:00:49Z
- **Closed:** 2026-10-13T20:00:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2362
- **Merged:** 2026-10-13T20:00:39Z
- **Merge commit:** b89644c05f2c782d205b550e78dbdc5624f76167
- **Base / Head:** main / UOR-Foundation:deepseek/panel-grader-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Decision: **KEEP** (tooling). The follow-up named in the defect-6 record.

## Correcting the field fixed the text and guarded nothing

Defect 6 was fixed by correcting `grader` in `docs/harness/panels/v5-reply.json`. **A future edit could put a neural grader back and no rule would fail** — and the field is not decorative: **it is what a pre-registration sheet prints**, so a prohibited grader there propagates into every sheet citing the panel, with `frozen: yes` beside it reading as approved.

I said at the time that *"the guard is text, and text is weaker than a check."* This is the check.

## The rule

**`prep.panel_grader_permitted`, error severity.** It reads every `docs/harness/panels/*.json` and, for each panel's `grader`:

- **skips a grader that says it is BLOCKED** (or names D28 / "not run" / "prohibited") — **a grader that states the prohibition is honouring it, not violating it**, and failing there would punish the correct state;
- **fails on a neural grader** — `qwen*`, `llama*`, `gpt-*`, `claude-*`, and `neural grader` / `llm judge` / `model grader`.

Narrow for the same reason `_PROHIBITED_TEACHER`'s patterns are: they target **a grader field naming a model**, not the word in a note about why it is blocked.

## Verified in both directions — that is the point

```
on the fixed tree                -> harness check: 1 rules, all pass
with qwen2.5:7b restored         -> FAIL prep.panel_grader_permitted: panel 'v5-reply' names a
                                    neural grader ('qwen2.5:7b') as its grader...
                                    fix: record the grader as BLOCKED with the D28 reason...
```

**The probe restored the exact string that caused Defect 6**, not a synthetic one.

```
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
parity.py                                       -> all comparisons ok, exit 0
```

Record: `docs/labs/panel-grader-rule-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **This is a Python rule**, in `scripts/harness/prep.py`. It protects the Python `check` labs run today — and **the Rust `check` does not have it. When `check` is ported, this rule must come with it, or the port silently drops a guard.**
- **It is a pattern match, not a semantic check.** A codename, an API alias, or a model behind a wrapper would slip past. **This raises the cost of the mistake; it does not make it impossible**, and claiming otherwise would be the same overclaim the rule exists to prevent.
- **Only the `grader` field is checked.** `prep.no_prohibited_teacher` covers other fields with different patterns; the two do not cover each other's gaps.
- **It found no other violations** — `v5-reply` was the only panel naming a neural grader. Worth stating, because **a new rule that reports nothing is easy to mistake for a rule that works**, which is why the probe above is in the record.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

