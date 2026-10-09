# Span extraction on the pointer panel — October 9

Span extraction, tested prospectively on the 84-cell pointer panel, **does not recover the five
class-C cells in its natural reading**: the pointer re-acquires the premise one id before the value,
at the copula `" is"`, so the span it locked onto is `" is Ithmar."`, which does not normalise to the
value. The headline instrument fires on exactly those five cells at every floor and gains **0**:
**72/84 at F=0.5, 72/84 at F=0.75, 71/84 at F=1.0**, identical to the baselines, 0 gained / 0 lost.
A **fitted** variant that drops the re-acquisition anchor measures **77/84 = 0.9167** (+5/0,
p=0.0625, Newcombe [−0.0391, +0.1594]) — a better point estimate, **not** an improvement — and at
F=0.75/1.0 it is +4/0 with one template firing per floor. The accepted best therefore remains the
parent result, arm-ptr with the token-identity copy stop at floor 0.5: **72/84 = 0.8571**. No new
learned model, no training, no serving result; this closes the "+5/84 ceiling" that
[the class-C classification](../../evidence/idstop_nonfiring_classification_2026-10-09.txt) left
open. References #2029, #2032.

## What was pre-registered, and what it fixed

[The pre-registration](../../evidence/span-extraction-preregistration-2026-10-09.md) was frozen and
sent before any run (`sha256 55469a43…`, then a dated addendum `27a84aa0…`): the exact rule, the
flags, the acceptance criteria A1–A9, the floors, the controls, and a statement of the
pre-registration's own weaknesses. Criteria that did the work: **A4** requires a +5/0 result to be
reported as p=0.0625, a better point estimate and NOT an improvement; **A8** makes zero gain, any
damage or any template firing a decisive negative that stops the round with no re-tuning; **A5**
counts firings whose span is disjoint from the row's recorded value span and calls them a FAILURE.

The reconnaissance that shaped the rule is itself a measured result and it is why this round is a
negative rather than a hopeful positive. From the sealed idstop3 traces (laptop CPU, no GPU): in all
five class-C cells the pointer reads a *different* occurrence of `" name"` (window position
28/29/30, in the question) at step 1 and only **re-acquires** the premise at step 2, window[6] —
the copula `" is"` — then sweeps one window position per step to window[11]; the value's span starts
at 7. A structural scan of all 84 traces found **exactly five cells with a mid-reply pointer-locked
run of at least two ids, and they are exactly the five class-C cells**, so a mid-reply-only rule
provably cannot touch the other 79. The runs confirm it: modes 1 and 2 fire on exactly those five at
all three floors.

## Measured

Rule (read-out only, default off, nothing serialized): a step is *locked* when the emitted id is the
window id at the pointer's own selected source; a *pointer-locked run* is a maximal sequence of
locked steps whose source advances by one each step. Modes: **1** mid-reply runs, anchor kept
(**headline**, the natural reading); **2** mid-reply runs, anchor dropped (**fitted variant**);
**3** any run, anchor kept (**broad control**). When a run qualifies the reply becomes exactly its
span; otherwise the reply is unchanged field for field and, with the rule off, no field is added.

| configuration | baseline | exact | gained | lost | fired | template-disjoint firings |
|---|---:|---:|---:|---:|---:|---:|
| mode 1 (headline), F=0.5 | 72/84 | 72/84 | 0 | 0 | 5/84 | **0** |
| mode 1 (headline), F=0.75 | 72/84 | 72/84 | 0 | 0 | 5/84 | **1** (natural/n_drovik) |
| mode 1 (headline), F=1.0 | 71/84 | 71/84 | 0 | 0 | 5/84 | **1** (natural/n_drovik) |
| mode 2 (**fitted**) F=0.5 | 72/84 | **77/84 = 0.9167** | 5 | 0 | 5/84 | 0 |
| mode 2 (**fitted**) F=0.75 | 72/84 | 76/84 = 0.9048 | 4 | 0 | 5/84 | 1 |
| mode 2 (**fitted**) F=1.0 | 71/84 | 75/84 = 0.8929 | 4 | 0 | 5/84 | 1 |
| mode 3 (broad control) F=0.5 | 72/84 | 70/84 = 0.8333 | 0 | **2** | 82/84 | **3** |

