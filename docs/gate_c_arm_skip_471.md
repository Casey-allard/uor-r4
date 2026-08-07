# Sampled Gate C runs pay for a whole-corpus pass they never read (#471)

Record for issue #471. Measured 2026-08-07 on the committed 500k fixture
(`crates/uor-r4-core/tests/fixtures`, 500,000 records / 2,507 stories,
100,306 held out), two cores, `R4_GATE_C_SAMPLE=10000`.

## The complaint

#467 made Gate C samplable: set `R4_GATE_C_SAMPLE=n` and the evaluation scores
`n` held-out positions instead of the whole split. On the #460 STAGES=5
decision run at 2.11M records that knob did not buy what it promised — the
Gate C phase still ran for about eighty-five minutes. The sample bounds the
per-position arm loop and nothing before it.

## The instrument this issue produced first

`evaluate_gate_c` now prints a per-phase wall clock to stderr. It costs
nothing, it is not report content (a duration would make `score_report.json`
differ between two runs of identical pinned inputs, which is what the
deterministic-rebuild gate exists to catch), and it turns "the Gate C phase
took eighty-five minutes" from an unattributable number into an attributed
one. That mattered immediately: the first two proposals for fixing this issue
named different passes as the culprit, and neither was checked against a
measurement.

Control run, 500k, sample 10,000:

