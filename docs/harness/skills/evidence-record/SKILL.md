---
name: evidence-record
description: Write the canonical record for a completed unit — the dated docs/labs README plus a newest-first current-state entry with Next — preserving negative results and exact identities.
version: 1
harness: 1
kind: research
applies-to: [docs/labs/*/README.md, docs/integration/current-state.md, docs/evidence/*]
when: [record a result, KEEP, REJECT, current-state entry, delivery cadence, negative result, canonical record]
requires: [panel-scoring, lesson-and-guard]
outputs: [docs/labs/<topic>-<YYYY-MM-DD>/README.md, one newest-first entry at the top of docs/integration/current-state.md, a Line and count declaration in the PR body]
forbidden: [a record that states proof where only measurement exists, rewriting an earlier record, logging only in an issue or chat, leaving a dated next in place of the live plan]
milestones: [m1, m2, m3, m4]
---

## When to use

Use for every completed unit — an experiment, a fix, a feature, or a negative
result — in the same PR that delivers it. Authority: CONTRIBUTING.md §
"Delivery cadence: every completed unit (owner, 9 October 2026)". Nothing is
logged only in an issue, a chat or a local file, and nothing is left for a docs
pass later.

## Do this

1. **Create the dated record** at `docs/labs/<topic>-<YYYY-MM-DD>/README.md`
   (sealed evidence goes to `docs/evidence/…`). Topic is kebab-case and specific,
   for example `read-binding-powered-2026-10-11`.
2. **Write the five required parts**, in this order:
   - the question the unit answered, in one sentence;
   - the exact artifact, data, config and operator identities — file paths, SHA
     or md5, seed, steps, device, and the command that produced the result;
   - what was actually run, including retries and the failed attempts;
   - the numbers with their scope: metric, panel, split, denominator, and what
     they are compared against;
   - the decision (`KEEP`, `REJECT`, or `NOT YET PROMOTED`) and the limitations.
3. **Label the evidence state** on every claim, using
   [formal_vocabulary.md](../../../formal_vocabulary.md):
   - **proof** — a definitional or machine-checked statement, e.g. an exact
     `Z[phi]` identity;
   - **measured** — observed on a named artifact, split and budget;
   - **hypothesis** — an interpretation not yet tested;
   - **NOT_RUN** — planned, registered, executed zero times;
   - **UNAVAILABLE** — cannot be read or reproduced (a missing artifact is not a
     failure of the model, and `UNAVAILABLE` is never model-quality evidence).
   Write "measured X on Y" for example, never "X works".
4. **Preserve the negative.** A rejected configuration lands with the same shape
   as a KEEP, and its scope is the recipe, dose, window, key, objective, base and
   steps that were tested. State explicitly that the mechanism is not closed
   (D22 §1). Do not erase or rewrite an earlier record; add a dated correction
   next to it.
5. **Add the current-state entry** at the **top** of
   [docs/integration/current-state.md](../../../integration/current-state.md):
   `## <one-line result> — <Month D>` (or the date-first form already in the file),
   two to five sentences, a link to the dated record, then the line declaration and
   the headline, then **Next:** with the next action. Newest-first; never insert
   below existing entries. Negative and inconclusive results get the same entry.
6. **Update every document the change makes stale** in the same PR:
   `git grep -n -i '<feature, flag, command, crate or number you changed>' -- '*.md'`
   — crate READMEs, `docs/geometry.md`, `docs/labs/*`, `CONTRIBUTING.md`,
   `AGENTS.md` and code comments that describe the changed behaviour. Update
   `STATUS.md`, `ROADMAP.md` and the milestone table in the body of
   [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028) when a measured
   position, best artifact or milestone status changed. Update the README only
   when a capability, headline result, milestone, command or mechanism changed,
   in its existing sections — never as a dated log.
7. **Declare the line in the PR body**: `Line: <name> · count n/3 · headline
   before → after`, and check it with
   `uor-harness lint check --body-file <pr-body.md>`.
8. **Land the artifact**: move used results to iCloud
   (`cloud-store put <lab> <dir>`), keep local-only data gitignored, and record the
   retained-evidence size and the disk floor in the record.

## Output

- `docs/labs/<topic>-<YYYY-MM-DD>/README.md` with question, identities, run,
  scoped numbers, decision and limitations.
- One newest-first entry in `docs/integration/current-state.md` ending in **Next:**.
- The stale-document updates, `STATUS.md`/`ROADMAP.md`/milestone-table changes
  where state changed, and the PR body's line declaration.

## Do not

- Do not write a record without the artifact/data/operator identities; a number
  with no identity cannot be compared or reproduced.
- Do not promote a hypothesis to a measured claim, or a measured claim to proof,
  to make the entry read better.
- Do not leave a negative result out, and do not silently replace a retracted
  number; keep it on the record with its correction.
- Do not put the running journal in the README, and do not duplicate a stage
  mirror between `project-track.md` and `current-state.md`.
- Do not let a dated "next" inside an old record override the live plan; the
  milestone's newest `OWNER DIRECTION` and `current-state.md` win.
