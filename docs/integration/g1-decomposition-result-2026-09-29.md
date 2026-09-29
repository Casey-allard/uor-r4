# G-decomposition: where G v1's held-out failures come from — result

September 29, 2026. References #973 and #820.
- **Lab:** Lab 2 (OpenCode), branch `lab/opencode/aerm-decompose` (`5f3de916` instrument, `e0eefd37` G-binding options).
- **Pre-registration:** [#973 comment 5881325638](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5881325638), posted before any fit; decision rule fixed there.
- **Evidence:** [g1-decomposition-2026-09-29.json](../evidence/g1-decomposition-2026-09-29.json) from the sealed roots; receipt `decompose.supervision.json` beside them.

**Status.** Complete; three seeds; all six runs completed and sealed. The G v1 gate reproduction (same conditions) is exact. The pre-registered decision rule selects **query-side address formation** as the next unit.

## What was measured

The instrument (`simulate` now records `WriteEvent`s; `address_cause` compares the model's own tag/trigger stream with gold on the query turn and on the last gold write of the query key; `DialogueEvaluation.failure_cause` counts every memory-arm failure). It changes no score, no training and no gate. Unit tests: `write_events_record_the_exact_clause`, `address_cause_names_the_query_and_write_slots` (14/14 `stack_aerm`).

**Reproduction of G v1** (identical conditions, `summarize gates=g1`): held-out Updated 0.1575 / 0.0733 / 0.0293 against the control's 0.4652 / 0.7729 / 0.8132 — the same numbers to four decimals.

**Held-out failure causes** (memory arm, per seed):

| Cause | Seed 1 | Seed 2 | Seed 3 | Combined (share) |
|---|---:|---:|---:|---:|
| **`QueryEntityTag`** | 587 | 523 | 370 | **1,480 (77.7 %)** |
| `WriteEntityKey` | 0 | 58 | 184 | 242 (12.7 %) |
| `QueryRelationTag` | 41 | 81 | 20 | 142 (7.5 %) |
| `Other` | 2 | 5 | 23 | 30 (1.6 %) |
| `WriteValueTag` | 5 | 4 | 0 | 9 (0.5 %) |
| `WriteMissed` | 1 | 0 | 1 | 2 (0.1 %) |
| `Evicted` | 0 | 0 | 0 | **0** |
| `WrongValueStored` | 0 | 0 | 0 | **0** |
| Total failures | 636 | 671 | 598 | 1,905 |

In-distribution (`dialogues`) failure maps are **empty** in every seed: the model fits the training phrasings perfectly. Combined tag causes (`Query*Tag` + `Write*Key`/`ValueTag`) are 98.3 %.

## Decision (pre-registered rule)

`QueryEntityTag` alone is **77.7 % ≥ 50 %**, so the rule selects **query-side address formation** — how a held-out query phrasing forms the address — as the next unit. The query's *entity* slot, not the relation slot (7.5 %), is the bottleneck; write-side causes are 13 % combined; eviction and wrong-value storage are absent.

## Reading

1. **The held-out failure is a role-binding failure on the query's entity.** On held-out templates the model often does not tag the entity token as an entity, so the always-on read never completes an address and the value slot cannot be supplied. That is the same lexical-identity failure Lab 1's independent prime-route analysis found at write time (`entity_atom=false`; held-out names are taught as "other" by the #1017 prose) — this diagnostic localises the *query* side of it.
2. **Attribution order caveat.** `address_cause` checks the query side before the write side, so a failure whose query *and* assertion both mis-tag the name counts as `QueryEntityTag`. The two views are complementary; the write-side share (12.7 %) is a lower bound.
3. **This is not a read-policy failure.** G v1's always-on read fires (2,294–2,706 events); the failures are upstream of it, in the tag/address stream. The store, its versions and its statuses are exact.
4. **Consequences already measured by Lab 1 on these checkpoints** ([comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883172773) context, `claude-prime-route/eval-2`): the GV1 register arm reproduces the probe exactly; the pre-registered `ExpertTrigger`/`ExpertTurnEnd` arms change nothing (consistent with `WriteMissed ≈ 0`); the not-pre-registered `ExpertSieve` is the first causal lever (0.271 / 0.440 / 0.330), and the gold ceiling is 0.68–0.84. It is not promoted (no arm reaches 0.90).

## Scope and limits

- One parent, one read policy, three seeds, a 1.4M probe on the synthetic relation world; the classifier is a diagnostic over model-vs-gold tag/trigger streams, not a causal intervention.
- The cause labels are one fixed ordering; a both-sides-wrong failure is attributed to the query side.
- **Absent-class keys are `Other`:** for `Absent`/`PreviousAbsent` queries there is no gold write of the key, so the classifier returns before any store comparison; a model that wrongly wrote an absent key is invisible to the cause map (observed `Other` = 30/1905).
- **The control arm's cause map is empty:** the failure path is gated on the memory branch, so the control's failures were not decomposed; its tag accuracy and trigger confusion are reported. The pre-registration's "control causes reported for the shared tag/trigger stream" is therefore not met for this run — recorded rather than retro-fitted.
- No serving, dialogue or general-language claim; checkpoints are saved (`opencode-g1-decompose/checkpoints`, 33 MB) and loadable.

## Resources

- Run 2026-09-29T00:32:03Z → 03:49:32Z, **11,849 s wall**; 3 processes × 2 Rayon threads; peak sampled RSS 2.24 / 2.25 / 2.61 GiB (3 GiB per-process guard not hit); roots ~112 KB each, checkpoints 33 MB. No paid compute; the slot was claimed only after the previous holder released it.
