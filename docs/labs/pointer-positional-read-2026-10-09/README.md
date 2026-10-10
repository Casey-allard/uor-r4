# The positional read: answer (b) — the value's position is KEPT and the attention never lands on it

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, no new knob, $0.** No v5
re-run. **Read on the CONTROL ARM (arm A, default knobs, byte-for-byte 13 of 13), so this is a
statement about the artifact and not about a modified decoder.**

**Structural qualifier: this artifact has NO MEMORY READER — a pointer fix is a pointer fix.**

## The question and the answer

**At the step where the value should be emitted, which window positions do the kept sources occupy, and
do they reach the second digit at all?**

**Answer: (b) — with a stronger form than the option as written.** At default knobs the pointer's
`select` is `None`, which the code documents as keeping **every source**, so the keep set is the whole
window: **the second digit's position is not excluded from the kept set — the attention simply never
places its mass there.** Measured over all 13 rows:

| | rows | attention at that step |
|---|---:|---|
| rows whose pointer argmax **ever** lands on the value's **FIRST** digit | **3 of 13** | 0.8533, 0.5141, 0.4743 |
| rows whose pointer argmax **ever** lands on the value's **SECOND** digit | **0 of 13** | — |

**The second digit is never the pointer's argmax on any row.** And on the 10 READER rows **neither digit
is ever attended at any step** — it is not that the first digit is attended while the second is dropped;
**neither is attended**, and the mass goes elsewhere entirely.

## The per-step evidence, verbatim, two READER rows

`conv-v5-mem-005` (value `41`, digit ids `[22, 19]`, 14 steps) — every step, `source` = the window
position the pointer chose, `source_id` = the window id there:

```
step  0: emitted 2997  source= 8  source_id= 223  attn=0.3779  matches=False
step  1: emitted 1700  source= 5  source_id=1498  attn=0.7675  matches=False
step  2: emitted  386  source= 6  source_id=2369  attn=0.6518  matches=False
step  3: emitted 1412  source= 5  source_id=1498  attn=0.4000  matches=False
step  4: emitted  435  source=47  source_id=1156  attn=0.4538  matches=False
step  5: emitted  772  source= 8  source_id= 223  attn=0.9202  matches=False
step  6: emitted  291  source=43  source_id=2728  attn=0.6257  matches=False
step  7: emitted   69  source= 5  source_id=1498  attn=0.3230  matches=False
step  8: emitted 1790  source=45  source_id=1044  attn=0.2313  matches=False
step  9: emitted  292  source=67  source_id=1790  attn=0.3214  matches=False
step 10: emitted   80  source=51  source_id= 754  attn=0.4650  matches=False
step 11: emitted   71  source= 5  source_id=1498  attn=0.1898  matches=False
step 12: emitted   16  source=64  source_id= 772  attn=0.4072  matches=False
step 13: emitted    1  source= 8  source_id= 223  attn=0.7021  matches=False
```

`conv-v5-mem-037` (value `98`, digit ids `[27, 26]`, 14 steps):

```
step  0: emitted 2997  source= 8  source_id= 223  attn=0.3505  matches=False
step  1: emitted 1700  source= 5  source_id=1498  attn=0.8009  matches=False
step  2: emitted  386  source= 6  source_id=2369  attn=0.7122  matches=False
step  3: emitted 1412  source= 5  source_id=1498  attn=0.4669  matches=False
step  4: emitted  435  source=47  source_id=1156  attn=0.4319  matches=False
step  5: emitted  772  source= 8  source_id= 223  attn=0.9064  matches=False
step  6: emitted  291  source=43  source_id=2728  attn=0.6094  matches=False
step  7: emitted   69  source= 5  source_id=1498  attn=0.3803  matches=False
step  8: emitted 1790  source=45  source_id=1044  attn=0.2312  matches=False
step  9: emitted  292  source=67  source_id=1790  attn=0.3476  matches=False
step 10: emitted   80  source=51  source_id= 754  attn=0.4631  matches=False
step 11: emitted   71  source= 5  source_id=1498  attn=0.1721  matches=False
step 12: emitted   16  source= 5  source_id=1498  attn=0.4182  matches=False
step 13: emitted    1  source= 8  source_id= 223  attn=0.7055  matches=False
```

**No step in either row selects a stored digit**, and the window ids selected (`223`, `1498`, `2369`,
`1156`, `2728`, `1044`, `754`, `772`) are none of them the value's.

## The finding the two tables give for free: the pattern is nearly IDENTICAL across rows

Two rows with **different values, different keys and different histories** (`41`/`brother` against
`98`/`brother`) choose **the same source positions in the same order** — `8, 5, 6, 5, 47, 8, 43, 5, 45,
67, 51, 5, …` — with attention values that track each other closely (0.3779 vs 0.3505; 0.7675 vs
0.8009; 0.6518 vs 0.7122; 0.9202 vs 0.9064).

**The pointer's positional behaviour is essentially history-insensitive on this artifact.** It is not
failing to find the value because the value is hard to find; it is **looking at a fixed pattern of
positions** and the value is not one of them. That is a stronger and simpler statement than "the second
digit is outranked", and it is consistent with the knob result: `TopK(2)` + a 0.9 gate floor moved
coverage to exactly 3 of 13 — exactly the 3 rows where the argmax happened to land on the first digit
(`mem-008` 0.8533, `mem-024` 0.5141, `mem-040` 0.4743) — because raising the gate does not change
**where** the pointer looks, only how much the copy distribution counts once it is there.

## What this answers and what it does not

**Answers:** the constraint is **not** the keep set (the whole window is kept at default) and **not**
the second digit being outranked by the first. It is that **the pointer's attention mass does not go to
the value's run at all on 10 of 13 rows, never goes to the second digit on any row, and follows a
near-identical positional pattern across different rows.**

**Does not answer:** why the attention pattern is what it is — whether the head was trained on targets
that never required a two-token run (the plan's training-path option) or whether the mask/window
geometry penalises it. **That is the next read, not another knob.**

**Does not claim:** any capability change. No judge was run; `check_pass` on v5 was not re-measured; the
treatment arm's distribution remains unmeasured and is not used here.

## The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The pointer's single-source shape is the binding constraint | **REFUTED in its simple form, replaced**: single-source selection is not the binding constraint — the **attention's positional target** is. The knob moved coverage to exactly the rows where the argmax already landed on the first digit |
| The learned read/emit path | **SPLIT**, and now localized: **attention placement, not keep-set coverage** |

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Read why the attention pattern is positional and row-invariant.** The trace already shows *where* it
looks; the next question is *what it is looking at there* — decode the selected window ids (`223`,
`1498`, `2369`, …) and establish whether they are structural tokens (role markers, separators) rather
than content. If they are, the head is attending the scaffold rather than the conversation, and the fix
is a training-target question for the owner with a timed calibration run first — **not another knob,
and not a wider keep set.**
