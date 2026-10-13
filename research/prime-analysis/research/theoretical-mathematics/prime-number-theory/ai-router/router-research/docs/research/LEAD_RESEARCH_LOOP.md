# Lead Research Loop

## Inputs
- `docs/DECISIONS.md`
- `docs/research/INCREMENTS.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/OPEN_QUESTIONS.md`
- `docs/governance/gates/*.md`
- `results/summary.csv`

## Required Outputs Per Cycle
1. A concise reading of what changed.
2. A decision on whether the current best-known route remains the leader.
3. The next increment specification.
4. Team assignments for that increment.
5. A user escalation only if the path is materially unclear.

## Default Loop
1. Read the newest increment and gate note.
2. Compare the result against the current best-known route.
3. Update `CURRENT_DIRECTION` if the lead route or interpretation changed.
4. Select the next increment from `OPEN_QUESTIONS`.
5. Assign work in `FLEET_ASSIGNMENTS`.

## Mandatory User Escalation
- Two branches are within noise but point toward different architecture families.
- A surprising result cannot be distinguished from benchmark artifact.
- The next decision would commit the project to a materially new thesis outside routing geometry.

## Escalation Standard
- When escalating, the lead researcher must still recommend one path and explain why.
