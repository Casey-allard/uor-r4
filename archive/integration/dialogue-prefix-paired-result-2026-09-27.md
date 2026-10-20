# Complete-prefix dialogue learning: paired development result, September 27, 2026

**Retain the full-prefix endpoint as a narrow conditioning candidate; do not promote it to general chat or automatically extend training.** The fixed comparison completed 1,024 matched updates per arm. Full-prefix improves the selected development response loss in all six sources and gives better answers for Paris, Alex, green and Tokyo than both the retained parent and role-only control. The same outputs still show failed current-turn responsiveness, repetition and instruction failures. Rewrite first-four-token loss regresses against both comparisons.

This result follows the [retained-artifact replay](https://github.com/UOR-Foundation/uor-r4/blob/3c43c6aa21631c5f335f74c287ac19a7000f32a6/docs/integration/dialogue-artifact-replay-2026-09-27.md), [context audit](https://github.com/UOR-Foundation/uor-r4/blob/3c43c6aa21631c5f335f74c287ac19a7000f32a6/docs/integration/dialogue-context-audit-2026-09-27.md), [learning plan](https://github.com/UOR-Foundation/uor-r4/blob/3c43c6aa21631c5f335f74c287ac19a7000f32a6/docs/integration/dialogue-prefix-learning-plan-2026-09-27.md) and [fixed study](https://github.com/UOR-Foundation/uor-r4/blob/0ac3df4679dd0696c4d1f56a231ac96e805ee3af/docs/integration/dialogue-prefix-study-2026-09-27.md). The [owning issue endpoint](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5855661290) records completion. The accompanying [portable evidence](../evidence/dialogue-prefix-paired-result-2026-09-27.json) contains all 58 parent/full/role reply triples, judgments, denominators and artifact bindings, without dense per-decision telemetry.

Both arms copied all 21 arrays of the retained R1d Quaternion/Dot/Full model: width 576, read width 64, context 256, vocabulary 4,096, 5,429,826 parameters. Each began with fresh Adam and local step 0. The actual study executable was source `3c43c6aa21631c5f335f74c287ac19a7000f32a6`, binary SHA256 `c283aec4d733493ba9bfe9d2e5409abd0a8db88fa24641433168e70c4f810e8c`, release with `cpu-accelerate`, batch 16 and two gradient shards. R1d’s historical source label 5109861c and its retrospective tokenizer/mask joins remain historical provenance, not a claim of bitwise reproduction of that training binary.

The intervention was training prefix content, history length and response position together. Full-prefix retained original BOS through the actual Assistant marker. Role-only used exact IDs `[0,35,560,652,714,28,223]`: BOS plus the separately encoded marker. Both learned exactly the same complete response and genuine EOS, with zero loss on prefix and padding and a response-token global mean across shards. No per-episode mean was introduced. Both were evaluated using full real prompts and exact generated history, greedy selection, a 32-token cap, and the existing EOS/cycle stops.

The principal joined all 1,024 retained schedule rows: response IDs, target hashes, supervised counts and cumulative counts match. Each arm saw 16,384 response presentations, 1,165,549 supervised target visits and 4,194,304 tensor positions, spanning 9,858 distinct eligible responses. The cumulative schedule SHA256 is `e5e0b82247c97028e25035e5956e369f11ed8c16668b642d2bfba70f0b342a5b`. The independent reviewer checked endpoint identity and schedule hashes without claiming to repeat that full rowwise audit.

The selected population is narrow: only 14,826 of 129,486 response runs fit the complete 256-ID condition. They contain 1,048,098 of 56,650,286 response tokens. Constraints and Everyday account for 89.48% of eligible responses and 89.19% of actual response presentations; those sources account for 86.25% of supervised target visits. Long and source-imbalanced exclusions limit any generalization claim.

| Training source | Eligible / original runs | Actual response visits per arm | Supervised target visits per arm |
|---|---:|---:|---:|
| Magpie | 404 / 41,340 | 474 | 63,241 |
| Constraints | 6,037 / 17,900 | 6,756 | 704,428 |
| Rewrite | 85 / 17,200 | 93 | 7,892 |
| Summarize | 401 / 12,300 | 428 | 16,339 |
| UltraChat | 669 / 32,121 | 776 | 72,795 |
| Everyday | 7,230 / 8,625 | 7,857 | 300,854 |


The frozen development panel has 161 responses, 13,353 supervised targets including 161 EOS tokens, and 644 first-four targets. All arms use original full prefixes. These are exposed development data despite source split names containing `test`; pooled loss is the token mean of this selected panel, not the original corpus mixture or an equal-source mean.

| Development source | Responses | Target / first-four counts | Response NLL: parent → full → role | First-four NLL: parent → full → role |
|---|---:|---:|---|---|
| Magpie | 23 | 3,266 / 92 | 3.317055 → 3.260219 → 3.280864 | 3.485396 → 3.251743 → 3.422242 |
| Constraints | 32 | 3,811 / 128 | 3.206600 → 2.709090 → 2.770132 | 2.552591 → 1.872382 → 2.370155 |
| Rewrite | 10 | 769 / 40 | 1.962991 → 1.939291 → 2.165941 | 0.583383 → 0.927748 → 0.713295 |
| Summarize | 32 | 1,325 / 128 | 2.052222 → 1.623803 → 1.976305 | 1.960448 → 0.940989 → 1.938515 |
| UltraChat | 32 | 3,233 / 128 | 3.125430 → 3.023602 → 3.089518 | 2.902075 → 2.851550 → 3.056944 |
| Everyday | 32 | 949 / 128 | 3.126874 → 2.791712 → 2.842334 | 1.481816 → 1.027871 → 1.257861 |
| Pooled |161 |13,353 /644 |3.022131 →2.773887 →2.863946 |2.302482 →1.852403 →2.247178 |


Full-prefix response NLL improves against both comparisons in every source. First-four NLL improves in five sources, but Rewrite worsens from 0.583383 to 0.927748, versus role-only 0.713295. That retained-source tradeoff prevents a blanket non-regression conclusion. No significance or fresh-held-out claim is made.

All 38 fixed requests were read as actual generated text: five greetings, eight factual requests, seven sentence instructions, ten three-turn memory trajectories and eight ambiguity/context requests, giving 58 replies per arm. Later turns use each arm’s own generated history, so these are not 58 independent trials or identical later prefixes. Correctness is judged against user facts even when an earlier assistant reply was wrong.

- **Paris, factual request:** Full-prefix says, “The capital of France is Paris. It is a great way to France.” The parent repeats France without Paris; role-only says, “To find the capital of France is the capital of France.” Role-only nevertheless emits the correct Paris sentence off task for the sun-writing and yesterday requests. The gain supports appropriate conditioning, not newly acquired world knowledge.
- **Alex, final recall:** Full-prefix says, “The name is Alex is Alex.” Parent and role-only omit Alex at final recall. The intermediate request about apples remains unanswered.
- **Favorite green, final recall:** Full-prefix says, “The favorite color is green.” before repeated planning filler. Role-only omits green and the parent mixes blue and green. Full-prefix also repeats green on the intervening blue-related turn rather than responding to it.
- **Tokyo, final recall:** Full-prefix says, “Yes, this sister lives in the Tokyo, including the Tokyo, and the Tokyo is a great way”. Both comparisons fail the final city answer. The correct payload is recoverable, but principal and independent reviewers retain different overall labels because the tail is malformed and unfinished.
- **Shared gains:** Both endpoints greet “Hi there!” with “Hello! How can I help you today?” Both recover pizza on the final food turn; full-prefix gives the favorite relation more clearly. Momo and blue-car payloads already occur in parent outputs and are not new acquisition.
- **Control-favorable result:** Role-only gives the same clean greeting to “Good morning!” where full-prefix introduces an irrelevant date requirement. More role-only EOS endings—46/58 versus full-prefix 28/58 and parent 11/58—often close irrelevant boilerplate rather than a fulfilled request.

No arm gives a coherently responsive three-turn conversation: all ten middle memory turns remain unresponsive. Full-prefix still misses seven of eight factual requests; role-only misses all eight on task. Neither endpoint usefully fulfills any of the seven sentence requests, and all eight ambiguity/context requests remain non-answers. A correct final initial-fact repetition does not establish ongoing dialogue, instruction following or reasoning.

The five earlier full-prefix label disagreements are preserved, not voted away. In each case the principal labels the overall reply **useful**, while the independent reviewer labels it **partial**:

| Request and turn | Shared observation and reason for retaining disagreement |
|---|---|
| `dev-mem-01`,1 | Alex is present, but repeated awkward acknowledgement is not an independent recall test. |
| `dev-mem-05`,1 | Tokyo is present; the P.S. tail ends in “includ”. Parent already retained Tokyo here. |
| `dev-mem-05`,3 | Tokyo answers final recall, but the repeated and unfinished tail lowers overall reply quality. |
| `dev-mem-07`,1 | July is associated with the birthday, followed by repeated “great way to July”. |
| `dev-mem-08`,3 | Blue answers the car color, but the whole reply is a cycle and the payload predates this study. |


There is one additional role-only difference: for `dev-core-greet-05`, “Hello! How can I help you today?” is **non-answer** for the principal and **partial** for the independent reviewer. Both agree it restarts a greeting rather than acknowledging thanks. The evidence packet preserves the exact texts and both original reasons. These differences do not change the narrow candidate decision or create an aggregate acceptance score.

Role-only was interrupted at step 558 when the internal storage guard fired. The resumed run loaded the sealed step 512 model and Adam state using the same executable, then completed 512 new updates and 577,427 new supervised visits to the declared step1,024 endpoint. The discarded 46 updates, 50,798 supervised visits and 188,416 tensor positions remain charged. They are excluded from retained endpoint exposure. Recovery’s initial development result belongs to step 512, not to the parent; no bitwise identity to an uninterrupted run is claimed. Both final saved/reloaded models report zero response-NLL delta on their declared reload check.

| Process cost component | Wall seconds | Maximum child RSS, bytes |
|---|---:|---:|
| Full-prefix complete | 4971.440720 | 5,535,236,096 |
| Original role-only interrupted | 2742.938996 | 4,894,605,312 |
| Role-only recovery complete | 2412.994606 | 5,206,999,040 |
| Total fit-process wall | 10127.374322 | Peak 5,535,236,096 |


These process times include learning, scheduled development, checkpointing and endpoint outputs. Saved update timers total 4879.851386s for full-prefix, 4823.201358s for retained role-only and 220.038760s for discarded role work; they are nested in process wall and must not be added again. The frozen study tests/build took 56.359656s separately. Earlier implementation/witnesses plus preparation, review and delivery remain in the principal cumulative ledger; this table is not whole-programme elapsed cost and resets no allowance.

Keep both endpoints, the stopped original role run and their negative outputs. The separately completed [native integer observation](native-dialogue576-observation-2026-09-27.md) localizes its largest numerical change to parameter discretization. That historical-parent result does not qualify this new full-prefix child. Reuse the retained numerical-repair evidence and the independent packed-workload comparison before selecting another learning or discretization change. This packet does not authorize an automatic longer fit, new seed, decoder sweep, new panel or mechanism search. It establishes neither general prose, coherent dialogue, coding/reasoning, fresh-held-out generalization, geometric advantage, integer language preservation nor complete-path speed/energy savings.
