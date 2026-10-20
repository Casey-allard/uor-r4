# #471 — Sampled decision runs still pay full-corpus table construction

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T02:45:25Z
- **Updated:** 2026-08-07T18:27:00Z
- **Closed:** 2026-08-07T18:16:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/471
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measured on the #460 STAGES=5 decision run: with `R4_GATE_C_SAMPLE=10000` the Gate C phase still took **~85 minutes** instead of the expected ~60s.

**Cause:** the sample knob (#467, PR #468) bounds the per-position arm loop, but not the full-corpus work that runs before it. `derive_right_codes` and `TwoSidedTable::build` (the #446 two-sided and latent arms) walk all 2.11M records regardless of the sample size, and a 5-stage code pass is heavier per record than 4-stage.

**Why it matters:** the whole point of the sampled mode is that a decision run should cost minutes. As it stands, a decision run still pays census-scale cost for **instrumentation rows that are not the metric under test** — the headline is Rule 1+2, while the two-sided and latent-mix arms are diagnostics carried over from #446 (which is closed; its family is measured dead for top-1, alive for bits).

**Options:** (a) make those table builds respect the sample — they only need to cover positions the sampled evaluation will touch, plus whatever construction-split support the tables require; or (b) skip the two-sided/latent arms entirely when sampling is active and say so in the output, so a sampled run reports only the rows it can produce cheaply. Option (b) is simpler and probably right: those arms exist to answer a question that has already been answered.

**Definition of done:** a sampled Gate C run on the 2.11M corpus completes in minutes, the rows it prints are unchanged in value for the rows it still prints, and any arm skipped under sampling is named in the output rather than silently dropped.

## Comments (4)

### comment 5212670735 — 2026-08-07T05:08:58Z — Casey-allard

Confirming the diagnosis against the tree so this is pick-up-able cold, and unassigning.

In `score.rs`, `derive_right_codes(...)` runs `(0..corpus.n).into_par_iter()` calling `assign_for_bundle` for **every** record; `TwoSidedTable::build(...)` and `LatentRightTable::build(...)` likewise scan the full corpus — and only afterwards does `gate_c_sample(held_out)` narrow the scored positions. No skip or sample knob exists for these arms (only `R4_LATENT_CLASS_DEPTH`). PR #470 explicitly excluded this pass: the right-context codes are a different quantity and were not cached.

**Two localized options, both confined to `score.rs` around the table builds plus `code_sidecar.rs`:** (a) extend the kappa-keyed sidecar to the right-context code pass as a second `R4CS` stream keyed on the same two kappas plus the right-context radius; or (b) add an arm-skip knob so a sampled decision run that never reads the two-sided / latent rows does not build them, naming the skipped arms in the output rather than dropping them silently. Option (b) is simpler and probably right — #446 is closed and those arms are diagnostics, not the headline.

**Interaction worth knowing:** #469 lever B (vectorizing the assign path) would speed `derive_right_codes` too, since it is the dominant consumer. Doing lever B first shrinks this issue's payoff to cold runs; doing this first shrinks lever B's. Not a blocker either way, just sequencing.

### comment 5214589778 — 2026-08-07T08:29:38Z — Casey-allard

Sequencing note: #469 lever B merged (#476, b90abe9), so the interaction flagged in the comment above has resolved in the direction that shrinks this issue's payoff.

`derive_right_codes` — the dominant consumer named there — now builds `runtime::AssignTables` once and calls `assign_code_for_bundle_with`, measuring **1.60x** on the pinned TLA7 artifact. The full-corpus table construction this issue is about still happens; it is just 1.6x cheaper per record now.

That does not close this issue. The structural complaint stands: a sampled decision run still pays a **full-corpus** pass for arms it never reads, and 1.6x off an unnecessary pass is still an unnecessary pass. It does change the ranking — option (b), the arm-skip knob you recommended, remains the right shape and is now the *whole* remaining win rather than sharing it with vectorization. Option (a), extending the κ-keyed sidecar to the right-context code pass, is correspondingly less attractive: it would cache a pass that is already 1.6x cheaper and still only helps the second run.

Nothing here is assigned; leaving it that way.

### comment 5219802042 — 2026-08-07T16:54:39Z — Casey-allard

Picking this up (assigned). Plan below, posted before the work so the shape is
reviewable and so the measurement that decides the design is declared first.

## What I verified in the tree first

`evaluate_gate_c` does four full-corpus passes before `gate_c_sample` narrows
anything, and the sample knob bounds only the fourth:

1. `fwd_table` — the #399 M2 forward-anchor table, `0..corpus.n` × stride.
2. `derive_right_codes` — full-corpus `bundle_window_plain` +
   `assign_code_for_bundle_with`. **Not cached** (PR #470's sidecar covers the
   left codes only; the right-context codes are a different quantity).
3. `left_codes` — the same pass on the left window, **κ-sidecar cached** since
   #470, so free on a warm run and full price on a cold one.
4. `TwoSidedTable::build` + `LatentRightTable::build` — five and ten levels
   respectively, each a full-corpus scan plus a sort.

Only 2 and 4 are what this issue names. 1 and 3 are the same structural
complaint and I want the measurement to say whether they matter before I touch
them.

Consumer check, so the blast radius is known: the twenty-four two-sided /
latent fields on `GateCOutcome` are read by **the CLI printer and nothing
else**. `scripts/check_gate_c_regression.py` reads `rule12_precedence` only,
and `GateCOutcome` has no `Deserialize` impl, so a schema change here cannot
break the trend alarm or any downstream parse.

## Step 1 — measure before designing (Rule 1: cheap check first)

I am not going to guess which of those four passes owns the 85 minutes. First
change is permanent phase timing in `evaluate_gate_c`, then this, twice:

    cargo run -q --release --bin r4 -- transformerless score \
      --corpus-meta crates/uor-r4-core/tests/fixtures/c_meta.bin \
      --corpus-recs crates/uor-r4-core/tests/fixtures/c_recs.bin \
      --artifacts  crates/uor-r4-core/tests/fixtures/tless_artifacts.bin \
      --out /tmp/gate_c_471

— the `scripts/gate_c_trend.sh` invocation, so the instrument is one CI already
trusts — under `R4_GATE_C_SAMPLE=10000`, cold sidecar and warm sidecar.

**Run contract for that measurement**

    metric to move:       sampled Gate C wall clock, 500k fixture,
                          R4_GATE_C_SAMPLE=10000. Currently ~60s (README).
    reachability ceiling: the scored rows themselves are 10,000 positions of
                          402,802, i.e. ~2.5% of a full pass. So the floor a
                          skip can reach is (prologue + 2.5% of scoring), and
                          whatever fraction of the 60s is NOT full-corpus
                          prologue is unreachable. The phase timings say what
                          that fraction is; if the four passes are under half
                          the wall clock, option (b) is not worth its schema
                          change and I will say so rather than ship it.
    instrument + verdict: the phase timing itself, on the 500k fixture. Binding.
    exit rule:            skipping the named group removes >=60% of sampled
                          wall clock, AND every retained row is bit-identical.
    if positive:          ship the knob, extend to a second corpus size to make
                          the 2.11M number a measured law (below).
    if negative:          record the profile in docs/, close #471 as measured
                          dead with the number, and redirect to whichever pass
                          the profile actually blames.
    cost estimate:        ~5 min per run, four runs. No other work blocked.

## Step 2 — the knob, if step 1 clears the exit rule

`R4_GATE_C_SKIP_ARMS`, comma-separated **arm-group names**, not a boolean.
Unset is behaviour-identical to today, in the `R4_GATE_C_SAMPLE` /
`capacity_override_usize` contract style. An unrecognised name panics rather
than being ignored — a typo that silently measured everything would be worse
than no knob.

One group ships: `right_context` — the #446 M1 two-sided family and the #446 M2
latent family together, because they are exactly the closure of
`derive_right_codes`. Grouping them by *what they cost* rather than by issue
number is what makes the knob extensible without becoming a list of ad-hoc
flags. A `forward_anchor` group for `fwd_table` ships only if step 1 says it is
worth a row.

This is your option (b), unchanged in intent. I am not doing option (a): with
#476 landed it would cache a pass that is already 1.60x cheaper and would still
only help the second run, and it leaves the table builds and the sort in place.

## Step 3 — the part I want to get right: a skipped arm must not read as a zero

This is the whole risk in option (b). Five vacuous instruments turned up in one
session on this repo, and the rule we wrote down was *an all-zero result across
every arm is a harness bug until proven otherwise*. A skip that leaves
`rule12_latent_mix.top1_agreement = 0.0` in `score_report.json` manufactures
exactly that reading, and `latent_exit_rule_met: false` would read as
*measured and failed* rather than *never measured*.

So the twenty-four affected fields move into one `RightContextArms` struct
behind `Option`, schema 25 → 26. A skipped run serialises
`"right_context_arms": null` and the CLI prints the five rows as `SKIPPED
(R4_GATE_C_SKIP_ARMS=right_context)` with a banner naming the group. The
absence becomes unrepresentable as a measurement instead of merely documented,
and Rust stops anyone reading those rows without handling the skip. Since
nothing deserialises `GateCOutcome` and the trend alarm reads only
`rule12_precedence`, the change is contained.

## Step 4 — tests

- **Equivalence, the DoD's actual claim:** the same fixture and the same
  sample, run with and without the skip, produce *bit-identical* values for
  every retained row. It fails if a skip ever moves a headline number.
- **Anti-vacuity guard on that test:** the control arm asserts
  `right_context_arms.is_some()` and that at least one two-sided row is
  non-degenerate, so the equivalence assertion cannot pass by comparing two
  empty runs. Without this the test is the sixth vacuous instrument.
- Env-parse units: unset, valid name, unknown name panics, whitespace/case.

## Step 5 — the 2.11M claim, honestly

The DoD is written at 2.11M and there is no cached 2.11M compile to reuse. I
would rather not quote an extrapolation dressed as a measurement, so: run the
before/after at two corpus sizes, show the skipped passes scale linearly in
`n`, and state the 2.11M figure as arithmetic off a measured slope with the
slope printed. If that reads as too weak for the DoD, say so and I will spend
the hours on a real 2.11M run instead — I would rather ask than quietly
downgrade your acceptance criterion.

## Step 6 — records

`docs/gate_c_arm_skip_471.md` with the phase profile (the reusable artifact
here — nobody has published where a Gate C run's time actually goes), the
README open-issues row, the schema-26 entry in the `ScoreReport` history, and
the outcome posted against the run contract above.


### comment 5220654586 — 2026-08-07T18:26:51Z — Casey-allard

Delivered in **#482 → 3401c60**. Record: `docs/gate_c_arm_skip_471.md`. Posting against the run contract I declared before launching.

## The run contract, answered

```
metric to move:       sampled Gate C wall clock, 500k fixture,
                      R4_GATE_C_SAMPLE=10000
  → 86.56s control, 32.14s skipped.

reachability ceiling: the floor a skip can reach is (prologue + the scored
                      sample); whatever fraction of the run is NOT
                      full-corpus prologue is unreachable.
  → measured: the prologue is 60.4% (right-context pass 51.05s + tables
    0.65s + scorers 2.60s), the irreducible remainder 36.6% is the 10,000
    scored positions. The skip reached 32.14s against a floor of ~31.7s.
    Nothing left on this lever.

instrument + verdict: the phase timing itself, binding.
  → it did bind, and it overruled the issue's own diagnosis (below).

exit rule:            >=60% of sampled wall clock removed AND every
                      retained row bit-identical.
  → BOTH MET. 62.9% of the Gate C phase (40.1% of the whole score
    pipeline). 45 of 45 remaining gate_c keys identical, plus compile
    inputs and emitted graph.

if positive:          ship the knob, extend to a second corpus size.
  → done. 250k and 500k give 105.7 and 102.1 µs/record — linear to
    within 3.5%.

cost estimate:        ~5 min per run, four runs.
  → six runs (two extra: the peak-RSS pair). No other work blocked.
```

## What the measurement overruled

The issue named `derive_right_codes`, `TwoSidedTable::build` and `LatentRightTable::build`. **Two of the three are 0.8% of a sampled run.** The two table builds are 0.65s of 86.56s; all of the cost is the right-context code pass at 59.0%.

Two more passes that look like the same complaint and are not: the **left** code pass is 0.09s, because the store build already wrote the #469 lever-A sidecar earlier in the same process — it is paid once per run, by someone else, before Gate C starts. The **forward-anchor** table (#399 M2) is 0.08s. There is no case for a `forward_anchor` arm group, and I would rather that be written down than rediscovered.

Your sequencing note was right for a reason neither of us had stated: a κ-keyed sidecar for the right codes (option a) misses on exactly the runs that need it, because a *decision* run is by definition one that changed the artifact or the corpus — which is what invalidates the key. Option (a) would have bought the second run of an unchanged configuration, which is the case nobody was waiting on.

## Where the knob does not pay

A census run over the whole held-out split (100,306 positions, same corpus) spends 265s of its 322s scoring. The right-context pass is then ~16%, not 59%. The skipped cost is fixed in `n` and the retained cost scales with the sample, so this is a decision-run lever by construction — anyone reaching for it on a census run should expect roughly a sixth.

## Absence is not zero — and the test caught me

The twenty-six #446 M1/M2 rows moved into `gate_c.right_context_arms: Option<..>` (schema 25 → 26), `null` on a skipped run, with `skipped_arm_groups` naming what was dropped and the CLI printing `SKIPPED` in each row's own place. The M2 exit-rule verdict is **suppressed rather than printed as NOT MET**: a pre-declared exit rule reported as unmet is a result, and that one would never have run.

`tests/gate_c_arm_skip.rs` caught two fields I had missed on its first execution — `win_loss.twosided_vs_rule12_live` and its shuffled twin were still flat, publishing `{both_correct: 0, neither: 0, other_only: 0, scorer_only: 0}` beside five populated cross-tabs. That is the vacuous-instrument pattern reproduced inside a change written specifically to avoid it, found by a machine rather than by a reader. Worth logging as evidence that the equivalence-test-with-anti-vacuity-guard shape pays for itself immediately.

The move is behaviour-preserving and demonstrated rather than argued: a schema-25 report produced before any of this was written and a schema-26 control report produced after it **agree on every value of every `gate_c` key**, key sets equal, nothing missing and nothing added.

## The part that is NOT closed

**The eighty-five minutes is not explained.** 102 µs/record extrapolates to ~215s at 2.11M on a two-core box — about four minutes, not eighty-five. I tested the memory hypothesis rather than assuming it, and it came back negative: peak RSS 1,382 MB control vs 1,305 MB skipped, a 5.6% difference that does not scale into a swap story on a 16GB machine.

So this issue closes having removed a measured 59% of a sampled run while leaving the number that provoked it unaccounted for. Filed as a follow-up rather than buried in this record, because a knob that makes a symptom tolerable is a reason to check the cause, not a substitute for it. The phase timing shipped permanently for exactly that: the next 2.11M run attributes its own wall clock, and the answer arrives as a line of output instead of an argument.



---
_Comment/review ids in this file: 5212670735, 5214589778, 5219802042, 5220654586_

