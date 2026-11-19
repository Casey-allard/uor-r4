# REPORT — paired query roles, assertion-bounded copies and equal-byte prediction

Lab: codex. Item: written-state-count-model (#4063). [ORDER](ORDER.md) · [Closure plan](../codex-9/CLOSURE-PLAN.md) · [Approved phases](../../../../../PLAN.md#approved-sequences).

## What did not work, first

The two model errors persist on independent questions. The role learner often assigns the wrong query word before the writer sees a source. The trained count base assigns the same next-token distribution to four different questions when their last four tokens coincide. Increasing TRAIN to 2.09 billion framed tokens lowered held bits and still missed the M1 dialogue floor by 4,696 predictions. Neither candidate qualified for a default change.

The assertion fence corrected all seven traced +2 starts on their original prompts. Independent fresh questions already gave the singleton oracle 384/384 exact and the fence changed no addresses on that panel. On held prompts the prior query selector sometimes nominates an unspecified person. Letting two improved value extractions agree added one unsupported copy. The role learner correctly separated delayed exclusion in all 192 held examples, but all 192 held “Regarding” referents went to the excluded role. These errors call for separate interventions on where the question begins and how the nominated word acquires a role. [Role diagnosis](ROLE-RESULT.md) · [Fence diagnosis](FENCE-RESULT.md)

## What changed

Three independent specialists tested paired roles, assertion-local value realization, and suffix counts. Each component had a non-author FIT before its deciding Wick runs. The released writer still receives an original source position, copies the original IDs with an integer cursor, and emits EOS at the registered release. A new experimental D11 schema loads the learned paired-role bank, requires exact whole-word historical matches, and permits a copy when all eligible matching facts produce completed, identical values. The existing schema and read-disabled controls remain available. Both original seed artifacts were used on identical cases. Repeated seeds on one question set do not increase the number of independent questions. [Writer registration](writer/SHEET.md) · [Pack and replay](writer/pack/main.rs)

For prediction, the S2 reader now supports up to four suffix token addresses, reads every count and mixture coefficient from its serialized native state, and supplies target mass and the greedy original ID to the client. The D87 runs use equal actual D11 byte lengths across trained context allocations within each dose. A separate ft/dev run checks all twelve against the same published M1 response-token rows. [Count protocol and results](count/RESULT.md) · [ft/dev protocol and results](ft-dev/RESULT.md)

## Measurements

Both seeds reproduced these end-to-end reply totals. Each split has 448 literal answers and 608 unsupported questions. “Copy” counts an actual copy on an unsupported question. A model that abstains may still generate an incorrect ordinary reply.

| Actual writer arm | Held exact | Held unsupported copies | Fresh exact | Fresh unsupported copies |
|---|---:|---:|---:|---:|
| Original ungated role | 117 | 608 | 368 | 608 |
| Previous learned action gate | 0 | 240 | 235 | 55 |
| Previous nominee, complete-value agreement | 0 | 191 | 112 | 96 |
| Previous nominee, agreement and fence | 0 | 192 | 112 | 96 |
| Paired role and agreement | 282 | 0 | 224 | 0 |
| Paired role, agreement and fence | 308 | 0 | 224 | 0 |
| Copy disabled | 0 | 0 | 0 | 0 |
| Forced correct source on supported rows | 448 | 0 | 448 | 0 |

At fixed paired nomination on held, 26 first errors at index zero moved to exact through EOS, with zero losses. On fresh that intervention changed zero. At fixed fenced realization on fresh, switching from the prior nominee to the paired role gained 168 exact answers and lost 56. Relative to the previous action gate, the joint arm gained 101 and lost 112, eliminated 55 unsafe copies, and finished eleven answers behind. Relative to the ungated role, it gained 40 and lost 184 while eliminating 608 unsupported copies. Gold source and blind prediction use separate data paths. Forced alternate-source rows can still be literally correct when repeated values coincide. [Row-paired control comparisons](writer/controls.jsonl) · [Crossed selectors](fence/SUMMARY.json) · [Seven localized faults](fence/HISTORICAL-FAULTS.json)

For “May held pink. Joe held brown. Set aside May, and describe the possession of Joe”, the historical query oracle anchors Joe at 32 and the matching fact at 9. The old value start reaches 13 beyond the delimiter at 12. With the boundary it starts at 11 and the actual writer returns brown, then EOS. On held “Regarding Zura” the role error is earlier: the excluded slot receives the proper token interval and the requested score is below threshold. Improving the copy cursor cannot recover that missing request. In historical transfer, 96 query-oracle failures remain at reply index zero on previously unseen connectors inside an otherwise correctly bounded assertion. [Historical and fresh examples](FENCE-RESULT.md) · [Literal endpoint traces](role/EXAMPLES.json)

| Full counting dose excluding held TRAIN fit | Depth one | Depth four |
|---|---:|---:|
| Artifact bytes | 125,998,708 | 125,998,708 |
| DEV-tune overall BPB | 2.767081 | 2.080167 |
| DEV-tune novel BPB | 2.866135 | 2.340089 |
| ft/dev greedy correct out of 14,287 | 1,168 | 2,306 |

The best full-dose model saves 24.82% overall and 18.35% novel bits against its matched one-token control. The previously measured ten-token reference scores 1.432318 and 1.747150 on the same development partitions. At 1%, depth three saves 15.65% overall versus depth one at 93,180,496 bytes. Across the twelve ft/dev comparisons the device and reader agree on all 171,444 predictions, including target and winner masses. Both tested count clients now emit text on the four frozen questions. Their common sixteen-token continuation begins “ We need answer.” and hits the limit. All eight first errors occur at reply index zero, including opposite Dan/Ben requests. On raw contexts sharing only the final token 79, the ordinary client reproduces the distinct device winners and Q32 masses. The suffix representation discards the differing content of the four text questions before generation begins. [Ordinary text](writer/count-text.jsonl) · [Raw context controls](writer/count-device.jsonl)

## Checks run at the head

The independent repeat, GPU/Rust identity and refusal records are in the component reports. The production client uses the packed seed-specific policy, and the exact-copy and count paths are exercised through their normal one-turn command. State and output receipts are linked in [client verification](writer/RESULT.md). The bound role pack rejected a bank from the other seed. CUDA training and evaluation used the existing Wick 4090 with a process lock. The count component measured 62.63 seconds end-to-end for the largest dose; this includes counting/fit/score instrument work and does not measure autoregressive speed. See the [role archives](ROLE-RESULT.md), [fence archives](FENCE-RESULT.md), [count preservation](count/RESULT.md) and [ft/dev preservation](ft-dev/RESULT.md).

## Skills used

The native-space-cycle and uor-research-architect reviews bound geometry, serialized state, controls, source handoffs, and first-error attribution for this unit.

## Still open

P1(a) has scoped 1% raw-count and reload evidence. P1(b) needs the deployable S2/S5/S6 candidate within the approved size and sealed score. Its development gap is already large enough to investigate without spending DEV-test. Claude's [parallel P1(c) report](https://github.com/UOR-Foundation/uor-r4/blob/d9b084155bd759cc030192c3dbea49cf86a19217/docs/loop/tasks/written-state-count-model/claude-433/REPORT.md) found that the 24-pass fit ended above its own KN expert at full TRAIN. Its decision remains unread. P1(d) failed for these count-only arms. The next prediction unit must test the same normalized mass with and without S5, preserve competing expert identities through S6 fitting and compare against the trivial pure-expert options. A score above a feasible lower-loss component invalidates a ceiling interpretation.

The specific next P1 diagnostic freezes S2, the unigram/floor and the S5 candidate set, then counts ft/dev rows whose gold ID could win under any admissible mixture. Include fallback-only contenders under the actual tie-order rule. The seed-1 M1 header carries a relation reader, so an identity-transport Pick trial must declare and serialize that auxiliary state explicitly. Use the original relation decoder as a control. If even the optimistic reachable set misses 7,002 by the prescribed paired band, revisit representation before fitting S6. If it clears the bound, measure a TRAIN-fitted mixture on unchanged dialogue rows and held D87 text. This is a proposed discriminator. The qualifying fitted deployment follows an interpretable P1(c) result.

On recall, first freeze the existing role bank and move only the admissible candidate region to the actual question under an explicit oracle. Measure exact role, other-word nomination and no-request abstention before the writer. A residual requested/excluded inversion within the correct region calls for ordered role evidence. A correct-role but early in-assertion value needs its own connector experiment. Keep copy-through-EOS on extractive replies distinct from the later action to resume free prose. The pinned LexLean operator uses a different nibble packing order and tie behavior on the inspected path. Resolve with shared integer test vectors before transferring any proof to the deployed learner. [LexLean correspondence and next test](ROLE-RESULT.md)

P2 retains the three equal-byte transport controls after P1(a,b); P3 retains its pointer and reply gates after P1 plus P2 or the prescribed alternate. P4 serving qualification and P5 transport training keep their approved dependencies. A time to useful conversation cannot be inferred from one minute of counting or the duration of a selector test. The approved resource envelopes remain at most one GPU-day each for P1 and P3 and two to six GPU-days for P5, plus implementation and any experiments reopened by failed gates. [Roadmap and estimates](../codex-9/TRAINING.md)

## Plain summary

When given the right address, the fixed writer answered every supported request on both independent panels. We can now identify whether an incorrect reply began at the role, the candidate value or the fallback text predictor. The main unresolved question is how to choose the right expert from the live prompt while preserving both the trunk's dialogue competence and the count model's held-text gains.
