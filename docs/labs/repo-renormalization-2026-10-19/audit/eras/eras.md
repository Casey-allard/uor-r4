# How UOR-R4 got here: June to mid-October 2026

UOR-R4 set out to build a language model that does not need a transformer or heavy matrix arithmetic, so it can run fast on an ordinary laptop. Its tools are geometry: prime-number addresses, zeta phases, quaternion rotations, and the E8 and H4 symmetry groups. Between June and mid-October it tried five broad approaches. Each was replaced when it hit a wall, and each left lessons. This page summarises 32 eras from the GitHub archive (items #1 to #2495). `#N` is an issue or pull request number; full detail is in `eras.json`.

## Phase A: compile a teacher into tables (era-01 to era-10, 10 June to 26 August)

**In plain words:** watch a real language model (the "teacher") and compile what it does into lookup tables and a graph that a CPU can run with no multiplications.

- **era-01, prime router (to 21 Jul).** The first code was a router over sparse 512-dimension prime vectors (#1), plus hypervector algebra (#9). It was folded into the graph-compiler plan (#10).
- **era-02, R4G1 graph compiler (21 to 26 Jul).** Defined the graph file format, certificates, and the rule that only *serving* must be multiplication-free and allocation-free (#11, #157). This was mostly structure.
- **era-03, quality chain (23 to 24 Jul).** A 16-fix plan started from 0.3% accuracy against a 31.7% baseline (#65). A metric reading zero hid a real loss (#102). The Furey algebra modules were never wired into inference (#103, #249).
- **era-04, token-index pivot (26 Jul).** Bad scores came from tokenizer indexing, not the runtime (#201). The first CLI and browser interface appeared (#212, #222).
- **era-05, integer store (27 Jul to 3 Aug).** Shift-and-add assignment gave smaller, more accurate stores with zero multiplies (#243). At a 500k-token corpus the integer runtime beat the float ceiling measured at 150k (#327). Phase-clock addressing was negative (#392). An outside model exposed losses inside the pipeline: 3.84 against 11.17 bits per token (#300).
- **era-06, geometry as substrate (3 to 6 Aug).** E8 codebooks beat sign codes at equal bits (#395). Hopf projection gave no retrieval value (#430), and a zeta-grid headline was a storage artifact (#440). Geometry helped only on repetitive data (#393).
- **era-07, broad text (5 to 8 Aug).** The 26-27% ceiling on broad text came from the teacher, not the store (#437).
- **era-08, bigger teacher (8 to 12 Aug).** SmolLM2-360M became the pinned baseline (#509, #516). Ruled-out mechanisms were kept behind gates rather than deleted (#515).
- **era-09, recompilation and serving (10 to 19 Aug).** Added an OpenAI-compatible API (#654) and a GPT-2 compile (#657). A corpus-ordering bug was a major cause of the garbage output (#755). The loops that remained came from argmax, not code collisions (#784).
- **era-10, S0 to S7 plan (19 to 26 Aug).** Set a truth baseline of 29.52% against 28.12% (#821) and activated a skip-mix lane (#908). Abstention, coherent free-running text and planning all ended as LIMIT. W(3,3) geometry showed no advantage (#929).

**Why it ended:** gains measured while the teacher's text was supplied never turned into coherent text the model wrote on its own. The reset in #948 made a learned geometric decoder the objective.

## Phase B: a learned geometric decoder (era-11 to era-14, 25 August to 8 September)

**In plain words:** learn a model directly, after first checking that ordinary attention can live inside the geometry.

- **era-11, decoder reset.** A one-layer R4 mixer in SmolLM2 gained 4.44%, short of its 5% bar (#951). Route-native attention worked on exact identities (#969), but semantic transfer could not be measured (#983, #986).
- **era-12, the #973 programme.** Softmax attention reproduced exactly in R4/Spin coordinates (#1002), and trace distillation added nothing (#1011). A 7.15M-parameter model became a continuation generator, not a chat model (#1017, #1041). A copied Zoology cell scored 99.17%, which showed the binding failures came from the training recipe (#1050).
- **era-13, build-first.** The owner replaced a rule that forbade running code (#1111) and restored a single Rust model path (#1125).
- **era-14, native model v0.** Built typed Stop/Copy/Add selection with a 663-answer regression set (#1140, #1200). An honesty pass withdrew fabricated scorecards (#1198). The project was named "UOR-R4 Geometric Language Model" (#1174).

## Phase C: native learner and exact memory (era-15 to era-19, 8 to 25 September)

**In plain words:** teach a home-grown Rust model to remember exact facts and versions, then find out why it still could not write prose.

- **era-15, version-intent cycles.** Built exact versioned memory and the rules for abstention and sealed reports (#1222, #1224). The shared-core gate failed (#1229).
- **era-16, addressed attention.** Emission was the first bottleneck (#1230). Typed attention exported to exact Boolean tables (#1254). Passes came on constructed panels only (#1264).
- **era-17, falsification sweep.** VSA attention was inert, and the trained model and the shipped artifact differed by 0.57 BPB (#1284). The readout took 99.97% of per-token compute (#1287). Count tables beat added history state (#1304).
- **era-18, relational readers.** Near misses: a reader-policy gain did not reproduce (#1330), and H4 tied a categorical control (#1344). Several claims were later corrected (#1320 to #1331).
- **era-19, prose deficit and D8.** Doubling training changed nothing (#1366), and no geometric mechanism had beaten a matched ordinary control (#1371). The goals split into S (multiplier-free serving) and R (geometric advantage) (#1377). The recurrent learner scored 2.110 NLL against 2.085 for the ordinary model and 1.574 for the transformer (#1389). Integer serving shipped (#1397), but prose stayed at 0 of 5 (#1398).

## Phase D: the geometric stack, labs, chat and memory (era-20 to era-29, 26 September to 11 October)

**In plain words:** train a deeper "geometric stack" on GPUs, serve it in integers, and push it towards chat and memory with several AI labs in parallel.

- **era-20, D11 contract.** The stack reached parity with a transformer control (#1437). D11 made multiplier-free serving the single contract (#1443). Fixed E8/H4 codebooks lost to k-means (#1456), and quaternion transport was kept (#1460). Integer serving used 4.3x more energy than its float parent (#1449). Numbers typed into tests were retracted (#1459).
- **era-21, D18 and D19.** The learned pointer copied the most recent value, not the queried one (#1558). An exact log with a prime sieve did the retrieval (#1559, #1607).
- **era-22, scale ladder.** Added CUDA training and a 1.39B-token chat corpus (#1660). Retrieval caused only 2.2% of failing rows (#1708), and a constant reply beat every model on tier C (#1733).
- **era-23, D20 contest.** The weak part was representation, not the output head (#1780, #1785). The geometric read's measured contribution was seed reliability (#1798).
- **era-24, end-to-end serving.** A chat stack was served multiplier-free at 0.93277 BPB, bit-exact against its reference (#1893, #1899). Training data mattered more than model size (#1913). A LUT-4 gain was retracted as a constant shift (#1881).
- **era-25, Codex constructor.** M2 stayed at 8 of 512 complete replies (#1882) until D21 stopped the line.
- **era-26, instruction following.** Compliance was 0 of 6 (#1918) until each demand was paired with a matching short answer (#1958).
- **era-27, governance reset.** Main became the single source of truth (#2015), alongside the tracker #2028, D21 (#2135), D22 (#2165) and mechanism briefs (#2167). Geometric operations were 0.05% of the stack's arithmetic (#2186).
- **era-28, pointer and read binding.** A gate floor showed the answer was already inside the read (#2008). The pointer used softmax and could not be served (#2072). Read binding was KEEP (#2190).
- **era-29, M2 replies.** Rose from 8 to 437 of 512 on development rows (#2162), then scored 0 of 128 on a fresh draw (#2164).

## Phase E: lawful operators and the discrete substrate (era-30 to era-32, 10 to 15 October)

**In plain words:** replace softmax and matrix multiplication with true geometric operations, then learn on exact integers.

- **era-30, mechanism rungs.** Softmax-free rank reads were REJECT (#2160). Quaternion matrices (#2193) and icosian weights (#2216) were KEEP. E8 memory met its bar but was not credited (#2205). The starling read was KEEP on synthetic recall (#2233) but lost entries to eviction at long context (#2239).
- **era-31, operator phase.** Banned softmax (D23, #2251) and matmul (D25, #2263), and added a rank-margin loss (#2255), the Hamilton gate (#2260) and SpiralCore/FBS projection (#2279). Controls passed and nothing was trained. D32 (#2367) replaced the phase.
- **era-32, D32 discrete substrate.** Integer learning proposals passed their CUDA controls (#2379). In the pilot, top-8 hits rose from 173 to 1169 while aggregate loss also rose; cell coverage and collisions limited it (#2449, #2454). The prime-zeta construction qualified (#2473), and an object-store prototype was built (#2495). This phase was still open at the snapshot.

## Knowledge to keep

1. Check the instrument before trusting a result. A zero-delta metric (#102), pipeline loss (#300), typed-in numbers (#1459), a constant LUT term (#1881) and vacuous controls (#2112) all misled.
2. Measure the shipped form: the trained model and the shipped artifact once differed by 0.57 BPB (#1284).
3. A pass that was teacher-forced or on an authored panel is not coherent generation (#948, #1264, #1398).
4. Development is not fresh: 437 of 512 became 0 of 128 (#2164).
5. Data and teacher breadth beat model size and clever stores (#437, #1913).
6. Fixed geometry usually tied or lost to matched ordinary controls (#929, #1371, #1456). Exceptions kept: quaternion transport (#1460), quaternion and icosian weights (#2193, #2216), and seed reliability (#1798).
7. Some geometry wins were storage or lexical artifacts (#440, #487).
8. Exact algebra reproduces perfectly but carries no meaning on its own (#952, #985).
9. An exact log plus a prime sieve retrieves. The learned pointer copied recency (#1558, #1607).
10. Chat failures came from missing knowledge and fluency, not retrieval (#1708).
11. Compliance needs a demand paired with its exact short answer (#1958).
12. Hard geometric selection gives keys that are not selected no gradient, so it needs a training bridge (#2195, #2249).
13. Savings counted on paper are not tokens per second (#2200, #2211). Integer serving first cost more energy (#1449).
14. Check where compute goes: readout was 99.97% (#1287), and geometry was 0.05% (#2186).
15. Frozen gates never move after a miss (#887). UNAVAILABLE is not a negative (#661, #986).
16. Keep dormant mechanisms behind gates; never delete them (#515, #1747).
17. Exact integer export and bit-exact serving work (#1254, #1893).
18. State and horizon were the levers, not training length (#1366, #1379).
19. Shift-and-add stores and E8 codebooks beat sign codes (#243, #395).
20. Binding failures came from the training recipe, not the architecture (#1045, #1050).
21. A process that forbids execution produces nothing (#1111). A negative closes a configuration, not a mechanism (#2165).
