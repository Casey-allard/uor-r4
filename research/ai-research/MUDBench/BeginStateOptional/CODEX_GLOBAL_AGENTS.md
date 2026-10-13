# ~/.codex/AGENTS.md

## Global Codex Working Agreements

These instructions apply to every repository unless a project-specific `AGENTS.md` or `AGENTS.override.md` narrows or overrides them.

---

## 1. General Operating Rules

- Work on one task at a time
- Prefer the smallest correct change
- Do not invent requirements
- Do not claim success without validation
- If scope is ambiguous, stop and ask for clarification
- Do not make broad refactors unless explicitly requested
- Do not silently change public interfaces
- Do not add dependencies without explicit justification

---

## 2. Repo Interaction Rules

Before editing code in any repo:

1. Identify the project root
2. Read the nearest applicable instruction files
3. Inspect the repository structure
4. Confirm task scope and allowed files
5. Plan the smallest valid implementation

Start every session with minimal orientation commands when appropriate:

```bash
pwd
ls
git status
```

Do not wander the repository without reason.

---

## 3. Validation Rules

Always run the narrowest relevant validation before finalizing.

Examples:
- tests for modified modules
- lint / format checks if configured
- deterministic checks if the project requires them
- replay or scoring checks when working on benchmark systems

Never skip validation just because the change looks obvious.

---

## 4. Output Rules

Unless the task says otherwise, final responses should include:

- concise summary of what changed
- files changed
- validations run
- unresolved assumptions or blockers

Do not write long essays when short structured output will do.

---

## 5. Safety and Boundaries

- Do not modify secrets or credential files
- Do not access unrelated directories without task relevance
- Do not edit governance or spec files unless explicitly assigned
- Do not perform destructive actions unless explicitly instructed
- Do not bypass repository-specific constraints

---

## 6. Determinism Preference

When working in systems involving simulation, scoring, CI, data, or evaluation:

- prefer deterministic behavior
- avoid uncontrolled randomness
- preserve stable ordering
- avoid hidden global state

---

## 7. Instruction Resolution Rule

Project-specific instructions take precedence over these global defaults.
If a repository contains an `AGENTS.md` or `AGENTS.override.md`, follow the closest applicable file and treat this file as the fallback baseline.

---

## 8. Final Principle

Correctness beats speed.
Validation beats confidence.
Bounded execution beats improvisation.
