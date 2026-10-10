# Why the numeric memory rows fail: the value is not delivered, and it is not confusion

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **Diagnosis only — no model
run, no intervention, and v5 was NOT re-run. CPU only, no pod, $0.** Every number below was
recomputed from the sealed v5 acceptance replies and the frozen tokenizer; the source artifacts are
named per claim.

## What this explains

The v5 declared acceptance run scored **10 of 40** against a declared ≥ 34
([v5-declared-acceptance-2026-10-09](../v5-declared-acceptance-2026-10-09/README.md)). Its failure
breakdown showed a cluster: distractor-key rows with a word value passed 9 of 23, with a **number**
value **1 of 11**, and the six keyless rows 0 of 6. This piece asks why, from the existing artifacts.

## The ledger, at the strength each claim actually earned

Four claims were made in the course of this diagnosis. Two of them are REFUTED and are recorded rather
than replaced, because they are the ones most likely to be rebuilt by the next reader.

### MEASURED — the vocabulary has no multi-digit token

From the frozen tokenizer `d36d3e87…`, recomputed here:

| | |
|---|---|
| vocabulary size | **4096** |
| pure single-digit tokens | **10** — `0`–`9` |
| **pure multi-digit tokens** | **0** |
| superscript digit slots | **3** — `¹ ² ³` |

So a two-digit value is two tokens and a four-digit value is four. **Three of the thirteen digit slots
in a 4096-token budget are superscripts while no multi-digit token exists at all** — reclaimable
vocabulary sitting idle, and part of the funding for any number-aware tokenizer.

### MEASURED — numeric rows fail far more often, and the split is exact

Recomputed from `data/panels/conversational-v5-checks.tsv` crossed with the sealed replies:

| row type | rows | failed | failure rate |
|---|---:|---:|---:|
| numeric value | 13 | **12** | **92.3 %** |
| word value | 27 | **18** | **66.7 %** |
| no distractor keys | 6 | **6** | 100 % |

**A 25.6-point gap between numeric and word rows**, with `expected_value` passing **40 of 40** — the
check can always be satisfied, so the check is not the obstacle.

### MEASURED CONTROL, REPLICATED — within-key comparisons hold everything but the value fixed

Keys appearing on more than one row, with context, check kind and history held constant inside each
key:

| key | n | values | type | pass |
|---|---:|---|---|---:|
| `brother\|brother's` | 6 | 41 68 71 54 98 84 | numeric | **1 / 6** |
| `grandpa\|grandpa's` | 3 | Tyler Phil Luis | **word** | **2 / 3** |
| `piano\|piano's` | 3 | 75 89 71 | numeric | **0 / 3** |
| `treehouse\|treehouse's` | 2 | 85 99 | numeric | **0 / 2** |

**The same key carries opposite outcomes by value type**, which kills the obvious objection that the
`brother` rows are simply harder: `grandpa` has the same structure with word values and passes 2 of 3.
Numeric within-key totals **1 of 11**, which is the same measurement as the 1-of-11 distractor-key
numeric cell of the 4×2 table, computed two independent ways.

### REFUTED (the Lead's, recorded as such) — token count explains the cluster

Numbers tokenize **better** than words: every numeric term is **exactly two tokens**, one per digit
(`41` → `4`,`1`; `84` → `8`,`4`), while word terms average **3.74 tokens**. So sequence length cannot
explain a deficit that runs the other way. Recorded so nobody rebuilds it.

### REFUTED (the Lead's, recorded as such) — minimal pairs, or digit order

The sharper version of the same idea was that numeric failures come from confusing near-identical
digit runs — `41`/`45`, `68`/`99`, `71`/`73`. **They do not.** `mem-040` — the single numeric row that
passes — has expected `84` and planted distractor `74`, a **digit transposition**: the most confusable
pair the panel could construct, and the model returns `84`. `mem-037`, whose `98` and `76` share no
digit position, fails. And the decisive cut below shows **zero of the twelve numeric failures name the
planted distractor at all**.

### OPEN, WITH A REPLICATED CONTROL — value addressability

`mem-037` and `mem-040` are a matched pair: identical key (`brother|brother's`), identical kind
(`exact`), identical history, **values `98` and `84`, opposite outcomes**. The key is not the
discriminator, the context is not, the check is not. The value alone decides — and the within-key table
replicates that across three more keys.

## The decisive cut: what the failing numeric replies actually emit

This is the measurement that settles the behavioural question, and it was free.

**The value was in the context the model saw, every time.** The expected value appears in the user
turns for **40 of 40** memory rows, including **13 of 13** numeric rows. The failure is not context
loss.

**The failing numeric replies do not emit a value at all.**

| what a failing numeric row emits | rows |
|---|---:|
| **no digit anywhere in the reply** | **10** |
| a digit, but not the stored value | **2** |
| names the planted distractor | **0** |
| **total failing numeric rows** | **12** |

`mem-005` (stored `41`) answers *"Your sister's name is Carcassonne."*; `mem-006` (stored `75`) answers
*"I'm a helpful assistant that runs on your computer."*; `mem-010` (stored `85`) answers *"The garage
code is the code word."*; `mem-037` (stored `98`) answers *"Your sister's name is Carcassonne."* The
two that do emit a digit emit **`1`** — `mem-008` (stored `68`) and `mem-024` (stored `54`) both answer
*"Your sister's locker number is 1."* And the **one passing numeric row emits the stored value
exactly**: `mem-040` answers *"Your sister's locker number is 84."*

