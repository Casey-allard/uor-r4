# #1605 — Diagnose geometric value loss with one bounded signed-root residual

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T06:17:34Z
- **Updated:** 2026-10-09T13:17:33Z
- **Closed:** 2026-10-02T06:43:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1605
- **Merged:** 2026-10-02T06:43:43Z
- **Merge commit:** 0e6298c949a1369ff4038dc603fb675638518cb3
- **Base / Head:** main / UOR-Foundation:codex/geometric-value-residual-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The nearest K1 value projection loses29 correct seed1 answers under unchanged scores/NoRead/decoder. Add one bounded signed-H4 residual atom per existing R4 lane: first atom exactlyK1, zero residual fallback, checked i64sum/i32output, exact primitive states/roots preserved. The offline oracle is fixed-first greedy, not globallyoptimal K2 or a native learned producer.

Both saved-model comparisons completed on the same512 authored development rows. Baseline/K1/K2 answers: seed1 original100/85/97 and stress110/96/110; seed2 original128/128/128 and stress127/127/127. K2 adds26 correct seed1 answers versusK1, but still loses3 original baseline answers and exchangesone stressloss/gain. Seed2 baseline predictions are allretained, reversingK1 stressgain/loss. Retain all changed/prior successful rows. All927232 actual-coordinate bounds hold and64 chunks restorebaseline; independent audit checks250624 lane records/62656 reducerrows.

Validation: integerpair3, residualcompiler4, loadedK1/K2 integration1 passed; cargo fmt --check, releasebuild andclaimwordingpassed. Actualruns sealed and Rust-verified. Principal mathematician independently approved source and actualresults. Exact source7204cbb7/executec9c85161/report hashes are retained; later changes docs-only.

Retain boundedK2 for native valuelearning and end the alphabetcomparison here. The activeplan now makes differentiable actualstate exposure plus declared finite-action delayedcredit the next causallearningchange, followedby token/state/span-conditioned values. A sourcecounterexample shows tangentprojectioncan erase antipodal/delayedcredit; correction remains prospective, notclaimedexecuted. Offline donor-informedtraining/recompilation remainsuseful; no donorvector/oracle search entersfinalserving. NoRead/output areunfinished. No general-language, geometryadvantage, completeinteger-serving orenergyclaim. References #1512 and #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

