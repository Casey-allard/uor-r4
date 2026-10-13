# INC-0027: Chart Cost Decomposition

## Status
Deferred fallback, not run.

## Trigger Condition
Only start this branch if:
- `INC-0026` (`H4`-Hopf geodesic pilot) does not improve enough, and
- the fallback shell-phase correction path also does not improve enough

## Purpose
Determine whether the remaining runtime loss is mostly:
- chart optimization cost
- route construction cost
- route fragmentation cost
- or another systems artifact

## Reason This Is Deferred
The deep math review indicates the project is still more likely blocked by a geometry mismatch than by a pure systems-cost issue.
