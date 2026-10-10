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

The Claude lab drafted these on 10 October from the source and the records, at the owner's request. Correct them in place when a mechanism's understanding changes. The owner does not need to write them.
