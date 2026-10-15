# #1036 — Complete #973 layerwise-normalized retained readout rung

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T17:40:33Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-01T17:41:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1036
- **Merged:** 2026-09-01T17:41:40Z
- **Merge commit:** 92aadcf01623792a03b9d7bb03c4c0df96afbda4
- **Base / Head:** main / UOR-Foundation:issue-973-layerwise-normalized-readout
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and executes the sole frozen `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` successor, preserves its complete canonical evidence, and synchronizes the programme to the binding terminal branch.

Terminal: `LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`.

The candidate improved prompt sensitivity and fresh language behavior over qualified V1, but missed both predeclared prompt-capacity gain floors. Per the frozen contract, this PR ends the parameter-free readout ladder and makes one freshly frozen learned associative binding/readout the sole #973 successor. It does not run generation, reasoning, widening, tuning, or lowering; #954 remains blocked.

## Measured result

- formula: `E @ [N(h) + (g/sqrt(2))*(N(a1)+N(a2))]`, fixed `g=1` versus equal-work `g=0`
- zero new learned parameters or persistent state; unchanged recurrence/data/order/optimizer/dose and one tied vocabulary projection
- Apple Accelerate CPU, four threads, 2,730 steps, 5,241,600 presentations, 1,447.764 seconds
- V3 prompt gain: candidate `0.0286980210`, V1 `0.0073316237`, delta `+0.0213663973`, `339/512` wins
- missed floors: absolute `0.0433216988`; incremental `0.0253415693`
- fresh heldout: candidate NLL/top-1 `3.7126411677` / `31.661826%`; V1 `3.8850003883` / `29.728138%`
- retained-state removal cost: `1.3495375637` nats and 20,595 correct decisions
- every language/mechanics gate passed; fresh-process verifier reproduced 13/13 comparisons with no optimizer/training work
- result CID: `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`
- verification CID: `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`

## Changes

- adds the layerwise-normalized readout, V3 independent prompt population, campaign lifecycle, recovery/finalization path, and focused tests
- adds the canonical result record plus seven byte-identical raw envelopes
- updates all current-state entry points, programme/evaluation policy, vocabulary/glossary, ADR history, #954 blocker record, and predecessor evidence chain

## Local verification

- 52 focused Python tests passed
- Ruff passed on all eight implementation/test paths
- `python3 scripts/check_claim_wording.py`
- `cargo fmt --check`
- `git diff --check`
- seven canonical envelopes byte-identical to completed campaign artifacts and valid JSON
- independent result verifier passed all 13/13 bound comparisons

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