So the failing replies are fluent, correctly framed sentences **with the value slot empty or filled by
something that was never stored.** The value is not mis-ordered on the way out; it does not arrive.

### The subdivision of the 30 failures, by what the reply does

| what the reply does | rows | row type |
|---|---:|---|
| emits no number at all | 10 | **all numeric** |
| names no value | 10 | **all word** |
| names the planted distractor | 8 | **all word** |
| emits a number not in the context | 2 | **all numeric** |

**Every single planted-distractor confusion is a word row, and every numeric failure is a
value-emission failure.** The two mechanisms are cleanly separated by value type — which is a fourth
independent refutation of the digit-order reading, from the behavioural side.

## The six keyless rows: not a panel defect

The six are `mem-011, -012, -013, -027, -028, -029`; **all six are 3-turn rows**, four word-valued and
**two numeric** (`-013` stored `88`, `-029` stored `83`).

- **They are not unpassable by construction.** The `expected_value` control passes **40 of 40**,
  including these six: every one is passed by its own bare expected spelling.
- **They fail for their value type's reason, not for a missing key.** The two numeric ones emit no
  number (`-013` *"Our tent is in a small, dark room."*; `-029` *"Our tent is in a bustling city."*),
  which is the numeric signature; the four word ones name no value, which is the word signature.

**So the 0-of-6 cell is not an independent cause and there is no panel defect to correct.** The
no-key set **overlaps** the numeric set (2 of 6) and otherwise behaves like its word peers. A reader
must not add 1-of-11 and 0-of-6 as though they were disjoint. The clean single statement the join
gives: **numeric values fail at 92.3 %, word values at 66.7 %, with the six keyless rows splitting
4 word / 2 numeric and failing for the same type-specific reasons.**

## Magnitude versus type

The numeric deficit is **not** magnitude: every numeric value on this panel is a **two-digit** number
drawn from a two-digit pool, so there is no multi-digit/single-digit contrast to measure, and the
`expected_value` control shows all of them are satisfiable. It is **not** position: the value sits in
an earlier user turn on all 13 numeric rows and on all 40 rows, and the failure rate does not track
turn position (the six keyless 3-turn rows split by type, not by length). **It is the value type** —
and within the numeric type, the deficit is emission, not selection.

## What this is and is not

- **It is a representation finding.** The tokenizer cannot express a two-digit value in one token, so
  the value has no single address; and the behaviour shows the value not arriving rather than arriving
  wrong. This is the project's own principle — *more scalar features cannot recover distinctions
  erased by representation* — measured on the panel that produced the cluster.
- **It is not a training-volume finding.** A model that had seen more numbers would still be reading a
  value spread across positions.
- **It is not a token-count finding**, and the two readings that said otherwise are recorded above as
  refuted.
- **It is not a panel defect.** The six keyless rows are satisfiable and fail for type-specific
  reasons.
- **It is a diagnosis, not a fix.** Nothing was changed and v5 was not re-run.

## The two named options, not started

1. **A number-aware tokenizer** giving a value **one token and therefore one address**. This is now
   the **direct** fix rather than the cheap one: if a two-token value has nowhere to live, giving it
   one address is the mechanism-level change. The three idle superscript slots and the absent
   multi-digit tokens are part of its budget.
2. **An explicit numeric slot in the memory path**, so a value is addressed *as a value* rather than
   as a digit run. This is the more native version of the same idea and the closer fit to the stated
   architecture (exact addressed memory over prime/zeta/R4 state).

**Neither is started here.** The `mem-040` specimen — a transposed distractor, one token per digit,
and the stored value returned exactly — is the evidence that decides which to try first, and whichever
is chosen gets its own pre-registered piece on a **fresh sealed panel**, not a v5 re-run: v5 is
development evidence now, exactly as v4 became.

## Next

1. **Test the addressability hypothesis directly, on a fresh panel built for it**: rows whose numeric
   value and planted distractor are controlled for magnitude, digit overlap and position, so the
   emission failure can be attributed to the value's representation rather than to the row. The
   `mem-040` matched pair is the template.
2. **Then, and only then, try one of the two options** on that fresh panel — number-aware tokenizer
   first if the address is the mechanism, memory-path numeric slot first if the read is.
3. **Do not re-run v5.** It is development evidence; the 10 of 40 stands as its declared reading.

Evidence: the sealed v5 acceptance replies and report in
`icloud:UOR-R4/results/deepseek/v5-declared-acceptance-2026-10-09.tar` (346,624 bytes, md5
`27518f606964dc316631e6afb0d52239`); `data/panels/conversational-v5-checks.tsv`; the frozen tokenizer
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; and this piece's bundle
`icloud:UOR-R4/results/deepseek/v5-numeric-diagnosis-2026-10-09.tar` (25,600 bytes, md5
`1b7b37d5c039e5412653228067c47428`) with the per-row value/reply table and `verify.txt`.
