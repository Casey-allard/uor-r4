---
name: lesson-and-guard
description: Turn a mistake into a guard — capture the trigger, symptom, lesson and a machine-checkable rule id, and tell a configuration negative (allowed) apart from a mechanism negative (needs the owner).
version: 1
harness: 1
kind: research
applies-to: [docs/harness/mistakes/*.md, scripts/harness/checkrules.py, docs/integration/DECISIONS.md]
when: [we made a mistake, repeated failure, capture a lesson, add a guard, negative result, close a mechanism, regression]
requires: [evidence-record]
outputs: [a docs/harness/mistakes/<id>.md file with trigger symptom lesson and guard, a check rule id that fires on the mistake, a short note in the dated record that cites the mistake id]
forbidden: [a lesson with no guard, closing a mechanism on one configuration result, counting a tooling failure as a negative, recording the same failure as a note twice without a rule]
milestones: [m1, m2, m3, m4]
---

## When to use

Use when something failed twice, when a run closed a line, or when a review found
a breach that a document alone did not prevent. A lesson without a guard is a note
that will be forgotten again: `checkrules.py` exists so a lesson becomes a rule.
The closure rules are D22 §1–§2 in
[DECISIONS.md](../../../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened).

## Do this

1. **Separate the three kinds of failure before writing anything.**
   - **Configuration negative** — the tested recipe, dose, window, key, objective,
     base and steps did not work. Allowed; land it as a scoped record and keep
     going on the mechanism.
   - **Mechanism negative** — a claim that a novel mechanism (prime/zeta/R4/H4
     geometry, exact addressed memory, pointer/copy, read binding, softmax-free
     reads, protected or discrete constructors) is dead. **Needs an owner
     decision** on the milestone; a lab may only propose it in a pivot card.
   - **Tooling or numerical failure** — a crash, a solver tolerance, a missing CLI
     field, a host fallback. A **blocker to fix**, not a negative; it does not
     count toward D21's three.
2. **Capture the mistake** with its observable parts:
   `uor-harness mistakes capture --id <kebab-id> --title "<one line>" --severity warn --class process --trigger "<phrase a task description would contain>" --paths <glob> --milestones m1 --mechanism <mechanism id or none> --guard "<rule id>" --symptom "<what was seen in output or review>" --lesson "<what to do instead>"`
   `--trigger` and `--paths` are how the lesson is found again: they are matched
   against a task description and the files about to be edited.
3. **Write the four headings** in `docs/harness/mistakes/<id>.md`, in this order:
   `## Trigger` (the situation that produces the mistake), `## Symptom` (what it
   looks like in a log, a review or a record), `## Lesson` (the instruction that
   prevents it), `## Guard` (the rule id, plus the command that runs it).
4. **Write the guard as a machine check, not a resolution.** Add a rule to the
   harness rule engine (`scripts/harness/checkrules.py` or a feature module's
   `register_rules()`) whose id is dotted, e.g. `prlint.record` or
   `skills.frontmatter`, and whose result names the exact fixing command in
   `fix`. The guard field in the mistake file must name that rule id; if no rule
   can express it, write `guard: none: <reason>` so the gap is visible instead of
   implied. Do not edit `checkrules.py` from this skill's scope in another
   project's harness — propose the rule where that file is owned.
5. **Prove the guard fires.** Run it against the current tree:
   `uor-harness check`
   and, where the rule is staged-scope, `uor-harness check --staged`.
   A rule that cannot fail is not a guard. Report the rule id and the output in
   the PR that adds it.
6. **Cite the mistake from the record.** The dated record of the unit that made
   the mistake names the mistake id and its guard; `Rule.mistake` and
   `Rule.rationale` in the rule itself cite the same id, so a lesson, its guard
   and its evidence stay linked.
7. **A repeated failure becomes a rule, never another note.** If a second record
   describes the same failure, delete neither: add the rule, mark the second
   record as the guard's first catch, and reference both.

## Output

- `docs/harness/mistakes/<id>.md` with Trigger, Symptom, Lesson and Guard.
- A dotted rule id in the guard field, and the rule that implements it, with the
  command that runs it and the output it produced.
- A line in the dated record citing the mistake id, and a rule rationale citing
  the same id.

## Do not

- Do not record a lesson without a guard or an explicit `none: <reason>`.
- Do not close a mechanism on one configuration; state the configuration as the
  scope and leave the mechanism to the owner (D22 §1).
- Do not count a tooling, numerical or infrastructure failure as a negative, and
  do not let it add to the D21 count.
- Do not rewrite the old record to make the new lesson look inevitable; keep both
  texts as evidence.
- Do not invent a rule id that does not exist: a guard that cannot run is prose
  with punctuation.
