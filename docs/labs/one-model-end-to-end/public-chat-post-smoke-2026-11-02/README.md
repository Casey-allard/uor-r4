# Public integer-chat post-smoke baseline

Lab: deepseek. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md).
Order: [deepseek-6](../../../loop/tasks/one-model-end-to-end/deepseek-6/ORDER.md).
Sheet:
[post-smoke FIT](../../../harness/prep/one-model-end-to-end-public-chat-post-smoke-2026-11-02.md).
Measured source head: `ffe7e5491` (documentation-only branch over `b44402e0c`).

## Evidence-integrity repair

The first raw run remains under
`local/deepseek/public-chat-post-smoke-2026-11-02/`, and none of its 24 process
outputs was overwritten. However, post-run analysis rewrote its
`verification.json` in place after the report had pinned the original hash.
That root is preserved as an evidence-handling failure and is no longer the
authoritative aggregate.

Before rerunning, the exact panel, artifacts, tokenizer, binary and commands
were rechecked. A fresh sibling root,
`local/deepseek/public-chat-post-smoke-2026-11-02-rerun/`, is the replacement.
Its raw outputs are written once, its status and timing files record all 24
processes, and its aggregate is written only after every process finishes. The
rerun is a serving-determinism repair, not a new model question. The decision
rule and claim boundary are unchanged. The rerun's 33 reply structures (text,
ids, stop and history) are identical to the preserved first-run outputs.

## Result

**The product path is connected and deterministic, but the public artifact is not
ready for a useful chat demonstration.** All four fixed memory-anchor checks
failed: neither seed named `Pickle` or `tomato` in its corresponding memory
probe.

The earlier protocol-1 `" Whilk from you."` smoke is not used as model evidence.
The corrected protocol-2 panel emitted different, internally decoded text and
token sequences.

The REPL exposes per-turn ids/s but not its private per-turn seconds. The
memory tables therefore derive seconds from `id count / displayed rate`; the
separate `/usr/bin/time -v` records supply process-level timing and memory.
This timing-precision limitation does not affect the memory classification.

## Instrument result

- 24 `uor-chat` processes produced 33 replies: 11 for each of seed 1, seed 2
  and the seed-1 repeat.
- Every process exited 0; every reply stopped by `eos` or `max_new_tokens`;
  every returned protocol was `uor-r4.literal-role-dialogue/2`.
- Observed reply rates ranged from 144.47 to 350.35 ids/s.
- All 33 reply texts independently decoded from their recorded reply ids.
- The seed-1 repeat matched the first pass in reply text, ids, stop reason and
  history depth for all 11 replies. Timing differed, as expected.
- Peak RSS over all processes was 11,592 KiB. The 24 process timing records sum
  to 2.82 s; no separate orchestrator timer is asserted. The rerun evidence
  retained is 284 KiB. External compute cost was `$0.00`.
- Raw stdout, stderr, `/usr/bin/time -v` records and
  `verification.json` are under
  `local/deepseek/public-chat-post-smoke-2026-11-02-rerun/`.
  `verification.json` SHA-256:
  `45e28a60e49369bbd46d877b9e06a429d98ad6ac64aa640a09928216626e8a4b`.
- `uor-chat` reported a residual contract notice: the retained bundle contract
  says 256 context positions while these artifacts declare 192. No turn in this
  panel reached that limit.

## Fixed inputs

| Panel id | User turns |
|---|---|
| `conv-clar-01` | `Will you finish it before lunch?` |
| `conv-do-01` | `Please say this in a kinder way: Give me that toy now.` |
| `conv-do-02` | `Give me three colors for painting a house.` |
| `conv-talk-01` | `Hello! How is your day going?` |
| `conv-story-01` | `Once upon a time, there was a little bunny who lost his carrot. What happened next?` |
| `conv-unk-01` | `What did I eat for breakfast yesterday?` |
| `conv-mem-01` | `The new puppy at our house is named Pickle.` then `What did we name the new puppy?` |
| `conv-mem-02` | `Grandpa grows tomatoes in a box by the window.` then `He waters them every day.` then `What does grandpa grow?` |

Artifact pins:

| Seed | Artifact SHA-256 | Tokenizer SHA-256 |
|---|---|---|
| 1 | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` | `1ca1ad2fe36ca43b4cc7deeb46e49c275f5172720d2e2fe30fafca2c17ce1514` |
| 2 | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` | `5c09890c37475e1fdc2ae5cb6e70c4613200a0bd6c59a1eafe7f3b4a4311f72c` |

Panel SHA-256:
`d7efc53b17a08fcf450266ae69f4a86ee82ab5fd3f3dbd1fcb58399c0c0cc362`.
Executable SHA-256:
`4f7458c4bfe3b5129033564e97f7cd7d781d838c24f79edf62c92a26109f45a2`.

## One-turn transcripts

### Seed 1

