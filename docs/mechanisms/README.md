# Mechanism briefs

One page per novel mechanism the project is built to invent and test. Each brief states:
- the idea from first principles, and what it replaces;
- where it lives in code;
- what has been tried, honestly, including whether each test could judge it;
- what success and failure look like;
- **what a fair test needs**;
- the open design questions.

The labs work from these, not only from milestone headline numbers. Every pre-registration of a mechanism run is reviewed against its brief before compute, and the review is posted as `TEST FITNESS: FIT` or `NOT FIT` on the milestone ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)). A negative closes a configuration, never one of these mechanisms; stopping a mechanism needs the owner.

| Mechanism | Milestone | Brief |
|---|---|---|
| Flock / rank-table reads (softmax-free attention) | M4 #2032 | [flock-rank-reads.md](flock-rank-reads.md) |
| Exact addressed memory (prime/semiprime keys, product-key memory) | M1 #2029, M3 #2031 | [exact-addressed-memory.md](exact-addressed-memory.md) |
| Pointer-copy head with an exact identity key | M1 #2029 | [pointer-identity-key.md](pointer-identity-key.md) |
| Read binding (supervised value binding) | M1 #2029 | [read-binding.md](read-binding.md) |
| Protected / discrete legal construction | M2 #2030 | [protected-legal-construction.md](protected-legal-construction.md) |
| Live factual/local H4 and Generate prototype learning credit | M2 #2030 | [unified-geometric-carrier-credit.md](unified-geometric-carrier-credit.md) |
| Starling read: ~7 topological neighbours on the 600-cell, no softmax | M4 #2032 | [starling-read.md](starling-read.md) |
| E8 orbit memory: an exact-E8-addressed memory in place of the MLP, no softmax | M4 #2032 | [e8-orbit-memory.md](e8-orbit-memory.md) |
| SpiralCore BFS plus retained ternary FBS projection | M1 #2029 / M2 #2030 | [spiralcore-fbs-projection.md](spiralcore-fbs-projection.md) |
| Spin-native weights: quaternion and icosian maps (Cayley-table products) | M4 #2032 | [spin-native-weights.md](spin-native-weights.md) |
| Ternary / XOR arithmetic: a multiplier-free weight alphabet (owner-suggested, not adopted) | M4 #2032, M5 #2033 | [ternary-xor-arithmetic.md](ternary-xor-arithmetic.md) |
| Invariant-addressed read: route a query to an address instead of scoring every earlier position | M1 #2029 | [invariant-addressed-read.md](invariant-addressed-read.md) |
| Rank-margin objective: a hard hinge over declared rivals, raw-score units, no exponential | M1 #2029 | [rank-margin-objective.md](rank-margin-objective.md) |
| Ternary / XOR evidence fact sheet: 1-bit and ternary inference, what is measured and what is estimate | M4 #2032, M5 #2033 | [ternary-xor-evidence-factsheet.md](ternary-xor-evidence-factsheet.md) |

The Claude lab drafted these on 10 October from the source and the records, at the owner's request. Correct them in place when a mechanism's understanding changes. The owner does not need to write them.
