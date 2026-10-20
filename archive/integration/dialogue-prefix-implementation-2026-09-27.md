# Complete-prefix dialogue learner: executed integration

References #973. The [selected learner](dialogue-prefix-learning-plan-2026-09-27.md)
now connects the retained R1d parameters to complete-response learning, sealed
checkpoint reload and actual exact-token replies. Two sequential **two-update
integration witnesses** completed. The substantive paired learning study remains
**NOT_RUN**. No model is promoted and native multiplier-free serving remains the
target; these width-576 checkpoints are offline F32 artifacts.

The [evidence packet](../evidence/dialogue-prefix-implementation-2026-09-27.json)
preserves both campaigns, executed response schedules, complete generation
records, source/artifact hashes, independent review and cost components.

## Implemented path

`training::dialogue_episodes` indexes the original token store and response mask.
It verifies the bound protocol's actual assistant-marker IDs, document/source
boundaries and genuine response EOS, then retains complete prefixes of at most
256 total IDs. Execution reproduces **129,486 response runs and 14,826 eligible
responses**, with no identical full/role-only prefixes. It preserves all six
source populations and the earlier audit's exclusion scope.

The full-prefix arm observes original BOS, request/history, marker and response.
The control observes original BOS, the same stored marker and the **same complete
response/EOS IDs**. Previous replies are context with zero loss. Right-padding
occurs after the last real target and receives zero weight. Response IDs are
drawn uniformly with replacement from a shared counter schedule. This changes
prefix content, response position and history length together; it is not a
content-only intervention or a reconstruction of historical window sampling.

`response_batch_gradients` computes a global response-token mean across whole-row
CPU shards. Each shard contributes in proportion to its supervised target count,
followed by one clip and AdamW update. The existing termination reducer remains
unchanged. Response-uniform sampling does not give each response equal loss
weight: longer sampled replies contribute more tokens.

`training::dialogue_learning` consumes a strict parameter-start adapter, creates
fresh Adam state, writes actual exposure and per-source counters, evaluates fixed
open development responses, saves and seals model/Adam/lineage metadata, reloads
through a narrow offline loader, compares fingerprints and predictions, and
generates replies with the bound tokenizer/protocol. Its new example is
`dialogue-prefix-fit CAMPAIGN NEW_REPORT_ROOT`. The committed source is
`1af89bf5b9be861f196905b50adae1d3b1d76bc1`; the executed binary SHA256 is
`e0d4b30f7e0d074e82ba3b3906988a0835581b6d29d9d846b73e03d8fb525693`.

The canonical loader and integer shape contracts are unchanged. The existing
public artifact replay API remains read-only; the training handoff is crate-private.
The foreign trainer and Google's conversational CLI files are preserved.

## Executed witness

Each arm independently loads the retained R1d's 21 arrays, parameter SHA256
`95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822`.
Both use width 576, **read coordinate width 64**, context/tensor time 256,
Quaternion/Dot/Full, batch 16, two CPU shards, learning rate 0.0001, decay 0.01,
data seed 20260927, and exactly two updates. Read width is not a 64-token memory
limit. Both have full causal access within the declared context.

| Executed quantity | Full prefix | Role only |
|---|---:|---:|
| Response visits / real EOS targets | 32 / 32 | 32 / 32 |
| Supervised response/EOS targets | 1,991 | 1,991 |
| Tensor positions | 8,192 | 8,192 |
| Real prefix positions | 3,382 | 224 |
| Padded positions | 2,851 | 6,009 |
| Initial open-development response NLL | 2.767759 | 2.767759 |
| Final open-development response NLL | 2.747973 | 2.753873 |
| Saved/reloaded last-batch NLL difference | 0 | 0 |

