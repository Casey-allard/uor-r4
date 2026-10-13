---
name: uor-research-architect
description: The repository-wide architect protocol that was already tracked at .agents/skills/uor-research-architect - governance, architectural invariants and delivery for any UOR-R4 task.
version: 1
harness: 1
kind: research
applies-to: [.agents/skills/uor-research-architect/SKILL.md, AGENTS.md]
when: [architect review, whole project, architectural invariants, delivery requirements, research architect]
requires: [context-hydration]
outputs: [an architectural judgement against the current source and live authorities, never a restatement of policy]
forbidden: [treating this page as the protocol - it is a pointer, and the protocol at the source path is authoritative]
milestones: [m1, m2, m3, m4]
---

## When to use

Use for a question that spans mechanisms or milestones: does a proposed change fit the
native geometric architecture, and does it survive the invariants in `AGENTS.md`. For one
bounded experiment, use `experiment-prereg` instead; for governance, `research-plan-cycle`.

## Do this

1. Read the protocol at its source, which stays authoritative and is not copied here:

   ```
   .agents/skills/uor-research-architect/SKILL.md
   ```

2. Load the rest of the context first so the judgement is against the live repo, not memory:

   ```
   python3 scripts/harness/cli.py brief -m <milestone>
   python3 scripts/harness/cli.py status
   ```

3. Record the review as a receipt so the owner can see it happened:

   ```
   python3 scripts/harness/cli.py skill use uor-research-architect -m <milestone> --outcome completed --note "<question>"
   ```

## Output

A scoped judgement: the mechanism or claim, the specific source path inspected, what the
architecture requires there, and the exact missing piece. Cite the file and line. Never a
general restatement of policy — that is already in `AGENTS.md`.

## Do not

- Do not duplicate the protocol into this file; one authority, linked.
- Do not answer an architectural question from this page alone — read the source and the
  mechanism brief, because both change and this page does not.
