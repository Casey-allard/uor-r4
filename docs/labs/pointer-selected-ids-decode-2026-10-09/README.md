# Decoding what the pointer attends: the sentence FRAME, never the varying slot

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, no knob, $0.** Read on the
**control arm**. No v5 re-run. **Structural qualifier: this artifact has NO MEMORY READER.**

## 0. The index check first, as required

**Vocabulary 4096, maximum id 4095, and every selected id is in range (223 … 2728).** The trace's
indexing is sound, so the window readings of the last three pieces are **not** in question.

## 1. The decode

**The ids the pointer selects, decoded with the frozen tokenizer `d36d3e87…`:**

| id | token | |
|---:|---|---|
| **223** | **`'Ġ'`** | a **bare space** — and it carries the **highest attention in both tables (0.9202, 0.9064)** |
| 1498 | `'Ġbrother'` | content subword |
| 2369 | `'Ġtakes'` | content |
| 1156 | `'Ġbus'` | content |
| 2728 | `'ser'` | content |
| 1044 | `'ĠWh'` | content |
| 754 | `'Ġtake'` | content |
| 772 | `'ĠC'` | content |
| 1790 | `'ass'` | content |

**The value's own ids, as the control:** `mem-005` `41` → `22 → '4'`, `19 → '1'`; `mem-037` `98` →
`27 → '9'`, `26 → '8'`; `mem-008` `68` → `24 → '6'`, `26 → '8'`; `mem-040` `84` → `26 → '8'`,
`22 → '4'`. **The value is a run of SINGLE-DIGIT tokens**, which is the whole reason it is a run.

**The structural candidates are never selected:** `<|bos|>` (0), `<|eos|>` (1), `.` (16), `:` (28) appear
in **no** step of either table.

## 2. The answer: neither "structural" nor "arbitrary" — it is the FRAME, and the variable slot is skipped

The selected ids are **content subwords, not role markers or separators** — so the structural answer is
**not** what the data gives. But they are not arbitrary either: they are the **recurring sentence frame**
of the panel's own construction. The two rows compared share the key `brother|brother's`, and the panel
builds rows on a shared frame per key, so `'Ġbrother'`, `'Ġtakes'`, `'Ġbus'` … are the **same frame
tokens** in both.

**That means my previous piece's phrase "history-insensitive" was overstated and is corrected here.**
The invariance I measured is the **frame's** invariance, not a demonstration that the head ignores
history. **The correction strengthens the finding rather than weakening it:** the two rows differ in
**exactly one respect — the value (`41` against `98`)** — and the pointer attends the frame they share
and **never the slot that differs**. The highest-attention selection in both tables is the **bare space
token `'Ġ'`** (0.92, 0.91).

**So the nearest honest statement is: the pointer attends the frame and skips the varying slot.** The
digits — the only tokens that distinguish the two rows — are selected on **0 of 13 rows for the second
digit and 3 of 13 for the first**, and never both.

## 3. What this implies, stated against the pre-registered options

| option | verdict |
|---|---|
| **STRUCTURAL** (role markers, separators, padding, scaffold) | **NO** — the selected ids are content subwords; the structural candidates are never selected |
| **CONTENT AT A FIXED OFFSET** (a position rather than a value) | **PARTLY, and it is the frame's offset** — the positions recur because the frame recurs, and the pointer follows the frame, not the value |
| **NEITHER** (arbitrary content, no consistent structure) | **NO** — the recurrence is explained by the shared frame |
| ids do not decode / out of range | **NO** — vocabulary 4096, all ids in range |

**The implication is the owner-level one, not a knob:** the head attends the sentence frame and never the
varying slot, which is a **training-target** property — the targets did not require it to look at the
slot that changes. Per the pre-registered mapping that belongs to the owner **with a timed calibration
run first**, and it is **not** addressed by `TopK`, the gate, or any wider keep set — the previous
piece's knob moved coverage only where the argmax already happened to land on a digit.

## 4. What this does NOT settle

- **It does not prove the head was trained on such targets** — it measures the attention on 13 rows of
  one artifact. The training-target reading is the natural explanation and it is **not** measured here.
- It says nothing about the reply half of criterion 1 and nothing about the addressed-memory path.
- No capability change: no judge was run; v5's `check_pass` was not re-measured.

## 5. The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The pointer's single-source shape is the binding constraint | **REFUTED in its simple form, replaced** |
| "The pointer's behaviour is history-insensitive" | **CORRECTED — the invariance is the frame's, not history's** |
| **The pointer attends the sentence frame and never the varying slot** | **MEASURED on 13 rows** — the smallest true statement this line has reached |

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**A training decision for the owner, not another measurement and not a knob.** The head must be given
targets that require the varying slot — the value's digit run — rather than the frame it already
attends. That needs a **pre-registered training piece with a fresh sealed panel** (a retrained artifact
cannot be compared to v5's 10 of 40) and a **timed calibration run before any estimate**, since the
ladder's throughput is still not citable.
