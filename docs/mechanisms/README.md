# Mechanism briefs

One page per novel mechanism the project is built to invent and test. Each brief states:
- the idea from first principles, and what it replaces;
- where it lives in code;
- what has been tried, honestly, including whether each test could judge it;
- what success and failure look like;
- **what a fair test needs**;
- the open design questions.

The labs work from these, not only from milestone headline numbers. Every pre-registration of a mechanism run is reviewed against its brief before compute, and the review is posted as `TEST FITNESS: FIT` or `NOT FIT` on the milestone ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)). A negative closes a configuration, never one of these mechanisms; stopping a mechanism needs the owner.

Status as of13 October2026. D32 requires discrete model computation and instruments,
including learning, without float gradients/optimizers or matrix multiplication.
Both training holds remain HELD. The discrete path at `a2a5b89c` passes eleven
admitted CUDA construction controls after Claude FIT. Earlier attempts retain
their launch blocker/review-order history. Delivery: #2379; M1's quality target
remains SUSPENDED pending its separate successor sheet.
No learned-language result follows. Older float trunk/results remain preserved
inactive through [#2370](https://github.com/UOR-Foundation/uor-r4/pull/2370).

**Current discrete source.**

| Mechanism | Milestone | Status | Brief |
|---|---|---|---|
| Exact SpiralCore/FBS moves and shared integer vector writer | M1 #2029 / M4 #2032 | source a2a5b89c;11 admitted CUDA controls PASS after Claude FIT; delivery #2379, training held; M1 quality target SUSPENDED | [discrete-geometric-learning.md](discrete-geometric-learning.md) |
| Prime-rank indexing and complex-plane zeta-phase routing | M1 #2029 / M4 #2032 | D32 addendum #2398; brief authored; pre-registration pending FIT review | [prime-zeta-complex-routing.md](prime-zeta-complex-routing.md) |

**Pre-D32 operator phase — preserved inactive float source scopes.**

| Mechanism | Milestone | Status | Brief |
|---|---|---|---|
| SpiralCore BFS plus retained ternary FBS projection | M1 #2029 / M2 #2030 | source, tested; the trunk's projection (#2279) | [spiralcore-fbs-projection.md](spiralcore-fbs-projection.md) |
| Rank-margin objective: a hard hinge over declared rivals, raw-score units, no exponential | M1 #2029 | source, tested; rung 0's objective (#2255) | [rank-margin-objective.md](rank-margin-objective.md) |
| Starling read: ~7 topological neighbours on the 600-cell, no softmax | M4 #2032 | historical rank/float source; discrete murmuration is a separate design | [starling-read.md](starling-read.md) |
| Flock / rank-table reads (softmax-free attention) | M4 #2032 | served; measured beside a softmax stack (historical) | [flock-rank-reads.md](flock-rank-reads.md) |

The G2 Hamilton gate (#2260) and the geometric recurrence gates (#2293) are trunk operators recorded in
[gate ops](../labs/gate-ops-2026-10-12/README.md) and [recurrence gates](../labs/recurrence-geometric-gates-2026-10-13/README.md).
They have no separate brief: each replaces a squashing function with a fixed geometric one, not a new
learning idea. Pictures of all four operators: [the geometry page](../geometry.md#the-operators-of-the-compliant-trunk).

**Other mechanisms.**

| Mechanism | Milestone | Status | Brief |
|---|---|---|---|
| Exact addressed memory (prime/semiprime keys, product-key memory) | M1 #2029, M3 #2031 | built; product-key memory trained in (historical) | [exact-addressed-memory.md](exact-addressed-memory.md) |
| Read binding (supervised value binding) | M1 #2029 | KEEP (historical float form) | [read-binding.md](read-binding.md) |
| Invariant-addressed read: route a query to an address instead of scoring every earlier position | M1 #2029 | source; its router campaign was cancelled under D23/D24 (a configuration, not a negative) | [invariant-addressed-read.md](invariant-addressed-read.md) |
| Pointer-copy head with an exact identity key | M1 #2029 | historical evidence; the margin objective refuses a pointer today | [pointer-identity-key.md](pointer-identity-key.md) |
| Live factual/local H4 and Generate prototype learning credit | M2 #2030 | source; training held | [unified-geometric-carrier-credit.md](unified-geometric-carrier-credit.md) |
| Protected / discrete legal construction | M2 #2030 | reopened by D22; not active | [protected-legal-construction.md](protected-legal-construction.md) |
| E8 orbit memory: an exact-E8-addressed memory in place of the MLP, no softmax | M4 #2032 | source; rung 1 KEEP and iterate (historical) | [e8-orbit-memory.md](e8-orbit-memory.md) |
| Spin-native weights: quaternion and icosian maps (Cayley-table products) | M4 #2032 | quaternion weights and 2-bit norms KEEP (historical) | [spin-native-weights.md](spin-native-weights.md) |
| Ternary / XOR arithmetic: a multiplier-free weight alphabet (owner-suggested, not adopted) | M4 #2032, M5 #2033 | assessed, not adopted | [ternary-xor-arithmetic.md](ternary-xor-arithmetic.md) |
| Ternary / XOR evidence fact sheet: 1-bit and ternary inference, what is measured and what is estimate | M4 #2032, M5 #2033 | reference | [ternary-xor-evidence-factsheet.md](ternary-xor-evidence-factsheet.md) |

The Claude lab drafted these on 10 October from the source and the records, at the owner's request. Correct them in place when a mechanism's understanding changes, and update the Status column when a mechanism's state changes (see [bookkeeping](../bookkeeping.md)). The owner does not need to write them.
