# REPORT — where the requested role disappears and which dialogue targets are reachable

Lab: codex. Item: written-state-count-model (#4063). [ORDER](ORDER.md) · [Previous instrument and model outcomes](../codex-12/REPORT.md) · [Closure plan](../codex-9/CLOSURE-PLAN.md).

## What did not work, first

Do not train the query-region detector from this configuration. The correctly labelled question region rescued 28 of 140 pre-registered assertion intrusions across the two original panels, below the strict-majority requirement. It preserved all previously exact requested endpoints. It recovered none of the lost requested endpoints on the previously fresh panel. The required answer is still present inside the model's candidate region on those prompts. Among the 192 held “Regarding” mistakes, the requested person still enters the excluded slot. Limiting positions cannot correct which slot the evidence supports.

## What changed

The role experiment restricts the positions eligible to win while computing each candidate's features on the original uncut prefix. Identical scores and threshold imply a retained unique winner survives this restriction. The Pick experiment freezes both count carriers and asks whether the original target appears among the possible winners under positive unigram weight, explicit suffix evidence and one raw geometric successor. The single fallback representative is valid when the separate uniform coefficient is zero, additions are nonnegative and ties prefer the smallest original ID. The three accompanying Lean theorems establish the ranking implications from those assumptions. The predecessor measurements, programme code and actual device-to-reader comparisons establish separate empirical facts. Both diagnostic sheets have independent pre-run reviews. [Role protocol](region/SHEET.md) · [Pick protocol](pick/SHEET.md) · [Checked ranking properties](Bounds.lean)

## Measurements

Each row below is per seed. Held and fresh are the two 1,056-row panels that exposed the earlier configuration. They contain 448 literal answers and 608 unsupported cases each. Both now serve as inspected diagnostic fixtures. Requested endpoints exist on 768 rows including some for which no value can be answered.

| Outcome | Held, frozen → query region | Fresh, frozen → query region |
|---|---:|---:|
| Exact requested endpoints / 768 | 528 → 556 | 384 → 384 |
| Correct abstention when no person is requested / 288 | 144 → 168 | 124 → 144 |
| Exact emitted reply including EOS / 448 | 308 → 316 | 224 → 224 |
| Unsupported copy / 608 | 0 → 0 | 0 → 0 |
| First error at index zero moved to exact | 8 | 0 |
| Previously exact reply lost | 0 | 0 |
| Supplied requested role only, normal fact search and fixed writer / 448 | 448 | 448 |

A separate row-level join on the original prefix digests found a perfect correspondence for these answerable prompts. With the true region, all 316 held and 224 fresh rows with an exact requested span produced an exact reply, and every incorrect requested span produced a first wrong token at index zero. There were zero lucky exact replies from a wrongly identified span and zero writer misses from a correct one. Of the 28 held role repairs, eight occurred on answerable prompts; the other twenty had a specified person but lacked a unique supported answer. [Independent role-to-reply counts](role-to-reply.jsonl) · [Join](role-to-reply.jq)

The shifted adverse region produced zero exact literal replies on either panel. The full interval fed through the new interface reproduced the original frozen decisions. All recovered held literal replies moved from a wrong first token directly to exact through EOS. The requested-role oracle identifies a 132-reply held and 224-reply fresh shortfall after supplying the true region. Knowing whom the user requested suffices for the frozen fact search and writer on this fixture. The limited corpus includes known word-for-word entity mentions and extraction-only responses. Later natural dialogue and novel connector forms need separate evidence.

The two count snapshots were graded separately on the 14,287 frozen response targets. The M1 relation reader scored 7,002. Counts in the support columns are optimistic upper bounds. They assume that any explicit suffix successor, or the unigram leader, could be made a winner. Shared fitted weights could achieve a much lower score. Pick here means the raw ID at the successor of the newest nearest earlier pair under identity transport. A committed EOS resets that diagnostic history; an EOS target remains eligible.

| Frozen count deployment | Actually chosen by counts | In count candidate set | With exact-token neighbor | With geometric Pick | With rotated Pick |
|---|---:|---:|---:|---:|---:|
| 1% depth three | 1,553 | 12,145 | 13,019 | 13,136 | 12,160 |
| Full minus held TRAIN, depth four | 2,306 | 13,747 | 14,069 | 14,101 | 13,754 |

The count family already contains most targets. At full dose the geometric successor adds 354 possible targets outside the count candidates; the exact-token neighbor adds 322 and the target-blind rotation adds seven. On the 3,965 rows where an exact token-pair neighbor exists, its returned ID and Pick's ID always coincide. Across all rows those literal successors are correct 3,067 times for the exact neighbor, 3,788 for Pick, and 201 for rotation. The 721 additional correct picks occur among 10,322 positive-distance rows. None of these successor controls emits a correct EOS. A post-capture check sharpens the distinction. Even an oracle choosing the correct answer whenever *either* the count greedy ID or the raw Pick ID is correct can reach only 5,033/14,287 at 1% and 5,666/14,287 at full dose. At full dose that oracle has 2,978 gains and 4,314 harms against the trunk, a net deficit of 1,336 outside a band of 171. A binary switch between those two emitted IDs cannot recover the M1 floor. A trained mass mixer could select a second-ranked count target that neither expert would return alone. Candidate membership accordingly does not qualify the deployed dialogue floor, the sealed P1(b) gate or the equal-byte P2 geometry comparison.

All eight repaired held replies were in one role-swap direction. Both-directions-correct pairs stayed 88 held and 64 fresh out of 128 supported pairs. In the separate 12-row appended-sentence fixture the interval prevented an unsupported copy of “brown” and the base then emitted “Got it, I'll remember that.” This preserved the copy refusal while leaving the free-prose response generic. [Role rank and fixture provenance](region/RESULT.md) · [Writer controls and exact steps](writer/RESULT.md) · [Pick and count expert outcomes](pick/RESULT.md) · [Independent post-capture switch recount](pick-recount.jsonl)

## Checks run at the head

The region CUDA run recreated the prior endpoint output for both seeds on both panels, and the independent cold repeat sealed the same 30-file manifest. The parent separately counted endpoint and no-request transitions by joining all sealed rows. The parent also rejoined the label-blind Pick and count IDs against the frozen row ordinals and independently reproduced counts for the two-greedy-ID switch, exact-token neighbor, rotation and M1 relation decoder. The integer crate ran 706 library tests successfully with one ignored and passed the wasm32 build check, including the public role guard and five existing Pick tests. Both cold writer stages checked 1,082 new trajectories, totaling 2,834 self-fed steps per stage, alongside fully bound cached trajectories. The [Pick result](pick/RESULT.md) records the original, longer canonical M1 spans and the handling of 8,950 committed EOS boundaries in the diagnostic history. The Lean checker ran using the compiler pinned by LexLean-R4, version 4.32.1, commit `f054605a`. These proofs cover abstract ranking given the stated score/weight assumptions. They do not certify the implementation of either language model or establish semantic role understanding.

## Skills used

The native-space-cycle and uor-research-architect reviews bound the typed state, source identity, independent controls, replay and subsequent decisions.

## Still open

The next recall discriminator is an ordered query-role test on unchanged facts. Separate an actually requested entity that the model already found in the excluded slot from a requested entity outranked by a different in-question word. Include “Regarding” with no exclusion, requests that reverse after “except” or “set aside”, no specified person, and an incidental in-question name. Preserve the role-only oracle as a ceiling, both directions with fixed facts, and the no-copy and generated-text controls. Freeze new wording before fitting any change. A contour that simply identifies the beginning of the question has failed its registered bar for further training.

On prediction, first get an interpretable P1(c) ceiling from the separately registered KN mixture fit. If the count branch remains viable, the subsequent S6 comparison must evaluate distributions over successors before taking their final top one, keep pure-expert baselines, and fit using TRAIN only. A greedy-ID switch between count and Pick already fails against the published floor under an oracle. The existential support result supplies room for ranking and supplies no evidence that a trained ranker can recover it. Keep the two separate count snapshots and the original relation reader as controls. Proceed to P2 only after the published P1(a,b) bars. [Approved programme](../../../../../PLAN.md#approved-sequences)

## Plain summary

The writer has sufficient information to answer these literal requests when it knows whom the user means. More accurate question boundaries leave the decisive errors inside the question. For prediction, an expert being able to supply an ID and a deployed model actually choosing that ID are separate tests.
