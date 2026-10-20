# The reader-versus-emitter split: the memory-read instrument cannot run on this artifact, and the pointer trace can

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no
training, no model run, $0.** No v5 re-run. **Result: the split is NOT MEASURABLE on the sealed v5
artifact with the read-path diagnostic, and the instrument that CAN measure it is identified and
specified below.**

## The question, stated so it cannot drift

For the 13 numeric v5 rows: **does the learned read path SELECT the right token before emission?**

1. Right token selected and the reply does not contain it → the failure is in the **EMITTER**.
2. Right token not selected → the failure is in the **READER**.

The addressability probe ([numeric-addressability-probe-2026-10-09](../numeric-addressability-probe-2026-10-09/README.md))
established that the representation is not the problem. This is the last unexplained link.

## Finding 1 — the read-path diagnostic exists, is public, and returns the selection

`Session::memory_read_diagnostic(model, target) -> Option<MemoryReadDiagnostic>`
(`crates/uor-r4-core/src/native_geometric/memory_training.rs:53`) returns

```text
predicted_token, candidate_routes, target_routes,
query_context_routes_with_registered_row, query_context_routes_without_registered_row
```

`predicted_token` **is** "what is selected before emission" for a memory read, and `target_routes` is
exactly the count of admitted routes carrying the expected token. That is the instrument the question
needs.

## Finding 2 — it cannot run on the artifact that answered v5, and the reason is structural

The diagnostic returns `Ok(None)` unless the session's memory operator has one of three schemas
(`QUERY_CONTEXT_MEMORY_SCHEMA`, `OCCURRENCE_MEMORY_SCHEMA`, `RESPONSE_MEMORY_SCHEMA` — the `/3`, `/4`
and `/5` readers), and unless the control is not `MemoryDisabled`.

**The v5-answering artifact carries no memory operator at all.** Its whole configuration is:

```json
{"arch":"geometric","vocab_size":4096,"width":576,"heads":8,"mlp_hidden":732,
 "context":384,"pattern":"rrarrarrar","read":"l2","rotation":true,"seed":1,
 "pointer":{"dim":32,"init_seed":1}}
```

There is a **`pointer`** (a copy operator) and **no memory reader**. So on `chat-29m-B-lr5e-4` the
diagnostic is `None` **by construction**, for every row — not because the model failed to select
anything, but because there is no memory read to diagnose.

**Consequence: the reader-versus-emitter split is NOT MEASURABLE on the sealed v5 artifact with this
instrument, and this record reports that rather than a suggestive reading.** The diagnostic belongs to
the memory-panel model family (the `/3`–`/5` readers the v3/v4/v5 *training* work used), not to the
plain dialogue stack that produced the v5 replies.

## Finding 3 — the instrument that CAN separate them, and it is cheap

**The artifact does carry a copy pointer, and the pointer's per-step selection is observable.**
`StackModel::next_scores_with_pointer` returns the emitted logits **and the selection** for each step
(`crates/uor-r4-training/src/geometric_stack.rs:3544`), and `pointer_copy_trace()` (`:3494`) turns the
per-step record on; `greedy_reply_with_copy_stop` already records those steps when the trace is asked
for. So there is a **runnable, CPU-only, untrained** path to the same split:

- for each of the 13 numeric rows, generate the reply **with the pointer trace on**;
- at each step record **what the pointer selected** and **what was emitted**;
- classify the row: the stored value's digit selected but not emitted → **EMITTER**; never selected →
  **READER**.

**This is the smallest probe that separates the two**, it needs no training, and it uses an operator
the artifact actually has. **It is specified here and NOT run** — this piece did not build the trace
binary, and it does not report a partial or suggestive reading in its place.

## Designed to fail loudly — how the next run must be built so it cannot pass by accident

Following the addressability probe's standard, the trace must assert **inequalities as well as
equalities**, so a trace that cannot discriminate shows up as a failure rather than a pass:

1. **Both classes must be non-empty or the probe is INCONCLUSIVE, not confirmatory.** If every row
   traces as READER, or every row as EMITTER, the trace is not discriminating and the result is
   "inconclusive".
2. **`mem-040`, the one numeric row that emits the stored value exactly, is the positive control.**
   Its trace must show the stored digit selected. If it does not, the trace is broken.
3. **A row must be reported as EMITTER only if the selected token is the stored digit and the reply
   does not contain it** — selection and emission are compared as **token ids**, not as strings.
4. **The twelve failures must not all land in one class** unless the per-step record is shown for at
   least two of them, so a class assignment can be audited rather than trusted.

## The ledger, at its own strength — four readings, three dead, one survivor

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers are **2 tokens against 3.74** for words |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a **transposition** of its `84` and it passes; **0 of 12** numeric failures name the distractor |
| Value addressability | **REFUTED** | the codec recovers a two-token run as **one ordered value with an interval**, order preserved, `74` vs `84` distinguishable |
| **The learned read/emit path does not deliver a correctly stored value** | **OPEN — SURVIVES, and is now reframed** | this piece splits its two halves: the **read** half is not measurable on this artifact (no memory operator), so what remains is **the pointer/selection-versus-emission half, which is measurable and unmeasured** |

**A fifth dead reading would have been welcome; instead this piece narrows the survivor.** The
question is no longer "does the learned path deliver" but "**does the pointer select and the emitter
drop, or does the pointer never select**" — and that is now a runnable measurement rather than a
hypothesis.

## What does not change

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged. Nothing in this piece changes
that.** v5 was not re-run; its 10 of 40 stands.

## Next

**Build the pointer-trace probe and run it on the 13 numeric v5 rows**, with `mem-040` as the positive
control and the four fail-loudly conditions above. It is CPU only, untrained, and it separates the
reader half from the emitter half in one run. Whoever builds it should report **INCONCLUSIVE** rather
than a reading if the two classes do not both appear.
