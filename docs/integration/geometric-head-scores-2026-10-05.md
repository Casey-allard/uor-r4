# Native compiler score replay and conditional-span defect — October 5

The zero-update replay reproduces every inspected retained prediction. It separates failed span capture from raw action selection and identifies contradictory training on a span branch that serving does not use. These findings change the next learning task; they do not establish improved model quality, chat, or a geometry-family verdict.

## Actual executed scope

Source `51be10a0c90b9453452423c52a5261c6736bb473`, Linux x86 CPU, executable SHA256 `023d1f8e46b4e20ea2e6265ac336b350c24905675034a80c26896153f01d2e86`. The unchanged predictor loads the six completed curriculum fits and all five checkpoints. Each checkpoint inspects 144 retained factor/repetition rows, 64 development writes and 44 deterministic training counterparts: 7,560 source-only predictions. No updates, checkpoint reselection, fresh prediction or reader generation occurs. The original interrupted relative1003 attempt remains excluded.

The diagnostic exposes integer-quarter bias/group/lane contributions, actual encoded root slots and strict maximal-span interval scores. It reconstructs the native head score and compares diagnostic action with independent `predict` and retained same-checkpoint output. It recomputes work for inspection; no production latency or cost claim follows.

## Corrected interpretation

At checkpoint64 across six familiar-frame/new-value panels, raw act is correct on377/384 and raw role on381/384. The final compiled action has correct act on171/384 and exact write on31/384. The old final `act_correct` count includes a downstream fallback: a failed positive-span search becomes Unresolved even when the act head selected a write correctly. Reporting it as a raw action-head failure was incorrect. Selected checkpoints are unchanged; their raw act count is307/384, including the initialized relative1002 selection. Checkpoint64 is a diagnostic, not a newly accepted model.

There are71 correct-start truncations in the checkpoint64 new-value panel. Every omitted suffix has negative total span margin; none is a pure-zero tie. Four-word repeated payloads have48/48 raw interval starts correct and48/48 truncated, while their final actions retain39/48 correct starts. Two-word repeats have only4/48 raw starts correct and2/48 complete writes. A continuation mechanism alone cannot solve every opening boundary.

The repeated-payload witness `I am a professional mariner.` gives `mariner` margin+29 quarters. In `I am a professional mariner mariner.`, the first margin is−3 after next-word contribution changes to−32; the second is−41. Such contributions describe the learned decision, not evidence that the neighboring-word or geometric transport family should be removed. The earlier six-run checkpoint64 new-value range is corrected to0–13/64, including product1002 at0/64.

## Training/serving branch mismatch

The fitter trains span CE and uses it for development selection on all512 training rows, including queries and NONE prose. Serving returns before span evaluation for query/NONE. Nonwrite prose deliberately repeats values and quoted write phrases, giving the local span head contradictory labels for a branch that is unused when the act decision is correct.

All six source metadata receipts report965 unique span tuples,100 conflicting span-label tuples and0 conflicting turn signatures. Independent reconstruction of case-preserved current/previous/next lexical triples from the frozen512 sources reproduces965/100. Restricting span supervision to256 labelled writes leaves4 conflict groups:96/100 conflicts involve unused NONE branches. This lexical reconstruction corroborates the source diagnostics; it is not independent native encoding of all training roots or a causal training ablation.

For example, previous `ochre`, current `canyon`, next `in` is inside in `I am a ochre canyon in my line of work.` but outside in `I heard the phrase ochre canyon in fiction.` The identical local span inputs cannot distinguish the distant frame. The remaining four write-only conflicts involve `a` after `in` and before `warm`, `blue`, `bright` or `gentle`: a distant frame determines whether the article belongs to the payload. Thus conditional supervision does not establish universal sufficiency of local features.

## Next implementation and outcome decisions

Repair offline span supervision and its development criterion to condition on **labelled assert/update tasks**, with an explicit write-row mean. Preserve all act/role supervision, all query/NONE examples and actual final false-write evaluation. Never gate training loss on the model's predicted action; abstention must not evade span credit. Runtime receives only original source text and retains its own learned act/role decision. No gold span, selected record or supplied answer is added to serving.

Retain the historical all-row objective for exact replay and compare three paired head seeds per arm under declared fixed conditions. Freeze new fresh inputs before a successor fit; exposed panels remain development evidence. Report actual predicted store, all-bank replies and reader calls separately. A correct compiler with failed natural replies advances reader cue/realization learning; residual negative continuation motivates conditional complete-interval credit and learned native OPEN/APPEND/COMMIT capture. The four genuine local ambiguities justify frame-sensitive geometric state if they remain limiting. Do not repeat the old objective/dose, force capture to the end or retire geometry from this instrument defect. Shared GroundedSession files remain unchanged.

## Verification, cost and preservation

Nine focused library tests, two probe tests, eight curriculum-driver tests and release build pass; all six zero-update replays exit0. Independent saved-artifact arithmetic admits7,560 rows with8,938,809 checks and0 errors: coefficients, masks, contributions, native sums, intervals, predictions and source/checkpoint/phase identity. It does not rerun BPE, native encoding, optimizer/backward or independently recompute BLAKE3 seals. The actual driver seals and verifies its report roots. Metal is unavailable on this host, GPU unused.

Wrapper elapsed586.579676s, evaluation wrapper28.016806s; ten worker receipts total582.753725s (build/test556.975601s, replay25.778124s). Maximum sampled process-tree RSS2,958,950,400B. Complete90-minute card includes preparation, implementation, review, return and delivery; cumulative ledger1,277,835,028/1,278,600,000ms. Before use, projected incremental storage rose384→768MiB and aggregate reports128→384MiB; individual reports remain128MiB. CPU48, RSS8GiB, owned storage10GiB, free floor4GiB plus128MiB margin. No new fit is charged to this zero-update card.

Complete returned inventory103 files/202,837,702B matches SHA256 and size. Inner archive17,005,159B SHA256 `7c8c5ee9f4281e591748235efee0e5138168de7efd530b7bde02633032949887` is retained under the approved pod `/workspace/codex-uor-r4-20261004` and local research `~/uor-r4-local/workspace/research/geometric-head-scores-20261005`; cloud receipt is in machine evidence. Every original checkpoint, negative and interrupted parent is preserved. This is completed diagnosis and implementation instrumentation, not model qualification.
