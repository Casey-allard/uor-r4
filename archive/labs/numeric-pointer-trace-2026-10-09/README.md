# The pointer trace is BUILT and BROKEN: its positive control fails, so no row is classified

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no
training, $0** (one release build, 10.6 s run). No v5 re-run.

## 0. The structural finding, first, because it outranks everything below

**THE ARTIFACT THAT ANSWERED THE v5 "MEMORY" PANEL CARRIES NO MEMORY OPERATOR AT ALL.**

Its entire configuration is:

```json
{"arch":"geometric","vocab_size":4096,"width":576,"heads":8,"mlp_hidden":732,
 "context":384,"pattern":"rrarrarrar","read":"l2","rotation":true,"seed":1,
 "pointer":{"dim":32,"init_seed":1}}
```

There is a **copy pointer** and **no memory reader**. Consequences, and they are bigger than this
piece's result:

1. **The v5 memory rows were never exercising a memory reader.** `memory_read_diagnostic` returns
   `None` **by construction** on every row — established in
   [numeric-reader-vs-emitter-2026-10-09](../numeric-reader-vs-emitter-2026-10-09/README.md).
2. **What the panel actually measured was the plain dialogue stack with a copy pointer.** The
   mechanism in play for `multi_turn_memory` on this artifact is **copy/pointer selection**, not
   addressed memory.
3. **So "10 of 40 on the memory category" needs a qualifier**: it is 10 of 40 for a copy-pointer
   dialogue stack on requests *named* memory rows, not 10 of 40 for an addressed-memory path. The
   qualifier has been added to the v5 acceptance record and the current-state entry in this
   delivery.
4. **It also reframes the standing reading**: "the learned read/emit path does not deliver the value"
   is, on this artifact, "**the POINTER does not deliver the value**". The pointer selects a span to
   copy; a two-digit value needs both tokens selected as a run.

## 1. What was built

`crates/uor-r4-training/src/bin/numeric-pointer-trace.rs` — loads the artifact and the frozen
tokenizer, turns on `set_pointer_copy_trace(true)`, generates each row through the same
`reply_panel` → `greedy_reply_with_copy_stop` path `chat-grade reply` uses, and records per step the
**emitted id** and the **pointer's selected source id**, comparing them **as token ids, never
strings**. Class rules, fixed before the run:

| class | rule |
|---|---|
| `READER` | the stored value's digit token never appears as any step's `source_id` |
| `EMITTER` | some step's `source_id` IS the stored digit token while that step's emitted `id` is not |
| `PARTIAL` | the pointer selects one digit of a two-digit value but never the other |
| `DELIVERED` | every digit token selected **and** every one emitted |

## 2. The run, and the loud failure

The trace ran on all 13 numeric rows in **10.6 s** and produced `READER: 12, PARTIAL: 1`.

**BY PRE-REGISTERED CONDITION 2, THAT IS NOT A RESULT.**

`conv-v5-mem-040` — the positive control, the one numeric row that **emits the stored value exactly**
(*"Your sister's locker number is 84."*) — traces as **`READER` with 0 of its digit tokens selected**.
The control failed, so **the trace is not observing the path that produced the v5 replies, and every
other row's classification is void.**

### The evidence that it is broken, not merely surprising

**Two rows with completely different sealed replies emit byte-identical id sequences in the probe:**

| row | sealed v5 reply | probe emitted ids |
|---|---|---|
| `mem-040` | *"Your sister's locker number is 84."* | `[2997, 1700, 386, 1412, 435, 772, 291, 69, 1790, 292, 80, 71, 16, 1]` |
| `mem-006` | *"I'm a helpful assistant that runs on your computer."* | `[2997, 1700, 386, 1412, 435, 772, 291, 69, 1790, 292, 80, 71, 16, 1]` |

**Identical.** And the pattern is general: 2 rows share one 12-id reply, 2 share a 14-id reply, 2
share a 9-id reply. **The probe is producing history-insensitive replies** — the row's own turns are
not reaching the model on this path — so its per-step selection cannot be compared to anything.

**Therefore: the reader-versus-emitter split is still NOT MEASURED, and this piece does not report
one.** The `READER: 12` count is an artefact of a probe that is not running the pipeline it claims to
run, and it must not be read as evidence for either class.

## 3. What the failure teaches, and what the next attempt must do

1. **The probe's generation path must first reproduce the sealed replies byte for byte** on the same
   rows and the same artifact, before any classification is trusted. That is the missing control and
   it is cheap: generate the 13 rows with the probe and with `chat-grade reply`, and require the id
   sequences to be **identical**.
2. **The likely fault is the history construction on the probe's path.** The probe calls
   `reply_panel` with a 13-row subset and the same protocol; the sealed run called it with the whole
   panel. Either the subset changes the encoded history, or the closure is not receiving the
   per-row history the sealed path used. **This is a diagnosis to make, not a guess to build on.**
3. **Do not re-run this probe as-is.** A trace whose positive control fails cannot be repaired by
   more rows.

## 4. The ledger

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers are 2 tokens against 3.74 for words |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a transposition of its `84` and it passes; 0 of 12 name the distractor |
| Value addressability | **REFUTED** | the codec recovers a two-token run as one ordered value with an interval |
| The learned read/emit path | **OPEN, and this piece adds a structural qualifier rather than an answer** | there is **no memory reader** on this artifact, so the question is the **pointer's**; the pointer trace is built but **not yet trustworthy** |

**Five pieces into this line, the expensive steps have all been avoided by cheap ones that ran
first** — and this piece is the pattern working again: a broken instrument was caught by its own
pre-registered control in a 10.6-second run, before any of its 13 classifications could be written
into a conclusion.

## What does not change

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run; its
10 of 40 stands, now **with the qualifier that no memory reader was involved**.

## Next

1. **Make the probe reproduce the sealed replies byte for byte** on the 13 rows — the missing control
   — and diagnose the history construction if it does not.
2. **Then re-run the trace** and report the class split under the four loud-failure conditions.
3. **Keep the qualifier on every v5 memory number**: it measures a copy-pointer dialogue stack, not
   an addressed-memory path.