The headline's five firings, floor 0.5, all still not exact: `natural/n_ithmar`, `sure/n_ithmar`,
`noted/n_ithmar` `' My name is Ithmar.' -> ' is Ithmar.'`; `sure/n_iyandel`
`' My name is Iyandel.' -> ' is Iyandel.'`; `sure/n_orvynn` `' My name is Orvynn.' -> ' is Orvynn.'`
(spans window[6..11], and window[6..12] for orvynn). At F=0.75/1.0 the same five fire and one of
them — `natural/n_drovik`, on a run-on reply `' Drovik.\nAssist'` — is a **template firing**
(`win[23..25] -> '\nAss'`), which A5 reports as a FAILURE.

The **fitted** variant (every number in this paragraph is fitted, and the word must travel with it):
at F=0.5, Wilson [0.8378, 0.9590], +5 = +6.0 %, exact McNemar 5 vs 0 **p=0.0625**, Newcombe
[−0.0391, +0.1594] **contains zero**; the five gained cells are the five class-C cells, each going
`' My name is X.' -> ' X.'`, zero lost. At F=0.75/1.0 it is +4/0 (sure/n_orvynn is not gained
there) with one template firing per floor. It improves the point estimate at all three floors with
no losses and no significance, and it does not generalise cleanly. It shows a read-out *could* in
principle recover these cells; it is not evidence that span extraction works.

The broad control shows the mid-reply gate is load-bearing, measured: without it the rule fires on
82/84 and truncates five currently-exact cells — `natural/n_candrake` `' Candrake.' -> ' Candr'` and
`natural/n_kelbrin` `' Kelbrin.' -> ' Kelbr'` become **lost**, while `natural/n_pellum`,
`noted/n_harrowen` and `noted/n_pellum` keep normalising to the value — and produces three
template-disjoint firings (`natural/n_therrick` win[26..27] `" What is"`, `noted/n_iyandel` and
`noted/n_therrick` win[18..21] `" Noted."`).

## Evidence discipline

- **A1 default proven, not asserted:** 9 files / 288 records at floor 0 with every rule off are
  field-for-field identical to the sealed references (`scripts/idstop-compare-records.py`,
  SEALED-COMPARE PASS, 0 differing fields, 0 missing, 0 added records).
- **A2 parser check first:** `scripts/idstop-score.py check` → PARSER-CHECK PASS, arm-ptr 35/84 with
  channels 0.5149/0.8351/0.5610/0.4901, arm-a-w05 10/84, arm-c-ctrl 24/84, before any new number.
- **A3 implementation witness at the baseline:** the rule-off run at the 72/84 configuration
  reproduces the sealed idstop3 trace run field for field (3 files / 96 records, 0 differing fields).
- **Replay witness:** the offline replay of the rule over each record's own `reply_trace`
  (`scripts/idstop-span-extract.py replay`) reproduces the binary's span decision **84/84** and its
  reply ids **84/84** in every measured configuration; the cross-checks (a different rule replayed
  on the same file) disagree as they must, so the check discriminates.
- **Focused tests:** three new tests pass (`cargo test -p uor-r4-training --lib span_extract`). They
  caught a real bug — the first build dropped a locked run that ended on a non-locked step, so the
  first pod run fired 0/84 on a trace that plainly has a mid-reply run; every number here is from
  the fixed binary.

## Artifacts, cost and next

Pod `ajrklwvw9uwvzr` (EU-RO-1, 2×4090, $1.78/h, ~26 min ≈ $0.80; volume pinned with
`UOR_POD_VOLUME_DCS=EU-RO-1` as `rfsx702p68`), released and deleted after archival. 33 generation
runs: rule off (3), mode 1 at F=0.5/0.75/1.0 (9), mode 2 at F=0.5/0.75/1.0 (9, the 0.75/1.0 part
added after the freeze and labelled exploratory), mode 3 (3), default proof (9). Binary sha256
`f8bb8c47…`. Warm-target build 26 s; the 27-run pre-registered set 42 s. Runs and binary on the
volume under `bindprobe/span-extract/`, stored to iCloud as `span-extract` (28,852,736 bytes, md5
`e2692d9d…`); scoring on the laptop CPU.

**Next, and the honest boundary:** do not re-tune `pointer_span_min`, the anchor convention or the
mid-reply gate on this panel — A8 forbids it and the population is five cells. The natural reading is
closed as a negative. A further read-out instrument would need an *independent* marker of where the
value begins (none was found in the pointer's own selection here: attention, floored gate and raw
gate at the re-acquisition step do not separate it from the value's first token in all five cells),
and the four class-A cells remain a generator/EOS failure no serving-time read-out can touch. The
line remains offline only: export still refuses a pointer carrying an identity term, so there is no
integer port and none of this is a serving result.
