# `harness.owner_rules_applied` — the durable fix for D31/D32 going missing

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the durable fix named in `docs/labs/prep-rules-d31-d32-2026-10-13/README.md`

## The gap this closes

Last unit I found that **D31 and D32 were both issued and neither reached a pre-registration sheet** —
and D32 **supersedes** D25's floating-point allowance, so the sheet **told a lab that scalar
floating-point GPU work was permitted.**

**Nothing caught it, and the reason was structural: no rule compared the sheet's list against the
registry.** `parity` keeps the two implementations' tuples equal **to each other**; **nothing kept
either equal to the registry.** I found it by reading `AGENTS.md` when it changed mid-session.

## The rule

**`harness.owner_rules_applied`** — every owner rule **scoped `all`** must appear in the sheet's
`APPLICABLE_RULES`.

**It reads the tuple out of `prep.py`'s source rather than importing it.** Importing the same tuple
would **compare a value with itself**, and the check would pass no matter what the registry said.
If the tuple stops being a literal, the rule **says it could not read it** rather than passing — a
check that cannot run must not report success.

**Verified in both directions**, which is the point:

```
on the tree          -> all pass
with D31/D32 dropped -> FAILS: "owner rule(s) scoped to everything are absent from a
                        pre-registration sheet: D31, D32 - the sheet a lab follows does not state them"
```

Three tests, including one that **monkey-patches the source reader** to prove the rule can fail.

## And it immediately found a second, real omission

**Five `OWNER-*` entries are also scoped `all` and none reaches a sheet:** no activations, no matmul,
MLP scope, rule freeze, ternary/XOR.

**They are sub-clauses of D23 and D25's subject matter**, and the sheet lists the `D`-rules as the
index — so **omitting them may well be the design.** But **a lab reading the sheet cannot tell the
difference between "covered by D23" and "nobody added it."**

**I exempted them and named them in the rule's docstring rather than hiding the question behind a
green check.** The clean resolution is a **distinct scope value** — a clause, not a rule a lab must be
handed — and **that is a registry change affecting every lab, so it is a question rather than my
call.** The rule says so where the next reader will find it.

## Verification

| check | result |
|---|---|
| on the tree | all pass |
| with D31/D32 dropped | **fails, naming both** |
| `python3 -m unittest discover -s tests/harness -t .` | **322 passed** (3 new) |
| `uor-harness check` | **0 errors**, 56 rules |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | exit 0 |

## Limitations, stated

- **It checks one implementation's tuple.** `parity.render_agreement` keeps Rust's tuple equal to
  Python's, so together they cover both — **but a change to Rust alone would be caught by `parity`
  rather than by this rule**, and that is worth knowing when reading a green result.
- **`OWNER-*` exemption is a judgement I made and flagged**, not a decision. Until the scope question
  is settled, **a genuinely-omitted `OWNER-*` rule passes this check.**
- **It only knows the tuple's current shape.** A refactor that computes `APPLICABLE_RULES` rather
  than listing it makes the rule report "could not read" — **loud, but a false alarm rather than a
  finding.**
- **The rule is Python.** When `check` ports, it must come with it — or the port drops the guard that
  protects every future owner rule from being silently omitted.
