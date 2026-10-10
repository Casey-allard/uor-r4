# The pointer split: READER 10, EMITTER 2 — and the full two-digit run is never selected

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, $0** (one rebuild, one
12 s run). No v5 re-run.

**Structural qualifier, kept visible: the artifact that answered the v5 "memory" panel carries NO
MEMORY OPERATOR — only a copy pointer — so "10 of 40 on the memory category" is 10 of 40 for a
copy-pointer dialogue stack on requests *named* memory rows.**

## 1. The instrument is now trustworthy on both counts

The last defect was the values file: tab-separated `id, value, forbid, keys`, and the probe took
**everything after the first tab** as the value, so `digit_ids` were 12-token sequences for two-digit
values. Fixed to column 1, **and asserted rather than checked by eye**, exactly as the turn-grouping bug
was: **every value must be a two-digit number and must tokenize to exactly two ids, or the run returns
an error and is VOID.** The assertion did not fire.

| check | result |
|---|---|
| **byte-for-byte reproduction of the sealed replies** | **13 of 13** |
| **`digit_ids` exactly two tokens on every row** (hard assertion) | **13 of 13** |

## 2. The split, under the four conditions

| condition | result |
|---|---|
| **1. both classes non-empty, or INCONCLUSIVE** | **READER 10, EMITTER 2 — both present, NOT inconclusive** |
| **2. `mem-040` positive control** | **PASSES**: one of its two stored digits **was selected** (1 of 2 selected, 2 emitted) |
| **3. per-step records for two rows of the dominant class** | **shown below** — `mem-005` and `mem-021`, 14 steps each, **0 steps selecting a stored digit** |
| **4. token ids, never strings** | held throughout |

| row | value | class | digits selected / emitted |
|---|---|---|---|
| `mem-005` | 41 | **READER** | 0 / 0 |
| `mem-006` | 75 | **READER** | 0 / 0 |
| **`mem-008`** | **68** | **EMITTER** | **1 / 0** |
| `mem-010` | 85 | **READER** | 0 / 0 |
| `mem-013` | 88 | **READER** | 0 / 0 |
| `mem-021` | 71 | **READER** | 0 / 0 |
| `mem-022` | 89 | **READER** | 0 / 0 |
| **`mem-024`** | **54** | **EMITTER** | **1 / 0** |
| `mem-026` | 99 | **READER** | 0 / 0 |
| `mem-029` | 83 | **READER** | 0 / 0 |
| `mem-037` | 98 | **READER** | 0 / 0 |
| `mem-038` | 71 | **READER** | 0 / 0 |
| `mem-040` | 84 | PARTIAL | 1 / 2 |

### The dominant class is READER: the pointer does not select the stored value

**10 of 13 rows: neither stored digit token is ever the pointer's selected source.** Two audited rows,
verbatim: `mem-005` (value `41`, digit ids `[22, 19]`) runs **14 steps with 0 steps selecting a stored
digit**; `mem-021` (value `71`, digit ids `[25, 19]`) the same. The assignment is auditable, not
asserted.

### The two EMITTER rows are the two that answered "1"

`mem-008` (stored `68`) and `mem-024` (stored `54`) each had **one stored digit selected and none
emitted**. At **step 8** the pointer selected `source_id` **24** (the token for `6`) and **23** (for `5`)
respectively, and the emitted id was **19** — the token for `1` — with `matches_source: false`. **That
is exactly the sealed reply *"Your sister's locker number is 1."*** So on these two rows the pointer did
select the right digit and the decoder dropped it.

### And the finding underneath both: the full run is NEVER selected

**0 of 13 rows — including `mem-040`, the one that delivers the value — had BOTH stored digits selected
as the pointer's source.** The value is a two-token run; **the pointer never selects the whole run.**

## 3. What this settles, and what it does not

**Settles:** on these 13 rows of this artifact, the **reader side dominates** — the pointer selects the
stored value's digits on at most one position and never the full two-token run — so a fix aimed at
*selection* addresses 10 of the 13 rows, while a fix aimed at *emission* addresses the 2 rows that emit
a digit never stored. The two mechanisms are distinct and separately countable.

**Does not settle:**
- It classifies **these 13 rows on this artifact**. It does not say that a reader-side or emitter-side
  fix would **raise the score**.
- It says nothing about the reply half of criterion 1, which is unchanged.
- It is a pointer mechanism, not a memory mechanism: **this artifact has no memory reader**, so nothing
  here transfers to an addressed-memory path without its own measurement.

## 4. The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The learned read/emit path | **SPLIT, first measurement: READER 10, EMITTER 2, PARTIAL 1**, on an instrument that reproduces the sealed replies 13 of 13 byte for byte |

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run; its 10 of 40
stands with the no-memory-reader qualifier.

## Next

**A reader-side fix for the 10 READER rows is the next lever**, with the 2 EMITTER rows as a separate,
smaller target — and **the measurement that would show a fix works is the same trace on the same 13
rows after the change, requiring both digits of the value to be selected as a run.** Build the
intervention on a fresh pre-registered piece, not on a v5 re-run.
