# Span extraction on the pointer panel — PRE-REGISTRATION (frozen before any new measurement)

Date: 2026-10-09. DeepSeek lab, session `span-extract`. References #2029 (language
base) / #2032 (served model); claims go on #2029 or #2032. Builds on
`docs/evidence/token_identity_stop_breakthrough_2026-10-09.txt` (72/84) and
`docs/evidence/idstop_nonfiring_classification_2026-10-09.txt` (the class-C limit).

Status at freeze time: **no new GPU run and no new measurement has been made.**
Everything in the RECONNAISSANCE section below is read off already-sealed trace
files on the local mirror (`/tmp/idstop3`, the idstop3 round's own data), is
labelled PREDICTION, and is not a result.

---

## 1. RECONNAISSANCE THAT DETERMINES THE RULE (free, laptop CPU, no GPU)

Run: `python3 /tmp/span_scan.py`, `python3 /tmp/span_runs.py` over
`/tmp/idstop3/trace/trace-arm-ptr-f0.5-ident-{natural,sure,noted}.json`.
Mandated parser self-check first: `python3 scripts/idstop-score.py --work /tmp/idstop3
--rate /tmp/idstop3/sealed/bindprobe/rate --ptrident /tmp/idstop3/sealed/ptrident
--round /tmp/idstop3 check` → `PARSER-CHECK PASS`, arm-ptr 35/84 channels
0.5149/0.8351/0.5610/0.4901, arm-a-w05 10/84, arm-c-ctrl 24/84.

Measured structure of the 5 class-C cells (all five, every step, from the sealed traces):

| cell | reply | reply == window[a..b] | pointer source per step | value span |
|---|---|---|---|---|
| natural/n_ithmar | ` My name is Ithmar.` | [4..11] | 4, 28, 6, 7, 8, 9, 10, 11 | [7,8,9,10] |
| sure/n_ithmar | ` My name is Ithmar.` | [4..11] | 4, 29, 6, 7, 8, 9, 10, 11 | [7,8,9,10] |
| noted/n_ithmar | ` My name is Ithmar.` | [4..11] | 4, 30, 6, 7, 8, 9, 10, 11 | [7,8,9,10] |
| sure/n_iyandel | ` My name is Iyandel.` | [4..11] | 4, 29, 6, 7, 8, 9, 10, 11 | [7,8,9,10] |
| sure/n_orvynn | ` My name is Orvynn.` | [4..12] | 7, 30, 6, 7, 8, 9, 10, 11, 12 | [7,8,9,10,11] |

The reply's first three ids are the template `" My name is"` (window[4..6]); the
value is window[7..10/11]. The pointer is **not** locked on the template: at step 1
it reads a different occurrence of the same id (position 28/29/30, in the question
`"What is my name?"`), and it only *re-acquires* the premise at step 2 (position 6,
`" is"`), then sweeps forward one window position per step.

Definitions used from here (all computed from channels the driver already records):

* a step is **locked** when `matches_source` is true (`emitted id == window[pointer source]`);
* a **pointer-locked run** is a maximal sequence of consecutive locked steps whose
  source advances by exactly one each step;
* a run **opens at the reply start** when its first step is step 0, otherwise **mid-reply**;
* the run's **span** is `window[source_first..source_last]`, its **post-anchor span** is
  `window[source_first+1..source_last]`.

Structural scan of all 84 cells (`/tmp/span_runs.py`):

* **Exactly 5 of 84 cells have a mid-reply pointer-locked run of length >= 2, and they
  are exactly the 5 class-C cells** (`natural/n_ithmar`, `sure/n_ithmar`,
  `noted/n_ithmar`, `sure/n_iyandel`, `sure/n_orvynn`). Their runs are window[6..11]
  (four cells) and window[6..12] (orvynn); post-anchor spans window[7..11] /
  window[7..12] = the value plus the sentence-final period.
* No currently-exact cell has a mid-reply locked run of length >= 2. Four exact cells
  have short mid-reply runs of length 1 (natural/n_candrake s3 win[21],
  natural/n_kelbrin s3 win[21], noted/n_harrowen s4 win[21], noted/n_pellum s3 win[20]).
* 5 exact cells have a reply-start run that is shorter than the reply:
  natural/n_candrake and natural/n_kelbrin (run [7..9], reply [7..11]),
  natural/n_pellum (run [7..9], reply [7..9]+1), noted/n_harrowen (run [7..10],
  reply [7..11]), noted/n_pellum (run [7..9], reply [7..11]).

**PREDICTION (not measured):** the natural reading of "emit only the span the pointer
selected" — replace the reply by the longest pointer-locked run — cannot reach the
value on any of the 5 cells, because the locked run starts one id early at window[6]
(`" is"`), so it emits `" is Ithmar."`, which does not normalise to the value. Applied
without the mid-reply restriction it would also truncate the 5 exact cells listed
above. The only convention that reaches the value is to drop the *anchor* id at which
the pointer re-acquires the region (the run's first id). That convention is fitted to
these 5 cells and is stated as such. Floors 0.75 / 1.0 are the pre-registered
generalisation check for it.

---

## 2. THE FROZEN INSTRUMENT

Read-out only. No training, no parameters, nothing serialized. Same place as the
serving-time copy stop (`StackModel::set_pointer_copy_stop`), and off by default.

### Flags

* `pointer_span_extract=0|1|2` (default **0**): `0` off; `1` **primary** — act only on
  **mid-reply** locked runs; `2` **broad control** — act on any locked run, including
  reply-start runs.
* `pointer_span_min=N` (default **2**): minimum length in ids of the **post-anchor**
  span. `N < 1`, or no qualifying run, means the rule does not fire and the reply is
  left exactly as the copy stop / EOS / short cycle / `max_new_tokens` left it.

### Rule (mode 1, primary)

At the end of a reply, walk the reply's steps. Among all pointer-locked runs that
**open mid-reply** and whose post-anchor span has at least `pointer_span_min` ids,
take the longest post-anchor span (ties: the earliest run). If one exists:

* the reply's ids become exactly `window[source_first+1 ..= source_last]`
  (everything before the run and everything after it is dropped);
* `reply_eos = false`; `reply_stop = {"pointer_span": {"extracted": n, "window_index": source_first+1}}`;
  `reply_stopped_at` = the run's last step; a record field `span_extract` is added with
  `{"window_index", "anchor", "extracted", "run_start_step", "dropped_prefix"}`.

If none exists the reply is **unchanged**, and no `span_extract` key is written.

Mode 2 is the same rule without the "opens mid-reply" requirement; it is the
pre-registered control that shows why that requirement is there.

Edge cases, frozen: a step with no pointer selection is not locked; the EOS id is an
ordinary emitted id for locking purposes; if the post-anchor span is empty or shorter
than `pointer_span_min` the rule does not fire; when it fires, the trailing period (or
any other id after the run's last position) is kept only if it is inside the span;
`pointer_span_extract=0` must add no record field and take no extra observation.

### What the rule is NOT allowed to be

It may not consult the row's `expected`/`forbid`, the recorded `span_positions`, the
record `id`, or any label. It sees only: the emitted ids, the window ids, and the
pointer's own per-step selection. The recorded `span_positions.expected` is used
**only by the scorer**, to count whether a firing reproduced the template.

---

## 3. RUN AND MEASUREMENT PLAN (after the go)

Same panel and protocol as the result it extends: 28 invented values x 3 conditions =
84 cells per configuration, arm-ptr checkpoint, protocol 2, greedy,
`max_new_tokens=40`, `device=cuda`, one seed, one session, one binary. Pod pinned to
the EU-RO-1 volume (`UOR_POD_VOLUME_DCS=EU-RO-1`), volume `rfsx702p68`
(`uor-shared-EU-RO-1`); CPU scoring on the laptop. Runs:

1. mode 1 at floors 0.5 / 0.75 / 1.0, with the identity copy stop exactly as in the
   72/84 configuration (`pointer_copy_stop=2 pointer_copy_stop_identity=1 copy_trace=1`): 9 runs.
2. mode 2 at floor 0.5 (control): 3 runs.
3. rule off at the 72/84 configuration (`copy_trace=1 pointer_span_extract=0`) to prove
   the new binary reproduces the sealed idstop3 trace run: 3 runs.
4. default proof (`pointer_gate_floor=0 pointer_copy_stop=0 copy_trace=0
   pointer_span_extract=0`) on all three arms: 9 runs.
5. offline (laptop, from the recorded traces of 1 and 3): the keep-anchor variant of
   mode 1, replayed and validated against the recorded decisions.

Scorer scripts: `scripts/idstop-score.py`, `idstop-crosschecks.py`,
`idstop-turn1check.py`, `idstop-compare-records.py` are reused **unchanged**. One new
script `scripts/idstop-span-extract.py` holds the span/template classification and the
offline replay; if any of the four had to change, the change and its reason are reported.

---

## 4. ACCEPTANCE CRITERIA (frozen)

* **A1 default unchanged.** Rule off: all 9 default files (288 records) field-for-field
  identical to the sealed references (`scripts/idstop-compare-records.py`,
  `SEALED-COMPARE PASS`, 0 differing fields), **plus** a strict both-ways comparison
  showing 0 added fields, so "bit-for-bit unchanged" is not just "no missing field".
* **A2 parser first.** `scripts/idstop-score.py check` prints `PARSER-CHECK PASS` with
  arm-ptr 35/84 and channels 0.5149/0.8351/0.5610/0.4901, arm-a-w05 10/84,
  arm-c-ctrl 24/84, **before any new number is believed**.
* **A3 implementation witness.** Rule off at the 72/84 configuration reproduces the
  sealed idstop3 trace run: same replies, same `copy_stop` decisions, 2x2 20/3/52/9, 72/84.
* **A4 success (mode 1, floor 0.5).** Strictly beating 72/84 **with zero damage**: no
  cell may go exact -> not-exact, and every changed cell is listed by name with its
  before/after reply. Gained/lost counts, exact count, Wilson interval, exact McNemar
  (`binom_two_sided`) and Newcombe 95 % vs 72/84 are all reported. Honest bar: +5/0 is
  p = 0.0625 (not significant at 95 %); such a result is reported as **a better point
  estimate, NOT as an improvement**. Only >= 6 net gained with 0 lost can be
  significant here.
* **A5 template firing.** For every firing, the emitted span is checked against the row's
  recorded `span_positions.expected`: report (i) firings whose span is disjoint from the
  value span — i.e. it reproduced the **template** or the question — and (ii) firings
  fully inside the value span. **Any count > 0 in (i) is a FAILURE and is reported as
  such**, with the cells named.
* **A6 controls reported separately.** Mode 2, and the offline keep-anchor variant, are
  reported as controls with their own gained/lost cells; they are never merged into the
  headline number.
* **A7 generalisation gate.** Mode 1 at floors 0.75 and 1.0 must not fall below the
  sealed identity-stop baselines 72/84 and 71/84. Any gain there is reported as
  secondary evidence; any loss is a negative.
* **A8 negative is a result.** If the rule fires on zero cells, gains zero, damages any
  cell, or fires on the template, that is reported clearly and the round stops; no
  re-tuning of `pointer_span_min` or of the anchor convention inside the same round.
* **A9 measurement vs inference.** Measured and inferred statements are separated in the
  report; every command, count, error and hashed artifact is listed.

## 5. KNOWN WEAKNESSES OF THIS PRE-REGISTRATION (stated before the run)

1. The anchor-drop convention is fitted to the same 5 cells it is meant to recover; the
   keep-anchor control and floors 0.75/1.0 are the only pre-registered checks that can
   distinguish "mechanism" from "panel fit".
2. The mid-reply restriction is structural (it is what keeps the rule off the 72 exact
   cells) but it is also exactly the boundary that isolates these 5 cells on this panel.
3. The panel has 28 values; +5 is below the significance bar at 84 cells, so the
   strongest honest claim available from this round is a better point estimate plus a
   decisive statement about the natural reading.

---

## ADDENDUM (appended after the go, before any run of the instrument)

1. **The anchor-drop convention was chosen AFTER seeing the sealed traces.** The frozen text
   above was sent to the team lead at 13:07 EDT on 2026-10-09 (this file's sha256 at that
   moment: `55469a43069a49a57b3ec83681e3a814b6f5a63f7dc77fc32f4fdc29b3cbb358`). The traces
   were inspected on the laptop, with no GPU, between 12:58 and 13:03 EDT: `/tmp/span_scan.py`
   (12:58:31), `/tmp/span_detail.py` (12:58:58), `/tmp/span_runs.py` (13:00:31) over the sealed
   idstop3 trace files. The anchor-drop convention is therefore **fitted**: it was chosen
   because the sealed traces show the value's first token is the id AFTER the re-acquisition
   anchor in all five class-C cells, and no other convention reached the value. Every sentence,
   table row and summary line that carries its number must carry the word **fitted**.
2. **Numbering follows the team lead's decision of 13:09 EDT**: **mode 1 = mid-reply runs with
   the anchor KEPT is the HEADLINE** (the honest test of the hypothesis as stated, reported
   whatever it says); **mode 2 = mid-reply runs with the anchor DROPPED is the FITTED VARIANT**;
   **mode 3 = any run with the anchor kept is the BROAD CONTROL**. This renumbering changes
   which variant is the headline, not the rule, the criteria or the run set.
3. **A8 damage accounting, by name, for the broad control** (the five currently-exact cells
   whose reply-start run is shorter than the reply, so mode 3 truncates them):
   `natural/n_candrake` (' Candrake.' -> ' Candr'), `natural/n_kelbrin` (' Kelbrin.' ->
   ' Kelbr'), `natural/n_pellum` (' Pellum.' -> ' Pellum'), `noted/n_harrowen` (' Harrowen.' ->
   ' Harrowen'), `noted/n_pellum` (' Pellum.' -> ' Pellum'). Measured in the result file:
   the first two become **lost** cells, the last three stay exact.
4. **One addition after the freeze, labelled exploratory**: mode 2 at floors 0.75 and 1.0 (6
   runs). The frozen A7 gate named only mode 1 at those floors; the fitted variant's single
   +5/84 number would otherwise rest on one floor. It is reported as exploratory, in full,
   including its one template firing per floor.