The two updates supervise 904 and 1,087 targets respectively in both arms.
Response IDs, selected-target hashes and the complete schedule-chain digest
match exactly. Starting parameters and fresh Adam fingerprints agree; both
parameter sets change, and each saved/reloaded parameter and Adam fingerprint
matches its pre-save state. Sixteen identical, already exposed development
responses supply 1,041 supervised targets per evaluation. Their small numerical
differences are integration observations, not a conditioning or language result.

The three unchanged open requests produce five replies per arm: greeting,
France's capital, and a three-turn name-memory exchange. All ten replies are
preserved and independently read. Neither arm answers Paris or recovers Alex in
the final memory turn. Both greeting replies use the same conversational
boilerplate with unrelated story/email framing. Full-prefix output has four cap
stops and one cycle; role-only has three cap stops, one cycle and one EOS. A real
EOS in an irrelevant answer is not useful conversation. Exact generated IDs and
explicit caller closures remain recorded across turns.

Fresh save/reload was executed. Although source implements source-bound resume
with an update-wise schedule hash chain, **next-update resume equivalence was not
executed by these witnesses**. No integer export or served width-576 path was run.

## Checks and complete cost boundary

Eight focused Rust checks pass: four episode cases, two strict offline checkpoint
cases, unequal-count shard/padding causality, and a retained canonical checkpoint
round trip. Independent reviews found and corrected a corpus error conversion,
an exclusive report-metadata filename collision and inconsistent schedule
hashing across resumes before model execution. The executable also retains the
64 MiB recurrence-worker stack used by the existing training command.

The two model processes took **39.554 seconds** internally and **40.528 seconds**
of supervisor wall time. Their four complete updates account for **20.817 seconds**
within that total; initial/final development, last-batch reload forwards,
checkpoint I/O and ten generated replies account for the remaining model path.
Peak child RSS was 4,767,744,000 / 4,057,464,832 bytes. These short contended-host
observations inform a later projection, not serving speed, typical training
throughput or energy claims.

Preparation, both build versions, focused checks, independent review, orchestration
repairs and delivery are charged as one elapsed interval in the shared ledger,
without adding overlapping subprocess times again. The phase projection reserved
three hours, 6 GiB model RSS, 512 MiB new internal material and 2 GiB shared SSD
growth. An initial supervisor launch failed before model/report creation because
the remote shell's older Python lacked `hashlib.file_digest`; portable chunked
stdlib hashing repaired only orchestration. An older cached library could not
link the report-only helper; the current compatible library did. Both attempts
remain in the cost record and are not model-quality failures.

The two build versions and both focused-check passes total 124.796 seconds of
supervised wall time. Retained witness roots occupy 137,766,211 logical bytes;
the phase's recorded files total 150,395,757 bytes at publication preparation.
These are file sizes, not physical allocation or reclaimed storage. The receipt
through 08:25:32 UTC charges 1,762,705 ms since the prior cursor, taking this
lab's cumulative elapsed to 28,348,803 / 57,600,000 ms and shared cumulative work
to 676,504,383 ms under the verified 722,400,000 ms allowance. The final
publication/delivery tail is charged separately in the owning issue; no time
allowance was extended for this implementation.

## Decision and next learning investment

The integrated learner is ready for a **separately projected, fixed 1,024-update
pair**, starting again from the retained parent with fresh identical Adam state.
The two-update descendants are integration artifacts, not selected parents.
Before that study, freeze an explicit source-stratified open-development
inventory and per-source reporting, the existing open request panel, optimizer,
common schedule and full cost. Keep checkpoint 1,024 as the sole endpoint;
intermediate observations inform interpretation rather than checkpoint selection.

Everyday Conversations and Constraints supply 89.4847% of eligible response runs;
their token share differs. Preserve the chosen training mixture, expose source
tradeoffs, and require actual request/history-correct output improvement beyond
both the retained parent and role-only condition before retaining a useful
conditioning candidate. Similar improvements, loss-only differences or a gap
caused only by harming the control are distinct outcomes. No automatic dose,
seed, decoder or validation sweep follows an unhelpful result. The deeper
geometric discovery work in other labs continues independently.
