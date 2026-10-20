# #2393 — harness.owner_rules_applied: the durable fix for D31/D32 going missing (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T22:57:27Z
- **Updated:** 2026-10-13T22:59:55Z
- **Closed:** 2026-10-13T22:59:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2393
- **Merged:** 2026-10-13T22:59:32Z
- **Merge commit:** 41bfc1687068a8d22b52b8eaca9a8e91342c44d0
- **Base / Head:** main / UOR-Foundation:deepseek/applicable-rules-check
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`harness.owner_rules_applied` — the durable fix.** Tooling only.

Decision: **KEEP** (tooling).

## The gap this closes

Last unit: **D31 and D32 were both issued and neither reached a pre-registration sheet** — and **D32 supersedes D25's floating-point allowance, so the sheet told a lab that scalar floating-point GPU work was permitted.**

**Nothing caught it, and the reason was structural: no rule compared the sheet's list against the registry.** `parity` keeps the two implementations' tuples equal **to each other**; **nothing kept either equal to the registry.** I found it by reading `AGENTS.md` when it changed.

## The rule

**Every owner rule scoped `all` must appear in the sheet's `APPLICABLE_RULES`.**

**It reads the tuple out of `prep.py`'s source rather than importing it** — importing the same tuple would **compare a value with itself**, and the check would pass whatever the registry said. If the tuple stops being a literal, the rule **says it could not read it rather than passing: a check that cannot run must not report success.**

```
on the tree           -> all pass
with D31/D32 dropped  -> FAILS: "owner rule(s) scoped to everything are absent from a
                         pre-registration sheet: D31, D32 - the sheet a lab follows does not state them"
```

Three tests, including one that **monkey-patches the source reader to prove the rule can fail.**

## It immediately found a second, real omission

**Five `OWNER-*` entries are also scoped `all` and none reaches a sheet:** no activations, no matmul, MLP scope, rule freeze, ternary/XOR.

**They are sub-clauses of D23 and D25's subject matter**, and the sheet lists the `D`-rules as the index — so **omitting them may well be the design. But a lab reading the sheet cannot tell the difference between "covered by D23" and "nobody added it."**

**I exempted them and named them in the rule's docstring rather than hiding the question behind a green check.** The clean resolution is a **distinct scope value** — a clause, not a rule a lab must be handed — and **that is a registry change affecting every lab, so it is a question rather than my call.**

```
python3 -m unittest discover -s tests/harness -t .   -> 322 passed (3 new)
uor-harness check                                    -> 0 errors, 56 rules
python3 scripts/check_no_softmax.py                  -> OK, no softmax added
parity.py                                            -> exit 0
```

Record: `docs/labs/owner-rules-applied-check-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **It checks one implementation's tuple.** `parity.render_agreement` keeps Rust's equal to Python's, so together they cover both — **but a change to Rust alone is caught by `parity` rather than by this rule.**
- **The `OWNER-*` exemption is a judgement I made and flagged, not a decision.** Until the scope question is settled, **a genuinely-omitted `OWNER-*` rule passes this check.**
- **It only knows the tuple's current shape.** A refactor that computes `APPLICABLE_RULES` makes it report "could not read" — **loud, but a false alarm rather than a finding.**
- **The rule is Python.** When `check` ports, it must come with it — **or the port drops the guard that protects every future owner rule from being silently omitted.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

