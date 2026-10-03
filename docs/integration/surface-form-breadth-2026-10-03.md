# Surface-form breadth in the synthetic memory corpus (phrase lock-in) — 3 October 2026

Status: measured (draft PR #1645; refs #1512, #820).

**Result in one line.** Training with eight surface forms per relation instead of
one lifts reading an unseen *statement* form from 37.8% to 94.6% (exact McNemar
p = 3.1e-11) but leaves unseen *question* forms at 58.1% -> 63.5% (p = 0.54), so
surface-form breadth generalises, and the residual binding failure is
request-form-specific. A one-word substitution of the trained question costs
almost nothing (95%), so the lock is on the sentence *frame*, not the string.

## Question

Training on 76 relations was reported to give 14/120 hits on unseen relations
while training on 10 gave 0/120, but a one-letter change in a relation phrase
(`color` -> `colour`) breaks binding, and the model fails to answer a question
about a fact it has just been given while it can echo a value verbatim. The
working hypothesis is **phrase lock-in**: binding is keyed to the exact trained
*shaped phrase*, not to the relation concept, and relation breadth helped only
because it multiplied surface phrases. (The 14/120 is the given figure; my own
strict count for that arm on the same panels is 31/120 — see the last section.)

Prediction under test: training with **many surface forms per relation**
generalises to unseen *forms* the way 76 relations generalised to unseen
*relations*.

## Instrument

`crates/uor-r4-training/src/bin/synthetic-memory-corpus.rs` (extended on top of
#1641's 76-relation table, which is not yet on `main`; #1642 edits the same file
and will conflict textually):

* `forms=N` — surface forms per relation the generator may draw from. Form 0 is
  the relation's canonical phrase. Forms 1..7 are paraphrase question and
  statement templates over a hand-authored per-relation *topic*; the **reply
  keeps the single canonical answer form in every arm**, so the output shape is
  held fixed and only the input wording varies.
* Forms 8..12 are **held out**. `forms` is clamped to 1..=8, so the draw index
  never reaches the held-out tail and no training store can contain those forms
  (`forms=13` is refused: "forms must be 1..=8; a larger value would train on
  the held-out tail"). The panel draws them from the same table, which is what
  makes "unseen form" a property of construction rather than of inspection.
* `panel_out=DIR` writes the panel (`requests.json`, `expected.json`,
  `manifest.json` including the full value table).
* Startup validation: every relation has exactly one slot, every slot names a
  relation, and the value sets cover the table exactly.

**Combined with the distance curriculum (#1642).** The generator now carries
both knob families in one file. `document()` draws one form index per selection
site in the order the narrow generator used, so `forms=1` adds `below(1)` draws
that leave the rng stream untouched, and the filler turns sit between the
statement pair and the question exactly as #1642 wrote them. Verified against
the pre-forms generator built from `origin/main`, `rows=64 seed=17`:

| args | tokens.u16 | response_mask.u8 | vs main |
|---|---|---|---|
| `distance=0 forms=1` | `380ebdaf690283cd…` | `c53704a4026838ca…` | byte-identical to main `distance=0` |
| `distance=3 forms=1` | `4722a1f03f0caf38…` | `bd1deb4530c8853d…` | byte-identical to main `distance=3` |
| `curriculum=1 forms=1` | `38e9e39c0f15a534…` | `32ca23a45da7def1…` | byte-identical to main `curriculum=1` |
| `distance=0 forms=8` | `8132103eaa929433…` | `d966c51d54d9d59f…` | paraphrases drawn, held-out tail never |
| `curriculum=1 forms=8` | `de7c8ecd0e63883b…` | `3d40d4a5e699ed95…` | fillers and paraphrases together |

Decoding the stores confirms the content, not just the option: at `distance=3`
three filler turns sit between the fact and the question; at `forms=8` the fact
is stated as "By the way, my market is St George." and asked as "Who's my
teacher?"; at `forms=1` both are the canonical phrase. Seven focused tests cover
the two knob families together (four from #1642, three for the forms
invariants).

**Byte identity of the fixed-form path.** `forms=1` must reproduce the previous
recipe exactly. With `rows=30000 seed=17` the new binary writes

```
tokens.u16        9bae7202ca4a94552b8894739e7f001fb8967d28da79ed04e99c3bc524c703fd
response_mask.u8  ed15cb21a948c7e308609ada9f352b3d7cf079609818ee9b80274174d5f2fa41
```

which are the hashes of `wide-memory-corpus2`, the store the existing
wide-relation arm trained on. The pre-change wide generator, built in another
lane, reproduces the same two hashes independently. Only the manifest's
`synthetic` block gains fields.

## Panel design

464 rows, 74 per condition over the 74 relations that have paraphrase forms
(`garden` and `mortgage` have `have` / `do not have` values and keep the
canonical single form in every arm, so they are excluded from the form axis):

| condition | statement | question |
|---|---|---|
| `seen` | canonical (trained) | canonical (trained) |
| `seen-comp` | canonical | canonical, with a competing statement between |
| `unseen-state` | held-out form | canonical |
| `unseen-ask` | canonical | held-out form |
| `unseen` | held-out form | held-out form |
| `unseen-comp` | held-out form | held-out form, with a competing held-out statement between |
| `perturb-ask` (n=20) | canonical | canonical with **one word substituted** (contraction or spelling variant) |

The competing conditions reuse their non-competing twin's exact statement and
question, so the extra turn is the only difference. The held-out form index
varies with the relation, so all five held-out templates are exercised.

Scoring (`scripts/copyfid-score.py`) is value presence in the last turn's reply:
**strict** = the value's words in order with no word character touching either
end (so `greenish` does not count for `green`); **lenient** = case-insensitive
substring. Degeneracy is reported separately (empty, truncated, missing EOS,
repeated word 4-grams, five-word echo of the question). Wilson 95% intervals
and an exact McNemar test over shared rows are in the summary JSON.

### Tool limits, not patched

* `lut-chat` refuses a panel above 128 requests ("a request panel needs 1 to 128
  requests with distinct ids, each with 1 to 8 nonblank user turns"), so the
  panel is run as four pre-split chunks and scored as a union
  (`scripts/copyfid-split-panel.py`).
* No surviving binary carries the corpus generator as a `geometric-stack` mode
  (it is a separate `bin`), and none of the surviving `synthetic-memory-corpus`
  binaries has the wide table, so the generator was built from this worktree
  (`cargo build --release -p uor-r4-training --bin synthetic-memory-corpus`).

## Arms

Matched on init, steps, learning rate, batch, data seed, dev seed, policy and
synthetic share:

| | F0 (phrase-locked) | F1 (form-varied) |
|---|---|---|
| synthetic store | `wide-memory-corpus2`, 30,000 rows, 1,717,822 tokens, seed 17 | `copyfid-f1-corpus`, 25,991 rows, 1,723,669 tokens, seed 17 |
| forms per relation | 1 (canonical) | 8 (canonical + 7 paraphrases) |
| synthetic share of the mixed store | 2.039% | 2.046% |
| mixed store | `mixed-wide-2`, 84,247,512 tokens, 103,632 documents | `copyfid-mixed-f1`, 84,253,359 tokens, 99,623 documents |
| eligible response tokens / responses | 1,344,521 / 44,826 | 1,304,875 / 40,817 |
| init | `chat-dose-continue-1/dialogue/model` | same |
| steps / batch / lr / warmup / min_lr / clip / weight decay | 2500 / 16 / 0.0002 / 50 / 0.1 / 1.0 / 0.1 | same |
| data seed / dev seed / policy / threads | 20260929 / 20260930 / `full_prefix` / 2 | same |
| supervised target visits over the run | 1,194,790 | 1,279,849 (+7.1%) |
| train seconds / mean step | 5,160.8 / 2.06 s | 8,663.0 / 3.46 s |

Rows were scaled, not the token count: F1 spreads the *same* synthetic token
budget over eight times as many surface forms, so each individual form is seen
about 43 times against F0's ~395 repetitions of its single form. The `forms=8`
draw is index 0..7, and the held-out forms live at index 8..12, so no training
store can contain them.

F0 is the **existing** `wide-bind-1` arm, not a fresh run: its launcher
(`d19-emitter-eval/scripts/launch-wide.sh`) sets `RAYON_NUM_THREADS=2` and passes
every setting above with the same init and the same synthetic store, which is
byte-identical to what `forms=1` produces here. F1 is a new run written to match
it. The two differ in the surface-form axis and in the +7.1% supervised target
visits that follow from F1's shorter synthetic documents, which is a minor
budget imbalance in F1's favour rather than a confound against it.

A third, untrained-for-this-purpose control is scored on the same panel:
`chat-dose-continue-1/integer/model.lut`, the init both arms start from. It never
trained on the wide relations, so it sets the "no relation training" floor.

## Results

Strict = the value's words in order with no word character touching either end.
`n = 74` relations per condition except `perturb-ask` (n = 20).
`other` = the reply also contains another value of the asked relation;
`distractor` = the reply contains the competing fact's value.
Rates from `copyfid-score-forms4-final.md` / `.json`.

| arm | condition | n | strict | 95% CI | lenient | other | distractor | trunc | no_eos |
|---|---|---|---|---|---|---|---|---|---|
| F0 | seen | 74 | 74 = 100.0% | [95.1, 100] | 100.0% | 1.4% | 0.0% | 2.7% | 2.7% |
| F0 | seen-comp | 74 | 72 = 97.3% | [90.7, 99.3] | 97.3% | 1.4% | 1.4% | 1.4% | 1.4% |
| F0 | unseen-state | 74 | 28 = 37.8% | [27.6, 49.2] | 37.8% | 4.1% | 0.0% | 4.1% | 8.1% |
| F0 | unseen-ask | 74 | 43 = 58.1% | [46.7, 68.7] | 58.1% | 1.4% | 0.0% | 17.6% | 18.9% |
| F0 | unseen | 74 | 35 = 47.3% | [36.3, 58.5] | 47.3% | 2.7% | 0.0% | 31.1% | 33.8% |
| F0 | unseen-comp | 74 | 34 = 45.9% | [35.1, 57.2] | 45.9% | 1.4% | 12.2% | 13.5% | 17.6% |
| F0 | perturb-ask | 20 | 19 = 95.0% | [76.4, 99.1] | 95.0% | 0.0% | 0.0% | 5.0% | 5.0% |
| **F1** | **seen** | 74 | **73 = 98.6%** | [92.7, 99.8] | 98.6% | 1.4% | 0.0% | 5.4% | 5.4% |
| **F1** | **seen-comp** | 74 | **71 = 95.9%** | [88.7, 98.6] | 95.9% | 1.4% | 0.0% | 5.4% | 5.4% |
| **F1** | **unseen-state** | 74 | **70 = 94.6%** | [86.9, 97.9] | 94.6% | 1.4% | 0.0% | 5.4% | 5.4% |
| **F1** | **unseen-ask** | 74 | **47 = 63.5%** | [52.1, 73.6] | 63.5% | 1.4% | 0.0% | 21.6% | 21.6% |
| **F1** | **unseen** | 74 | **44 = 59.5%** | [48.1, 69.9] | 59.5% | 1.4% | 0.0% | 27.0% | 27.0% |
| **F1** | **unseen-comp** | 74 | **39 = 52.7%** | [41.5, 63.7] | 52.7% | 0.0% | 10.8% | 5.4% | 5.4% |
| **F1** | **perturb-ask** | 20 | **18 = 90.0%** | [69.9, 97.2] | 90.0% | 0.0% | 0.0% | 5.0% | 5.0% |
| dose | seen | 74 | 22 = 29.7% | [20.5, 40.9] | 31.1% | 4.1% | 0.0% | 67.6% | 71.6% |
| dose | seen-comp | 74 | 18 = 24.3% | [16.0, 35.2] | 24.3% | 1.4% | 9.5% | 52.7% | 56.8% |
| dose | unseen-state | 74 | 15 = 20.3% | [12.7, 30.8] | 20.3% | 0.0% | 0.0% | 70.3% | 75.7% |
| dose | unseen-ask | 74 | 18 = 24.3% | [16.0, 35.2] | 25.7% | 2.7% | 0.0% | 63.5% | 67.6% |
| dose | unseen | 74 | 14 = 18.9% | [11.6, 29.3] | 18.9% | 0.0% | 0.0% | 73.0% | 77.0% |
| dose | unseen-comp | 74 | 10 = 13.5% | [7.5, 23.1] | 13.5% | 1.4% | 4.1% | 70.3% | 73.0% |
| dose | perturb-ask | 20 | 11 = 55.0% | [34.2, 74.2] | 60.0% | 0.0% | 0.0% | 45.0% | 50.0% |

`empty` and `repeat` are near zero everywhere and are omitted; they are in the
summary JSON. The dose control is degenerate in a different way: 52-77% of its
replies run to the token cap without an EOS, so its 13-30% is largely the floor
probability of emitting a value that is sitting in the context.

### F0 vs F1, exact McNemar on shared rows

| condition | only F0 | only F1 | p |
|---|---|---|---|
| seen | 1 | 0 | 1 |
| seen-comp | 3 | 2 | 1 |
| **unseen-state** | **2** | **44** | **3.1e-11** |
| unseen-ask | 10 | 14 | 0.54 |
| unseen | 6 | 15 | 0.078 |
| unseen-comp | 11 | 16 | 0.44 |
| perturb-ask | 2 | 1 | 1 |

### Where the unseen-question failures sit, by held-out question template

| held-out question | F0 strict | F1 strict |
|---|---|---|
| "What was that {t} I told you about?" | 5/14 = 36% | 8/14 = 57% |
| "Have you kept a note of my {t}?" | 9/16 = 56% | 9/16 = 56% |
| "My {t} has slipped my mind. What is it?" | 9/15 = 60% | 10/15 = 67% |
| "Did I ever mention my {t} to you?" | 13/15 = 87% | 11/15 = 73% |
| "Tell me again about my {t}." | 7/14 = 50% | 9/14 = 64% |

Held-out statement templates, F0 27-53% against F1 87-100% (per-template counts
in `copyfid-analyse.py` output).

### Held-out relations (relation breadth, not form breadth)

The same two artifacts on the three held-out-relation panels, strict:

| arm | panel-40 | s5 | s11 | total |
|---|---|---|---|---|
| F0 (wide-bind-1) | 9/40 | 11/40 | 11/40 | **31/120 = 25.8%** |
| F1 (forms=8) | 11/40 | 6/40 | 5/40 | **22/120 = 18.3%** |

Per-panel exact McNemar: p = 0.79, 0.23, 0.070; pooled over the three, only F0 =
21 and only F1 = 12, p = 0.16. There is no evidence that form breadth improved
unseen-relation transfer, and the direction is if anything the other way.

### Reading

* **Surface-form breadth generalises, but only on the statement axis.** F1's
  unseen-*statement* rate is 94.6% against F0's 37.8% (p = 3.1e-11). Eight
  statement forms per relation is enough to read a ninth.
* **The question form is not fixed by the same treatment.** Both arms land at
  58.1% and 63.5% when the question is paraphrased (p = 0.54), even though F1
  trained on seven paraphrase question forms per relation. With both sides
  paraphrased, 47.3% -> 59.5% (p = 0.078, not resolved). The residual failure is
  request-form-specific, which is the direction of the phrase-lock-in hypothesis.
* **The lock is a frame lock, not a string lock.** A one-word substitution of the
  trained question costs almost nothing: 19/20 (F0) and 18/20 (F1). The
  `color` -> `colour` anecdote does not reproduce on this table; what breaks
  binding is a change of sentence *shape*, not of spelling.
* **Competition is a small tax here.** On trained phrasing 100% -> 97.3% (F0) and
  98.6% -> 95.9% (F1); on paraphrased rows 47.3% -> 45.9% and 59.5% -> 52.7%.
  When the competing fact is taken, it is taken whole: 12.2% (F0) and 10.8% (F1)
  of unseen-comp replies carry the distractor's value, and never alongside the
  asked value.
* **Form breadth is not free at fixed token budget.** F1's unseen-relation rate
  is lower (22/120 vs 31/120, p = 0.16 pooled, no resolved difference) and its
  trained-form rate is 1.4 pp lower, consistent with trading per-form repetition
  (395 -> ~43 exposures) for form diversity. Whether a larger synthetic budget
  buys both is not measured here.

## What could not be resolved

* **No seed replicate.** One run per arm; the confidence intervals treat panel
  rows as independent, but the 74 rows per condition are six conditions over 76
  relations and therefore cluster by relation, so the intervals are optimistic
  for relation-level effects. The 3.1e-11 statement-axis result is far outside
  that concern; the 0.078 and 0.54 question-axis results are inside it.
* **Frame-homogeneous statement variety.** All seven trainable statement forms
  and all five held-out ones are the same core frame, "my {t} is {v}", under a
  different discourse prefix. F1's 94.6% therefore measures generalisation across
  prefixes of a learned frame, not paraphrase-invariance in general. A held-out
  statement set with different syntax was not built.
* **Topic-level restatement.** For the ~40 relations whose canonical sentence
  carries a verb or preposition ("Which city do I want to visit?", "I live at
  number {v}"), the paraphrase frames restate the fact at topic level ("my city
  is {v}") rather than being strict paraphrases. This is a treatment difference
  beyond surface form and is not quantified per relation.
* **The `seen` cell cannot separate reading from weights.** Every value and both
  the statement and question phrases of the `seen` cell were in training, so
  100% there is consistent with weight-level phrase -> relation association as
  much as with in-context reading. The unseen conditions are the measurement.
* **No novel values.** Every value in the panel was seen in training for its
  relation; the arms are never asked for a value they have not been fitted on.
* **The 14/120 figure for the same arm and panels is not reproduced.** My strict
  count on the three held-out-relation panels is 31/120 (lenient 32, <=12 words
  29, <=8 words 22, exact reply 0), all measured on `wide-bind-1/integer`. I
  could not find a defensible criterion that gives 14/120, so I report mine and
  flag the difference rather than reconciling it.
* **Decoding is greedy only.** `lut-chat` supports `temperature`, `top_k` and
  `top_p`, but no sampled run was made, so nothing here speaks to whether the
  residual question-form failures are a decoding artefact. This is a stated gap,
  not a claim that they are not.
* **One token budget.** 2% synthetic share at 2500 steps; the F0 comparison is
  the existing arm rather than a fresh matched run, and no arm was run at a
  longer dose.

## Artifacts

| artifact | sha256 (first 16) |
|---|---|
| `wide-bind-1/integer/model.lut` (F0) | `e16ed660f7d5aa19` |
| `copyfid-f1-arm/integer/model.lut` (F1) | `7805dc500793279f` |
| `wide-bind-1/model` (F0 float, sealed) | `dc0951d3bdc89a3c` |
| `copyfid-f1-arm/model` (F1 float, sealed) | `36ce614cab540e5e` |
| `copyfid-panel-forms4/requests.json` | `f777fa4b1176bbc2` |
| `copyfid-f1-corpus/tokens.u16` | `d6591e4ca87be089` |
| `copyfid-mixed-f1/tokens.u16` | `2e0f0bb37daa254a` |

Everything is under `~/uor-r4-worktrees/reports/`; the per-condition summary is
`copyfid-score-forms4-final.{md,json}`, the relation-panel per-arm summaries are
`copyfid-score-f1-heldout-{40,s5,s11}.json`, and the raw replies are in the
`copyfid-eval-*` roots. Resource cost: F1 training 8,663 s at 2 threads, peak
RSS ~0.6 GB, 32 MB retained for the arm and 241 MB for the mixed store;
generation about 2 minutes; the twelve `lut-chat` evaluations about 4 minutes.