| phase | seconds | share |
|---|---:|---:|
| scorer construction | 2.60 | 3.0% |
| forward-anchor table (#399 M2) | 0.08 | 0.1% |
| **right-context code pass (#446)** | **51.05** | **59.0%** |
| left code pass (#469 lever A) | 0.09 | 0.1% |
| two-sided table build (#446 M1) | 0.27 | 0.3% |
| latent right-context tables (#446 M2) | 0.38 | 0.4% |
| unigram null (#390) | 0.02 | 0.0% |
| scoring 10,000 positions | 31.70 | 36.6% |
| reduction, rollups, replay probes | 0.38 | 0.4% |
| **total** | **86.56** | |

## What the profile revised

The issue named three things: `derive_right_codes`, `TwoSidedTable::build`
and `LatentRightTable::build`. Two of the three are not the problem. The two
table builds together are **0.65s of 86.56s — 0.8%**. All of the cost is the
right-context code pass, which is a whole-corpus `bundle_window_plain` +
`assign_code_for_bundle_with`.

Two further passes that look like the same structural complaint turn out not
to matter either, and both are worth writing down so nobody re-proposes them:

- The **left** code pass costs 0.09s, not because it is cheap but because the
  #469 lever-A sidecar has already been written by the store build earlier in
  the same process. It is paid once per run, by someone else, before Gate C
  starts.
- The **forward-anchor** table (#399 M2) is 0.08s. There is no case for a
  `forward_anchor` arm group.

## The knob

`R4_GATE_C_SKIP_ARMS`, comma-separated **arm-group names**. Unset is
behaviour-identical to the pre-#471 evaluation, in the same contract style as
`R4_GATE_C_SAMPLE`. An unrecognised name panics: a typo that silently
evaluated everything hands back an eighty-five-minute run when the caller
asked for a thirty-second one, and a typo that silently skipped nothing looks
exactly like a knob that does not work. Both failures are invisible.

One group ships. `right_context` is the #446 M1 two-sided family and the #446
M2 latent family together, because they are exactly the closure of the
right-context code pass — nothing else in the evaluation reads a
right-context code, so skipping the group and skipping the pass are the same
act. Grouping arms by *what they cost* rather than by issue number is what
keeps the knob from becoming a list of ad-hoc flags as arms accumulate.

    R4_GATE_C_SAMPLE=10000 R4_GATE_C_SKIP_ARMS=right_context \
      cargo run --release --bin r4 -- transformerless score …

## Why not the κ-keyed sidecar (option a)

The alternative on the issue was to extend the #469 lever-A sidecar to the
right-context codes: cache the pass instead of skipping it, and keep all five
rows. The measurement argues against it, and not on effort grounds.

A sidecar is keyed on the artifact κ and the corpus κ. It hits when both are
unchanged. But a *decision* run is, almost by definition, a run that changed
one of them — STAGES=5, a codebook-fit change, a capacity change, a different
corpus. Those are the runs the knob exists to make cheap, and they are
precisely the runs on which a κ-keyed cache misses. Option (a) would have
bought the second run of an unchanged configuration, which is the case nobody
was waiting on.

It is also worth recording that option (a) would have been *sufficient* in
one narrow sense the issue did not anticipate: since the table builds turn out
to cost 0.8%, caching the code pass and skipping it are within a percent of
each other in effect. The reason to prefer skipping is entirely about which
runs get the benefit.

## Absence is not zero, and the test caught a leak

The whole risk in skipping arms is what the skipped rows then say. This
repository found five instruments in a single session that could not fail, and
wrote down the rule that an all-zero result across every arm is a harness bug
until proven otherwise. A skip that left `rule12_latent_mix.top1_agreement:
0.0` in `score_report.json` manufactures exactly that reading — and
`latent_exit_rule_met: false` is worse, because a pre-declared exit rule
reported as unmet is a *result*, and this one would never have been run.

So the twenty-six #446 M1/M2 fields moved from the top level of `gate_c` into
`gate_c.right_context_arms`, which is `null` on a skipped run;
`gate_c.skipped_arm_groups` names what was skipped; and the CLI prints the
five rows as `SKIPPED` in their own places, with the M2 exit-rule verdict line
suppressed entirely rather than printed as NOT MET. Schema 25 → 26. The move
does not redefine anything: with the override unset, every one of those rows
holds the value schema 25 gave it, one level deeper.

The equivalence test then earned its keep on the first run. Two of the
twenty-six fields were ones this record's author had missed —
`win_loss.twosided_vs_rule12_live` and `win_loss.twosided_shuffled_vs_rule12_live`
were still sitting flat in the win/loss cross-tab block, and a skipped run
published them as `{both_correct: 0, neither: 0, other_only: 0, scorer_only:
0}` next to five populated cross-tabs. That is the vacuous-instrument pattern
reproduced in miniature, in a change written specifically to avoid it, and
found by a machine rather than by a reader. They now live in the arm group
with the rest.

## Result

| | control | `right_context` skipped |
|---|---:|---:|
| right-context code pass | 51.05s | 0.00s |
| two-sided + latent builds | 0.65s | 0.00s |
| scoring 10,000 positions | 31.70s | 29.12s |
| **Gate C total** | **86.56s** | **32.14s** |
| whole `score` pipeline | 138.1s | 82.8s |

**62.9% of the Gate C phase**, 40.1% of the whole scoring pipeline. Forty-five
`gate_c` keys were compared between the two runs and every one is identical,
as are the compile inputs and the emitted graph — the skip does not reach
outside the evaluation. `tests/gate_c_arm_skip.rs` is that comparison, and it
is also the instrument that produced this table.

Separately, and stronger than the in-run comparison: a schema-25 report
produced *before* any of this change was written and a schema-26 control
report produced after it agree on **every value of every `gate_c` key**, once
the twenty-six moved fields are read back out of `right_context_arms` and the
two moved cross-tabs out of `win_loss`. The key sets are equal with nothing
missing and nothing added. The restructure is a move, and the unset knob is
the old evaluation.

## Scaling, and an honest gap

The skipped pass is `O(n)`; the retained scoring is bounded by the sample.
Measured at two corpus sizes with the sample held at 10,000:

| records | right-context pass | µs/record | two-sided + latent | scoring 10,000 | Gate C total |
|---:|---:|---:|---:|---:|---:|
| 250,000 | 26.42s | 105.7 | 0.23s | 26.94s | 54.71s |
| 500,000 | 51.05s | 102.1 | 0.65s | 31.70s | 86.56s |

The pass is linear to within 3.5% per record over a 2× range. The scored
population is fixed but its cost is not quite constant — it grew 18% for 2×
the records, because a larger store makes each position's scoring a little
heavier. So the saving *fraction* rises with corpus size: 51.3% of Gate C at
250k, 62.9% at 500k.

**Where the knob does not pay, which is the same arithmetic read backwards.**
A census run over the whole held-out split (100,306 positions, same 500k
corpus) spends 265s of its 322s scoring positions. The right-context pass is
then about 16%, not 59%. This is a *decision-run* lever by construction: the
skipped cost is fixed in `n` while the retained cost scales with the sample,
so the smaller the sample the more the knob is worth. Anyone reaching for it
on a census run should expect roughly a sixth, not two thirds.

**What this does not explain — stated plainly, because the temptation is to
round it off.** Extrapolating 102 µs/record to 2.11M records gives about 215s
of right-context code pass on this two-core box: roughly four minutes, not
eighty-five. **The linear model does not reproduce the pathology that opened
this issue.** Everything above is a real saving on a real pass, and the
eighty-five minutes remains unaccounted for.

Peak resident set, same two runs (sampled every 500ms from `VmHWM`, whole
`score` process):

| | control | skipped | delta |
|---|---:|---:|---:|
| peak RSS | 1,382 MB | 1,305 MB | 77 MB (5.6%) |

That was the leading candidate before it was measured, and the measurement
weakens it. Seventy-seven megabytes at 500k scales to a few hundred at 2.11M
even allowing STAGES=5 its extra levels — real, but not on its own the
difference between four minutes and eighty-five on a 16GB machine. Recording
it as a negative rather than leaving it as an untested suspicion.

What is left:

1. **The eighty-five minutes may not all have been inside
   `evaluate_gate_c`.** Nothing at the time could tell the Gate C evaluation
   apart from the cover induction, store build and emission stages around it.
   This is now the leading candidate precisely because it is the one nobody
   could rule out.
2. **Per-record cost at 2.11M may simply not be 102 µs.** The #469 record has
   a whole-corpus code pass at 2.11M costing 625s pre-lever-B, which is ~11×
   more core-time per record than measured here. Either that figure covers
   more than the code pass, or per-record cost depends on something that a 2×
   corpus range does not expose. Both are worth knowing and neither is settled
   by this record.
3. **STAGES=5**, which the issue raised. Worth 1.25× on stage count alone, so
   not sufficient by itself.

This is why the phase timing shipped as a permanent, always-on part of the
harness rather than as scaffolding for this issue. The next 2.11M run
attributes its own wall clock, and whichever of the three it is arrives as a
line of output instead of an argument. Until then this record claims exactly
one thing: the right-context code pass is 59% of a sampled Gate C run at
500k, it is linear in `n`, and skipping it is free of any effect on the rows
that remain.

## Follow-ups worth an owner

- **Attribute the 2.11M residual.** Run the phase timing on a real 2.11M
  corpus and read the gap between 215s of predicted code pass and whatever the
  total turns out to be. Cheap, and it either closes this record or reopens it
  against a different pass. **This is the follow-up that matters** — #471
  removed a measured 59% of a sampled run, but the number that provoked the
  issue is still not explained, and a knob that makes the symptom tolerable is
  a reason to check the cause, not a substitute for it.
- **The right-code pass allocates per record.** `derive_right_codes` builds a
  `Vec` of at most `TWO_SIDED_RIGHT_R` (default 4) tokens for every corpus
  position, inside a Rayon map. A stack array would remove one allocation per
  record from a pass that is 59% of a control run. Unmeasured, and it only
  helps census runs now that decision runs can skip the pass outright — which
  is exactly why it should not be done on the assumption that it pays.
