<!-- freshness: sources=archive/catalog.jsonl checked=d5eb4a55656726b04ea79b06968f91905933c925 -->
# How the project got here

UOR-R4 began in June 2026 as an attempt to run a language model without a transformer or heavy arithmetic, using the geometry of primes, rotations and symmetry groups. Over four months it tried five broad approaches. Each was replaced when it hit a wall, and each left lessons. The 32 steps below are the audit's "eras", read from the GitHub archive (items [#1](https://github.com/UOR-Foundation/uor-r4/issues/1) to [#2495](https://github.com/UOR-Foundation/uor-r4/issues/2495)). The work has been careful and open about its failures, and it has not yet produced a model that holds a conversation.

![A timeline from 10 June to 19 October 2026. A top band shows five phases, A to E, divided into 32 numbered eras. An area chart below shows catalog records per day, with nothing before 21 July and a peak of 74 on 13 October.](../figures/research/timeline.svg)

*Five phases and 32 eras on one time axis, with the number of catalog records per day underneath (1,051 of the catalog's 1,173 records carry a date). Era spans come from the archive audit; the daily counts come from `archive/catalog.jsonl`. The figure draws Phase E to 19 October; the era snapshot below ends on 15 October.*

Terms follow the [glossary](../start/glossary.md). `#N` is an issue or pull request number in `UOR-Foundation/uor-r4`. Every row below is a **measurement** of what a recorded item said, at that item's scope, unless marked otherwise. Eras overlap on their edges, and some items were read by title only (`docs/labs/repo-renormalization-2026-10-19/audit/eras/eras.json`, field `notes`).

## Phase A: compile a teacher into tables (10 June to 26 August)

*In plain words: watch a real language model, the "teacher", and compile what it does into lookup tables and a graph that a CPU can run with no multiplications.*

| Era | Dates | What was tried, and what happened |
|---|---|---|
| 01 Prime router | to 21 Jul | A router over sparse 512-dimension prime vectors ([#1](https://github.com/UOR-Foundation/uor-r4/issues/1)) and hypervector algebra ([#9](https://github.com/UOR-Foundation/uor-r4/issues/9)), folded into the graph-compiler plan ([#10](https://github.com/UOR-Foundation/uor-r4/issues/10)) |
| 02 R4G1 graph compiler | 21 to 26 Jul | The graph file format, certificates, and the rule that only serving must be multiplication-free ([#11](https://github.com/UOR-Foundation/uor-r4/issues/11), [#157](https://github.com/UOR-Foundation/uor-r4/issues/157)). Mostly structure |
| 03 Quality chain | 23 to 24 Jul | A 16-fix plan began at 0.3% against a 31.7% baseline ([#65](https://github.com/UOR-Foundation/uor-r4/issues/65)). A metric reading zero hid a real loss ([#102](https://github.com/UOR-Foundation/uor-r4/issues/102)). The Furey algebra code was never wired into inference ([#103](https://github.com/UOR-Foundation/uor-r4/issues/103), [#249](https://github.com/UOR-Foundation/uor-r4/issues/249)) |
| 04 Token-index pivot | 26 Jul | Bad scores came from tokenizer indexing, not the runtime ([#201](https://github.com/UOR-Foundation/uor-r4/issues/201)) |
| 05 Integer store | 27 Jul to 3 Aug | Shift-and-add assignment gave smaller, more accurate stores with zero multiplies ([#243](https://github.com/UOR-Foundation/uor-r4/issues/243)). Phase-clock addressing was negative ([#392](https://github.com/UOR-Foundation/uor-r4/issues/392)) |
| 06 Geometry as substrate | 3 to 6 Aug | E8 codebooks beat sign codes at equal bits ([#395](https://github.com/UOR-Foundation/uor-r4/issues/395)). A zeta-grid headline was a storage artifact ([#440](https://github.com/UOR-Foundation/uor-r4/issues/440)) |
| 07 Broad text | 5 to 8 Aug | The 26 to 27% ceiling on broad text came from the teacher, not the store ([#437](https://github.com/UOR-Foundation/uor-r4/issues/437)) |
| 08 Bigger teacher | 8 to 12 Aug | SmolLM2-360M became the pinned baseline ([#509](https://github.com/UOR-Foundation/uor-r4/issues/509), [#516](https://github.com/UOR-Foundation/uor-r4/issues/516)). Ruled-out mechanisms were kept behind gates ([#515](https://github.com/UOR-Foundation/uor-r4/issues/515)) |
| 09 Serving | 10 to 19 Aug | A corpus-ordering bug was a major cause of garbage output ([#755](https://github.com/UOR-Foundation/uor-r4/issues/755)) |
| 10 Plan S0 to S7 | 19 to 26 Aug | Truth baseline 29.52% against 28.12% ([#821](https://github.com/UOR-Foundation/uor-r4/issues/821)). Abstention, coherent free-running text and planning all ended as LIMIT. Fixed W(3,3) geometry showed no advantage ([#929](https://github.com/UOR-Foundation/uor-r4/issues/929)) |

**Why it ended.** Gains measured with the teacher's text supplied never became coherent text the model wrote itself. The reset in [#948](https://github.com/UOR-Foundation/uor-r4/issues/948) made a learned geometric decoder the objective.

## Phase B: a learned geometric decoder (25 August to 8 September)

*In plain words: learn a model directly, after first checking that ordinary attention can live inside the geometry.*

| Era | Dates | What was tried, and what happened |
|---|---|---|
| 11 Decoder reset | 25 to 29 Aug | A one-layer R4 mixer gained 4.44%, short of its 5% bar ([#951](https://github.com/UOR-Foundation/uor-r4/issues/951)). Route-native attention worked on exact identities ([#969](https://github.com/UOR-Foundation/uor-r4/issues/969)) |
| 12 The [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) programme | 28 Aug to 3 Sep | Softmax attention reproduced exactly in R4/Spin coordinates ([#1002](https://github.com/UOR-Foundation/uor-r4/issues/1002)). A 7.15M-parameter model could continue text but not chat ([#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017)). A copied cell scored 99.17% ([#1050](https://github.com/UOR-Foundation/uor-r4/issues/1050)) |
| 13 Build first | 3 to 5 Sep | The owner replaced a rule that forbade running code ([#1111](https://github.com/UOR-Foundation/uor-r4/issues/1111)) and restored a single Rust model path ([#1125](https://github.com/UOR-Foundation/uor-r4/issues/1125)) |
| 14 Native model v0 | 5 to 8 Sep | Typed Stop/Copy/Add selection with a 663-answer regression set ([#1140](https://github.com/UOR-Foundation/uor-r4/issues/1140)). An honesty pass withdrew fabricated scorecards ([#1198](https://github.com/UOR-Foundation/uor-r4/issues/1198)). The project got its name ([#1174](https://github.com/UOR-Foundation/uor-r4/issues/1174)) |

**Why it ended.** No single mechanism failed. A lab closed the early-September ladder on 29 September in favour of one plan of record ([#1507](https://github.com/UOR-Foundation/uor-r4/issues/1507)).

## Phase C: native learner and exact memory (8 to 25 September)

*In plain words: teach a home-grown Rust model to remember exact facts and versions, then find out why it still could not write prose.*

| Era | Dates | What was tried, and what happened |
|---|---|---|
| 15 Version-intent cycles | 8 to 12 Sep | Exact versioned memory, with rules for abstention and sealed reports ([#1222](https://github.com/UOR-Foundation/uor-r4/issues/1222), [#1224](https://github.com/UOR-Foundation/uor-r4/issues/1224)). The shared-core gate failed ([#1229](https://github.com/UOR-Foundation/uor-r4/issues/1229)) |
| 16 Addressed attention | 12 to 17 Sep | Emission was the first bottleneck ([#1230](https://github.com/UOR-Foundation/uor-r4/issues/1230)). Typed attention exported to exact Boolean tables ([#1254](https://github.com/UOR-Foundation/uor-r4/issues/1254)). Passes came on constructed panels only ([#1264](https://github.com/UOR-Foundation/uor-r4/issues/1264)) |
| 17 Falsification sweep | 19 to 20 Sep | Attention in the vector-symbolic layer (VSA) was inert, and the trained model and shipped artifact differed by 0.57 bits per byte (BPB) ([#1284](https://github.com/UOR-Foundation/uor-r4/issues/1284)). The readout took 99.97% of per-token compute ([#1287](https://github.com/UOR-Foundation/uor-r4/issues/1287)) |
| 18 Relational readers | 20 to 23 Sep | A reader-policy gain did not reproduce ([#1330](https://github.com/UOR-Foundation/uor-r4/issues/1330)), and H4 tied a categorical control ([#1344](https://github.com/UOR-Foundation/uor-r4/issues/1344)) |
| 19 Prose deficit | 23 to 25 Sep | Doubling training changed nothing ([#1366](https://github.com/UOR-Foundation/uor-r4/issues/1366)). The recurrent learner scored 2.110 negative log-likelihood (NLL, lower is better) against 2.085 for the ordinary model and 1.574 for the transformer ([#1389](https://github.com/UOR-Foundation/uor-r4/issues/1389)). Integer serving shipped ([#1397](https://github.com/UOR-Foundation/uor-r4/issues/1397)), prose stayed at 0 of 5 ([#1398](https://github.com/UOR-Foundation/uor-r4/issues/1398)) |

**Why it ended.** The offline learning ladder reached integer serving but not prose, so focus moved to a chat model and parallel labs.

## Phase D: the geometric stack, labs, chat and memory (26 September to 11 October)

*In plain words: train a deeper "geometric stack" on GPUs, serve it in integers, and push it toward chat and memory with several AI labs working in parallel.*

| Era | Dates | What was tried, and what happened |
|---|---|---|
| 20 Serving contract | 26 to 29 Sep | The stack reached parity with a transformer control ([#1437](https://github.com/UOR-Foundation/uor-r4/issues/1437)). Fixed E8 and H4 codebooks lost to k-means ([#1456](https://github.com/UOR-Foundation/uor-r4/issues/1456)). Integer serving, on a dense model with every multiply replaced by a table read, cost 4.3 times more energy than its float parent ([#1449](https://github.com/UOR-Foundation/uor-r4/issues/1449)) |
| 21 Durable labs | 29 Sep to 2 Oct | The learned pointer copied the most recent value ([#1558](https://github.com/UOR-Foundation/uor-r4/issues/1558)). An exact log with a prime sieve did the retrieval ([#1559](https://github.com/UOR-Foundation/uor-r4/issues/1559), [#1607](https://github.com/UOR-Foundation/uor-r4/issues/1607)) |
| 22 Scale ladder | 2 to 5 Oct | CUDA training and a 1.39B-token chat corpus ([#1660](https://github.com/UOR-Foundation/uor-r4/issues/1660)). Retrieval caused 2.2% of failing rows ([#1708](https://github.com/UOR-Foundation/uor-r4/issues/1708)). A constant reply beat every model on tier C ([#1733](https://github.com/UOR-Foundation/uor-r4/issues/1733)) |
| 23 Contest | 5 to 6 Oct | The weak part was representation, not the output head ([#1780](https://github.com/UOR-Foundation/uor-r4/issues/1780)). The geometric read's measured gain was seed reliability ([#1798](https://github.com/UOR-Foundation/uor-r4/issues/1798)) |
| 24 End-to-end serving | 6 to 9 Oct | A chat stack served multiplier-free and bit-exact (0.93277 BPB at 64 windows; the 0.877 float figure used 512, so the pair is not comparable) ([#1893](https://github.com/UOR-Foundation/uor-r4/issues/1893), [#1899](https://github.com/UOR-Foundation/uor-r4/issues/1899)). Data mattered more than size ([#1913](https://github.com/UOR-Foundation/uor-r4/issues/1913)) |
| 25 Constructor line | 6 to 10 Oct | M2 stayed at 8 of 512 complete replies ([#1882](https://github.com/UOR-Foundation/uor-r4/issues/1882)) until the rule on three negatives stopped the line |
| 26 Instruction following | 9 Oct | Compliance was 0 of 6 ([#1918](https://github.com/UOR-Foundation/uor-r4/issues/1918)) until each demand was paired with its short answer ([#1958](https://github.com/UOR-Foundation/uor-r4/issues/1958)) |
| 27 Governance reset | 9 to 10 Oct | Main became the single source of truth ([#2015](https://github.com/UOR-Foundation/uor-r4/issues/2015)) and the rule that a negative closes a configuration only was written ([#2165](https://github.com/UOR-Foundation/uor-r4/issues/2165)). Geometric operations were 0.05% of the stack's arithmetic ([#2186](https://github.com/UOR-Foundation/uor-r4/issues/2186)) |
| 28 Pointer and binding | 9 to 11 Oct | The pointer used softmax and could not be served ([#2072](https://github.com/UOR-Foundation/uor-r4/issues/2072)). Read binding was KEEP ([#2190](https://github.com/UOR-Foundation/uor-r4/issues/2190)) |
| 29 M2 replies | 10 Oct | Replies rose from 8 to 437 of 512 on development rows ([#2162](https://github.com/UOR-Foundation/uor-r4/issues/2162)), then scored 0 of 128 on a fresh draw ([#2164](https://github.com/UOR-Foundation/uor-r4/issues/2164)) |

**Why it ended.** The fresh failure showed the development panel had been fit, and later bans on softmax and matrix multiplication made this path non-compliant ([#2164](https://github.com/UOR-Foundation/uor-r4/issues/2164), [#2251](https://github.com/UOR-Foundation/uor-r4/issues/2251), [#2263](https://github.com/UOR-Foundation/uor-r4/issues/2263)).

## Phase E: lawful operators and the discrete substrate (10 to 15 October)

*In plain words: replace softmax and matrix multiplication with true geometric operations, then learn on exact integers.*

| Era | Dates | What was tried, and what happened |
|---|---|---|
| 30 Mechanism rungs | 10 to 13 Oct | Softmax-free rank reads were REJECT ([#2160](https://github.com/UOR-Foundation/uor-r4/issues/2160)). Quaternion matrices and icosian weights were KEEP ([#2193](https://github.com/UOR-Foundation/uor-r4/issues/2193), [#2216](https://github.com/UOR-Foundation/uor-r4/issues/2216)). The starling read lost entries to eviction at long context ([#2239](https://github.com/UOR-Foundation/uor-r4/issues/2239)) |
| 31 Operator phase | 12 to 14 Oct | Softmax and matmul were banned ([#2251](https://github.com/UOR-Foundation/uor-r4/issues/2251), [#2263](https://github.com/UOR-Foundation/uor-r4/issues/2263)). A rank-margin loss, the Hamilton gate and SpiralCore/FBS projection were added. Controls passed and nothing was trained |
| 32 Discrete substrate | 13 to 15 Oct | Integer learning passed its CUDA controls ([#2379](https://github.com/UOR-Foundation/uor-r4/issues/2379)). In the pilot, top-8 hits rose from 173 to 1,169 while aggregate loss also rose ([#2449](https://github.com/UOR-Foundation/uor-r4/issues/2449), [#2454](https://github.com/UOR-Foundation/uor-r4/issues/2454)). The prime-zeta construction qualified ([#2473](https://github.com/UOR-Foundation/uor-r4/issues/2473)) |

**After the snapshot (15 to 19 October).** The owner re-allowed offline float training on 18 October, with an export-gap check. The K16 runs then completed with no KEEP: the best top-1 was 65 and 463 of 14,287 rows, below a unigram table at 1,226 ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)). The A8 address-cell dose ended at its registered first diagnostic stop ([A8 dose](../labs/a8-dose-2026-10-19/README.md)). Training was held then; D67 later replaced the blanket owner hold with each lab's own pre-run review, leaving D41's donor pass as the named exception.

## Phase F: the in-space trunk and the pivot (27 to 29 October)

*In plain words: train in the model's own space, find that it has nowhere to store answers, add exact memory, and make serving fast.*

- **27 Oct.** A trunk trained in its own space reached about 60% next word on held-out text, with reply openings at 220 of 1,226. The Hamming read recalled 0.98 to 0.99 in its probe. Whole-table nearest was chosen over the native writer ([claude-150](../loop/tasks/one-decoder/claude-150/REPORT.md)).
- **28 Oct, the pivot.** The diagnosis ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md)) found the storage missing, and the owner chose to pivot to route plus exact memory. Memory named 598.7 and 598.3 of 1,226 openings at two seeds, for claude-161's opening table only ([claude-161](../loop/tasks/route-plus-memory/claude-161/REPORT.md)). D80 and D81 allowed integer multiply and divide in serving, and a served token went from 270 to 4,829 tok/s, snapped ([claude-177](../loop/tasks/integer-mul-div-serving/claude-177/REPORT.md)).
- **28 to 29 Oct, the export.** A deterministic backward ([claude-179](../loop/tasks/d11-exporter-float-trunk/claude-179/REPORT.md)) and the unsnapped contract `lut-stack/5` put the export gap inside the bar at DEV ([claude-182](../loop/tasks/d11-exporter-float-trunk/claude-182/REPORT.md)).
- **29 Oct, directions.** One working model end to end, and this documentation refresh. Four labs work in the loop: claude, antigravity, deepseek and kimi. The project is still pre-alpha: 0 of 8 milestones are complete.

## Phase G: M1 closes and generation becomes the question (30–31 October)

*In plain words: the saved discrete model beats both count tables, but complete replies on new phrasing still fail.*

- **The saved model closes M1.** antigravity-110's two integer artifacts pass all six bars at both seeds: DEV 7,002 / 7,001 against T2 6,300; Fresh V2 7,215 / 7,228 against T2 6,582; and p-f 7,850 / 7,872 against T2 7,375, beyond paired band 81. Exact 14,287-row DEV export agreement and B4 hold, and Apple M1 decode is 10,876 / 10,566 tok/s. The older close grade covers (a)–(e); claude-211 supplies bar (f), and #2029 closes ([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).
- **The limit stays visible.** p-f has no prompt value-masked templates in TRAIN at K = 1,024, but 62.5% of its reference replies are verbatim TRAIN replies. M1 reads new prompt phrasings into known reply kinds, not new reply kinds or useful conversation. The project is pre-alpha: **1 of 8 milestones is complete**.
- **Generation exposes the next gap, and an offline key probe passes.** claude-217's cycle step ends every registered loop, but every content decision is underpowered and greedy remains the decode. On p-s, teacher-forced reply openings are 17.4%, 13.7% and 18.6% at three seeds, against about 80% on later tokens. Exact turn memory keys on exact first token ids and misses reworded prompts ([claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md)). Claude-218's offline teacher-forced key probe then found sign and point keys that improved reply openings on p-u, selecting the cheaper sign key before a free-generation test ([claude-218](../loop/tasks/one-model-end-to-end/claude-218/REPORT.md)).
- **A point-key opening gives a partial free-generation gain, then enters the model.** On fresh p-v, claude-219's point key improves templates by +13/+8/+12 and exact replies by +9/+8/+12 at three seeds, reaching only 12–18% of the forced-opening lift; the sign key is inconclusive at this panel size. Claude-220 then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. Claude-222 has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade; [claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun; useful chat remains unproven.
- **Housekeeping follows the result.** Kimi's documentation refresh reconciles the front door, book, living READMEs, figures, trackers and Studio text with M1 and the current research, while preserving dated evidence and other labs' active work.

## Phase H: replies in conversation and one native state (1 to 14 November)

*In plain words: the FBS line learns human text and recalls told values, but its facts come from an exact table, so the route moves to one native integer state.*

- **Human text.** On a held-out panel of 1,460,697 rows, a batch of 64 with twice the training text gets 331,698 and 342,250 next tokens right at two seeds, ahead of T2's 256,660 (claude-301, 6 Nov).
- **Recall in controlled conversations (from 10 Nov).** claude-358's three models state a told value on 314, 320 and 325 of 578 questions (13 Nov). Additions follow: an E8 address memory (not called), pair experts counted from TRAIN (adds, but a reply served that way is a stored TRAIN reply) and trained pair rows (adds, not served).
- **One native state (D85, 13 Nov).** The owner made one persistent integer geometric state, shared by training and serving with bit-identical replay, the default route. A first Q31 vector state trains on CUDA in a 64-step contract test (codex-5).
- **What the vectors carry (14 Nov).** V0 serves claude-358's models with and without the exact turn memory: 377 to 441 against 171 to 211 exact replies of 2,218, with almost every fact coming from the memory's table (claude-376). Carrying the state across training windows (V1) and vector-native reads (V2(a)) are in progress.

## Where the effort went

![Bars showing, for each of 14 weeks from 20 July to 19 October, the share of records that were model results. For the eight weeks from 31 August the share is 60%, 79%, 38%, 43%, 80%, 78%, 11% and 13% in order, with a sharp drop in the week of 12 October.](../figures/research/results-share-by-week.svg)

*Share of each week's records that were model results (positive, negative, inconclusive or a diagnostic stop). Source: `archive/catalog.jsonl`, counted by ISO week (for example week 36 is 59 of 99 records, week 42 is 34 of 309); see also the corrected table in `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md` section 3. The figure shows 14 weeks, from week 30 (20 July).*

The chart is a **measurement** of the audit's record counts. Results fell from 78% of the week's records in the week of 5 October to 11% in the week of 12 October.

| Window | Model-line results | Other model-line records | Governance and operations | Results share |
|---|---:|---:|---:|---:|
| 9 to 11 Oct | 83 | 21 | 4 | 77% |
| 12 to 18 Oct | 34 | 116 | 157 | 11% |
| 19 Oct | 4 | 21 | 7 | 13% |

Part of the fall was ordered, not chosen. On 12 and 13 October the owner's bans on softmax and matmul, the GPU-only rule and the Rust-first rule forced a rebuild of the substrate and many tool ports. So the 89% of non-result records that week is an upper bound on discretionary process. Many of the 34 "results" that week are kernel parity or qualification passes, not learning. Five training efforts have run on the discrete path since 13 October (the A2 pilot, Dose 1, rows Dose 2, K16 and A8), and none produced a KEEP. Source: `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md` section 3, from the audit's `records/*.jsonl`.

## Go deeper

- [What the project knows](./findings.md) and [what is still open](./open-questions.md).
- [Research rules now](./README.md) and the [archive catalog](../../archive/CATALOG.md).
- The [mechanism board](../mechanisms/README.md).