| Panel | Reply text | Reply ids | Stop | History | Seconds | Ids/s |
|---|---|---|---|---|---:|---:|
| `conv-clar-01` | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 29 | 0.048209 | 248.92 |
| `conv-do-01` | `small Pur I G's greatice'm good,uc cobweb!.` | `[563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1]` | `eos` | 42 | 0.054867 | 291.62 |
| `conv-do-02` | `small Pur I G's greatice'm good,uc cobweb!.` | `[563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1]` | `eos` | 36 | 0.047057 | 340.02 |
| `conv-talk-01` | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 31 | 0.039956 | 300.33 |
| `conv-story-01` | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 42 | 0.055041 | 218.02 |
| `conv-unk-01` | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 35 | 0.045596 | 263.18 |

### Seed 2

| Panel | Reply text | Reply ids | Stop | History | Seconds | Ids/s |
|---|---|---|---|---|---:|---:|
| `conv-clar-01` | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 23 | 0.029125 | 206.01 |
| `conv-do-01` | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 32 | 0.041531 | 144.47 |
| `conv-do-02` | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 26 | 0.035042 | 171.23 |
| `conv-talk-01` | `Your G me have all oflkreonddot in am you'm` | `[2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915]` | `max_new_tokens` | 35 | 0.047179 | 339.13 |
| `conv-story-01` | `Your G me have all oflkreonddot in am you'm` | `[2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915]` | `max_new_tokens` | 46 | 0.064346 | 248.66 |
| `conv-unk-01` | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 29 | 0.037522 | 159.91 |

The seed-1 repeat emitted the same text, ids, stop reasons and history depths
shown for seed 1 in all six rows. Its timings were
`0.037181 / 0.055180 / 0.046897 / 0.039961 / 0.055302 / 0.045393` seconds in
the table's panel order.

## Multi-turn memory transcripts

### Seed 1

| Probe/turn | Reply text | Reply ids | Stop | History | Ids/s | Seconds* |
|---|---|---|---|---:|---:|---:|
| `mem-01` turn 1 | `small Pur I G's greatice'm good,uc cobweb!.` | `[563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1]` | `eos` | 38 | 307.08 | 0.052104 |
| `mem-01` turn 2 | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 68 | 278.90 | 0.043026 |
| `mem-02` turn 1 | `small Pur I G's greatice'm good,uc cobweb!.` | `[563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1]` | `eos` | 39 | 312.29 | 0.051234 |
| `mem-02` turn 2 | `small Pur I G's greatice'm good,uc cobweb!.` | `[563,971,456,340,1155,386,972,552,915,561,14,592,3606,3,16,1]` | `eos` | 72 | 349.84 | 0.045735 |
| `mem-02` turn 3 | `small Pretty good, I M coat.` | `[563,971,623,576,561,14,340,379,581,299,16,1]` | `eos` | 99 | 320.40 | 0.037453 |

### Seed 2

| Probe/turn | Reply text | Reply ids | Stop | History | Ids/s | Seconds* |
|---|---|---|---|---:|---:|---:|
| `mem-01` turn 1 | `Your G me have all oflkreonddot in am you'm` | `[2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915]` | `max_new_tokens` | 38 | 310.06 | 0.051603 |
| `mem-01` turn 2 | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 62 | 188.71 | 0.031795 |
| `mem-02` turn 1 | `Your G me have all oflkreonddot in am you'm` | `[2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915]` | `max_new_tokens` | 39 | 304.09 | 0.052616 |
| `mem-02` turn 2 | `Your G me have all oflkreonddot in am you'm` | `[2997,1155,519,500,434,372,78,77,275,2795,70,302,315,745,351,915]` | `max_new_tokens` | 72 | 350.02 | 0.045712 |
| `mem-02` turn 3 | `Your G me legs.` | `[2997,1155,519,2295,16,1]` | `eos` | 93 | 216.40 | 0.027726 |

The seed-1 repeat's five memory replies matched every text, id, stop and history
value above. Its rates were
`322.45 / 290.16 / 312.06 / 350.35 / 320.92` ids/s in table order.

`Seconds*` is derived as reply-id count divided by the product's displayed
two-decimal ids/s. It preserves the observed rate exactly; sub-10 μs precision
is not claimed because `uor-chat`'s REPL prints the rate, not its internal
microsecond timer. The complete 24-process elapsed sum is retained separately
in `verification.json`.

## Decision

| Seed | `Pickle` in `mem-01` turn 2 | `tomato(s)` in `mem-02` turn 3 |
|---|---|---|
| 1 | No | No |
| 2 | No | No |

The registered rule therefore classifies the artifact as **not ready for a
useful-chat demonstration**. The result is ungraded and makes no
answer-correctness, general-language, fresh-phrasing or full-conversation
claim. The published artifact's own README already states that it is not an
assistant and not chat-capable. The seeds are initialization replicates on a
common batch stream, not independent data-order samples, so no cross-seed noise
or capability estimate is inferred.
