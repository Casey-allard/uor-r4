# Lessons learned (the mistake registry)

One file per durable lesson: `docs/harness/mistakes/<id>.md`. A lesson is **stored once,
retrieved automatically** when a future task touches the same files, topic or milestone,
and - where a machine can decide it - **becomes a guard rule** that runs in
`uor-harness check`.

This is how the project stops re-learning the same failure. The rules these files cite
(`prlint.pivot_at_three`, `prlint.prereg_before_compute`, ...) are the guards; the
evidence paths are the records that prove the lesson really happened.

## `class: assumption` - ruled-out beliefs, not just mistakes

Most entries record something that went wrong. Some record something that was **believed and turned
out false**, and those are worth keeping for the opposite reason: the next lab would otherwise
re-derive them. "The mesh does not exist as a coordination concept" is the type specimen - a tracked
search settles it in a minute, and without an entry here every new lab spends that minute again.

They use the same four sections, so `mistakes find` retrieves them beside the lessons:

- **What happened** becomes what was assumed, and by whom.
- **Symptom** is how the false belief shows up - a plan that depends on it, a question asked twice.
- **Lesson** is what the search actually showed, with the method, so it can be re-checked.
- **How to apply** is what to do instead of re-investigating.

An assumption entry should say **how it was checked**, not just that it is false. `guard: none` is
normal: a negative finding has nothing to intercept, and inventing a rule for one would be a rule
that checks nothing.

## Use it

```bash
uor-harness mistakes list                      # all lessons, block first
uor-harness mistakes find -f <files you touch> # what applies here, and why
uor-harness mistakes show <id>                 # the whole lesson
uor-harness mistakes capture --id <id> --title "<short>" \
    --severity block --class measurement --trigger "<phrase>" "<phrase>" \
    --paths 'docs/labs/**' --evidence docs/labs/<topic>-<date>/README.md \
    --guard prlint.prereg_before_compute
uor-harness check --only mistakes              # keep the registry honest
```

`mistakes capture` refuses to write a file that fails `mistakes.registry_valid`, and
refuses to overwrite an existing lesson without `--force`.

## File format

Front matter is the strict subset `scripts/harness/core.py` parses: scalars, inline lists
(`[a, b]`) and block lists. Nothing else.

```yaml
---
id: <kebab-id>                     # must equal the file stem
title: <short>
date: <YYYY-MM-DD>
severity: block|warn|info
class: process|evidence|compute|measurement|delivery|coordination
trigger: [<phrases that should recall this lesson>]
paths: [<glob patterns this lesson applies to>]
milestones: [m1, m2]
mechanism: <docs/mechanisms id, or empty>
guard: <rule id from the harness rule engine, or "none: <reason>">
evidence: [<paths to the records that prove the lesson>]
status: active
---
```

The body has exactly four sections, in this order. `## Lesson` and `## Symptom` are what
`registries.Mistake.lesson` / `.symptom` read (their first paragraph), and they are what
`mistakes find` prints:

```markdown
## What happened
## Symptom
## Lesson
## How to apply
```

Keep each section to a short paragraph or a few bullets. A lesson nobody reads is a lesson
not learned.

## Severity and guards

| severity | meaning | what it must carry |
|---|---|---|
| `block` | repeating this has cost the project real cycles; the rule must bite | a real guard rule id, or `none: <reason why no machine can decide it>` |
| `warn` | expensive but survivable | preferably a guard |
| `info` | worth knowing, cheap to ignore | no guard required |

Rules this registry is checked by (all in `scripts/harness/mistakes.py`):

- `mistakes.registry_valid` (error): every file parses, ids are unique and equal their file
  names, severity/class/status/date are valid, and the required fields are present.
- `mistakes.evidence_exists` (error): every cited `evidence:` path exists on disk.
- `mistakes.guard_covered` (warn): every `severity: block` lesson names a rule id that
  really exists in the engine - so a lesson cannot be written down and quietly forgotten.
- `mistakes.stale_check` (info): a lesson's `paths` still match a file in the repo.

## Rules for writing one

1. **Mine it, do not invent it.** Cite the record that proves it in `evidence:` (a dated
   `docs/labs/<topic>-<date>/README.md`, a negative in `docs/integration/current-state.md`,
   a decision in `docs/integration/DECISIONS.md`). An invented lesson with a plausible
   story is worse than no lesson.
2. **A trigger is a phrase someone would actually type** when they are about to repeat the
   mistake: `host fallback`, `only 1/3 seeds learned`, `report root`. Not a topic label.
3. **A path is a glob for the files the mistake would touch again**, so the lesson is
   recalled by file: `crates/**`, `docs/labs/**`, `scripts/harness/*.py`.
4. **If a rule can decide it, write the rule.** Then `guard:` names the rule id and the
   lesson stops depending on anyone's memory. `mistakes.guard_covered` warns when a
   `block` lesson claims a guard that does not exist.
5. **Correct in place.** If a lesson turns out to be wrong, fix the file and say why; do
   not delete the record of having held it.
