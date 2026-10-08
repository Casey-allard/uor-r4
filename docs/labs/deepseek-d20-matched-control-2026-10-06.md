# D20 §2 conditions 1 and 2 for the geometric read: a parameter- and compute-matched ordinary control

**Lab:** OpenCode/DeepSeek · **Date:** 2026-10-06 · **Base:** `origin/main` @ `d2cae5ba`
**Worktree:** `~/uor-r4-worktrees/d20-control`, branch `deepseek/d20-control`
**Scope:** a measurement and a named missing component. **No mechanism is retired, parked, demoted or
relabelled by this document.** D20 §3's suspension does **not** apply: its §2 preconditions are not met.

---

## 0. The claim under test, and its verdict

The prior finding was that `crates/uor-r4-training/examples/mqar_bench_step2/` contains an `ArmSpec`
with **only the Stack variant and the architecture hardcoded**, so the geometric read "may never have
been compared to a parameter-matched ordinary attention model at all."

**VERDICT: TRUE.**

| Evidence | Location (`origin/main` `d2cae5ba`) |
|---|---|
| `enum ArmSpec` declares exactly one variant, `Stack` | `crates/uor-r4-training/examples/mqar-bench.rs:476-492` |
| `arch: StackArch::Geometric` is hardcoded in the only `stack_config` branch | `crates/uor-r4-training/examples/mqar-bench.rs:597` |
| `ArmSpec::parse` accepts `arm=stack` only | `crates/uor-r4-training/examples/mqar-bench.rs:499-534` |
| Module doc, verbatim: *"Offline floating-point training only; nothing here is a serving path, and **there is no transformer control**."* | `crates/uor-r4-training/examples/mqar-bench.rs:59` (pre-change) |

The `ArmSpec` is the one in `examples/mqar-bench.rs`; `mqar_bench_step2/` (`decide.rs`, `fact.rs`)
contains no arm registry. The library, not the bench, already shipped the ordinary path:
`StackArch::Transformer` (`crates/uor-r4-training/src/geometric_stack.rs:158-161`),
`StackModel::attention` (`:2060-2091`), `StackConfig::transformer_control` / `transformer` /
`geometric_matched_to` (`:1122-1200`). The claim is therefore about **the instrument**, not about
missing machinery.

---

## 1. The incumbent, reproduced

**Incumbent (the geometric read under test):** `pattern=rrarra`, `read=l2`, `rotation=true`,
`width=128`, `heads=4`, `mlp=384`, `context=512`, `vocab=512`, `read_key_shift=false` — the frozen
Step-2 recipe (`docs/research/barrier-assessment-2026-10-05/proposals-and-reviews.md:637`).

- **Parameters: 1,370,008** (reported by the run).
- **Compute per token at position `t`:** 1,431,048 MACs at `t=256`, 1,563,648 at `t=511`
  (4 `r` layers at 217,088 MACs + 2 `a` layers at 213,504 + ~260·(t+1)).
- **Access:** 2 read layers × 4 heads × (t+1) = **2,840.875 positions/query token** on the final
  in-class panel.
- **Panel identity:** `context=512`, `pairs_per_bucket=8`, buckets `d16/d64/d200/d400`, pairing
  classes `(key_index + value_index) % 4` with class 0 held out, 64 fresh in-class sequences and 32
  held-out-class sequences, all drawn from `seed` and a domain constant alone
  (`mqar-bench.rs:1258-1270`).
- **Artifact identity:** no weights are saved (`save_model=false`, the default, as in every published
  run). The identity is the sealed report root plus its manifest: source commit, full `argv`, task
  block, arm block.

**Published number vs mine** — published root
`~/uor-r4-local/mqar-bench/key-shift-production-787c350d/B-rrarra-seed2`
(`report.json` blake3 `84d2eddb2026fb09520811b7989c54cd9868c64a31cd240b71105068f2989670`):

| | published | this run (`R2-G-rrarra-s2`) |
|---|---|---|
| `final_in_class_fresh_pairings.accuracy` | 0.9995 (2047/2048) | **0.9995 (2047/2048)** |
| `final_held_out_class_pairings.accuracy` | 1.0000 | **1.0000** |
| per-bucket in-class `d16/d64/d200/d400` | 1.000 / 1.000 / 0.998 / 1.000 | **identical** |
| `positions_scored_per_query_token_mean` | 2840.875 | **2840.875** |
| `supervised_queries_seen` | 460800 | **460800** |
| steps / early stop | 1800 / false | **1800 / false** |

**The whole run is numerically identical, not just the headline.** Stripping only the timing
parentheses and the new `reachability:` line, the modified binary's `log.txt` is byte-identical to the
published `log.txt` across all 19 lines (18 evaluation points plus the final line), and every numeric
field of `report.json` matches — including the complete 19-point evaluation curve
(`0.0, 0.0, 0.0117, 0.0938, 0.9141, 0.9531, 0.9766, 0.9766, 0.9805, 0.9844, 0.9883, 0.9961, 1.0 …`).
This is also the proof that the harness extension left the existing arm unchanged.

The published seed-1 failure reproduces exactly: `R2-G-rrarra-s1` = **0.0073**, and the published
`grid3-ctx512-eba6a15f/B-rrarra-l2` = 0.0073.

---

## 2. The matched ordinary control

**Construction.** A new additive arm, `arm=transformer`, building the library's own ordinary
causal-softmax attention stack (`StackArch::Transformer`): RoPE + strict causal mask + SwiGLU MLP +
pre-norm, `StackModel::attention` → `fused_read_selected(ReadScore::Dot)`. `layers`, `width`, `heads`
are explicit; `match_pattern` / `match_read` / `match_rotation` name the geometric arm whose
**measured** total parameter count is matched; `mlp=matched` solves the MLP width (the mirror of the
library's private `matched_mlp_hidden`, `geometric_stack.rs:1378-1391`), `mlp=<n>` overrides. The
report records both counts and the residual under `arm.parameter_match`, so the match is a number,
not an assertion.

**Matching table** (parameters measured by the run; MACs from the reported config):

| | geometric `rrarra` | control C2 (`layers=2`) | control C6 (`layers=6`) |
|---|---|---|---|
| `mlp_hidden` | 384 | 1527 | 395 |
| **parameters** | **1,370,008** | **1,369,984** (−24, −0.0018 %) | **1,370,496** (+488, +0.0356 %) |
| depth (layers) | 6 (4 `r` + 2 `a`) | 2 (attention) | 6 (attention) |
| MACs/token @ t=256 | 1,431,048 | 1,441,560 (**+0.73 %**) | 1,716,552 (**+19.95 %**) |
| MACs/token @ t=511 | 1,563,648 | 1,578,240 (**+0.93 %**) | 2,126,592 (**+35.99 %**) |
| access positions/query token | 2,840.875 | **2,840.875 (exact)** | 8,522.6 (**3×**) |
| non-access bulk MACs (t-independent) | 1,297,408 | 1,303,808 (+0.49 %) | 1,303,296 (+0.45 %) |

**Two controls, because no single one is matched on every axis.**

- **C2 is the parameter- and compute-matched control**: parameters within 24 of 1,370,008 (0.0018 %),
  per-token MACs within 0.93 %, and the context-access term **exactly** equal (2 layers × 4 heads).
  Its residual mismatch is **depth**: 2 layers against the geometric arm's 6.
- **C6 is the depth-matched control**: same 6 layers, width and heads, parameters within 0.036 %. Its
  residual mismatch is **compute**: 3× the access positions and +20 %/+36 % per-token MACs, i.e. the
  mismatch is in the control's favour.
- **Neither is fully matched on every axis; both are stated.** The geometric arm loses to neither on
  compute.

**What was changed, and what was not.** Only `examples/mqar-bench.rs` (plus new helper and analysis
scripts). **No library file was modified**; `geometric_stack.rs` is untouched. The changes are (a) the
new `ArmSpec::Transformer` variant with its parse/label/validate/build/stack_config branches,
(b) `StackArm::kind()`, `access_note()` and `read_heads()` made arch-aware, (c) the two reachability
counters of §4, (d) the report fields they add, (e) the module doc, which said "there is no
transformer control" and now describes the control, (f) two focused tests. `read_heads()` returns
empty for the control only, because the library's `attention` path does not populate the binding
capture (`geometric_stack.rs:6389-6391`); the geometric arm's answer is unchanged, which §1's
byte-identical reproduction proves.

**Tests.** `cargo test --release -p uor-r4-training --example mqar-bench`: **16 passed, 0 failed**,
including the two new ones — `the_transformer_control_is_parameter_matched_to_the_geometric_arm` and
`both_arms_read_the_context_at_the_scored_positions`.

---

## 3. Per-seed results (D20 §2 condition 2 — ≥3 seeds)

Full frozen recipe for every arm: `context=512 batch=8 steps=1800 lr=0.001 warmup=100
weight_decay=0.1 clip=1.0 min_lr=0.1 eval_every=100 curve_sequences=8 final_sequences=64
probe_steps=none`, `RAYON_NUM_THREADS` unset (8, the published setting). Every arm reports
`steps_completed=1800`, `stopped_early_at_max_seconds=false`, `supervised_queries_seen=460800` —
**equal data, equal tokens, equal positions, equal metric, equal steps.**

`in_class` = `final_in_class_fresh_pairings.accuracy`; `held_out` = `final_held_out_class_pairings.accuracy`.
Chance = 1/224 = 0.0045.

| arm | seed | in_class | held_out | d16 | d64 | d200 | d400 |
|---|---|---|---|---|---|---|---|
| **G** geometric `rrarra` | 1 | **0.0073** | 0.0000 | 0.014 | 0.008 | 0.002 | 0.006 |
| **G** | 2 | **0.9995** | 1.0000 | 1.000 | 1.000 | 0.998 | 1.000 |
| **G** | 3 | **1.0000** | 0.9980 | 1.000 | 1.000 | 1.000 | 1.000 |
| **F2** `rrarra` + `key_shift=true` | 1 | 1.0000 | 1.0000 | 1.000 | 1.000 | 1.000 | 1.000 |
| **F2** | 2 | 1.0000 | 1.0000 | 1.000 | 1.000 | 1.000 | 1.000 |
| **F2** | 3 | **0.0063** | 0.0000 | 0.014 | 0.004 | 0.004 | 0.004 |
| **C2** matched, 2 layers | 1 | 0.2490 | 0.0000 | 0.752 | 0.188 | 0.004 | 0.053 |
| **C2** | 2 | 0.2500 | 0.0010 | 0.744 | 0.211 | 0.014 | 0.031 |
| **C2** | 3 | 0.2568 | 0.0010 | 0.742 | 0.234 | 0.008 | 0.043 |
| **C6** matched, 6 layers | 2 | 0.2510 | 0.0010 | 0.754 | 0.217 | 0.006 | 0.027 |
| **C6** | 3 | 0.2563 | 0.0010 | 0.758 | 0.215 | 0.012 | 0.041 |

Mean and sample variance across seeds:

| arm | metric | mean | sample variance | range |
|---|---|---|---|---|
| G (`rrarra`) | in_class | 0.6689 | **0.3283** | 0.0073 – 1.0000 |
| G (`rrarra`) | held_out | 0.6660 | **0.3327** | 0.0000 – 1.0000 |
| F2 (`rrarra` + lineage) | in_class | 0.6688 | **0.3291** | 0.0063 – 1.0000 |
| F2 (`rrarra` + lineage) | held_out | 0.6667 | **0.3333** | 0.0000 – 1.0000 |
| C2 | in_class | 0.2519 | **1.80e-5** | 0.2490 – 0.2568 |
| C2 | held_out | 0.00067 | 3.33e-7 | 0.0000 – 0.0010 |
| C6 | in_class | 0.2537 | — (2 seeds) | 0.2510 – 0.2563 |
| C6 | held_out | 0.0010 | — (2 seeds) | 0.0010 – 0.0010 |

**Paired, per seed** (d = geometric − control):

| comparison | metric | s1 | s2 | s3 | mean |
|---|---|---|---|---|---|
| G − C2 | in_class | −0.2417 | +0.7495 | +0.7432 | +0.4170 |
| G − C2 | held_out | +0.0000 | +0.9990 | +0.9970 | +0.6653 |
| G − C6 | in_class | — | +0.7485 | +0.7437 | +0.7461 |
| G − C6 | held_out | — | +0.9990 | +0.9970 | +0.9980 |
| F2 − C2 | in_class | +0.7510 | +0.7500 | −0.2505 | +0.4168 |
| F2 − C2 | held_out | +1.0000 | +0.9990 | −0.0010 | +0.6660 |

**The mean is not the finding; the variance is.** Both geometric configurations are **bimodal across
seeds**: they either solve the task essentially exactly or fail to learn it at all. The matched
controls are **tightly clustered at ~0.25 in-class and chance on the held-out class**, at every seed.

Three separable facts, which must not be merged:

1. **The geometric stack's ceiling is far above the matched ordinary control's.** On no seed does
   either control solve the task.
2. **The geometric stack's reliability is far below the control's.** Across 6 geometric seeds (2
   configurations × 3) it solves 4 and fails 2; the controls never solve, and never fail either.
3. **The key/query lineage (F2) does not remove the seed lottery.** Published #1704 measured F2 at
   seeds 1 and 2 (both 1.0000) and concluded it "removes the `rrarra` seed lottery"
   (`synthesis.md`, SAVED OPTION). At a third, pre-declared seed it scores **0.0063** — a total
   training failure, indistinguishable from the un-shifted seed-1 failure. F2 moves *which* seed
   fails; it does not make the outcome reliable. The published reliability claim rests on 2 seeds,
   which is exactly the completeness gap D20 §2 condition 2 names.

---

## 4. Wiring reached (D20 §2 condition 3)

Two measurement-only counters run once on the scored positions of the final in-class panel. Both are
extra forward passes; neither changes a parameter or a logit.

**(a) `read_firing` — the geometric read's own binding mass, the `Work::selected_operations`
analogue.** `ContextArm::source_mass` → `StackModel::read_binding_masses` ("the actual read,
including age and NoRead"), one held-out query per (sequence, bucket), every sequence, one batch item
each, processed in chunks of 32: **2,048 read rows per run** (8 heads × 256 queries), measured at both
the key `q−d` and the value `q−d+1`.

| run | in_class | rows evaluated | >0 on key | >0 on value | rows >0.5 on value | mean mass key | mean mass value | uniform |
|---|---|---|---|---|---|---|---|---|
| G s1 | 0.0073 | 2048 | 1981 | 1990 | **0** | 0.0019 | 0.0035 | 0.0078 |
| G s2 | 0.9995 | 2048 | 2002 | *not measured¹* | — | 0.0011 | — | 0.0082 |
| G s3 | 1.0000 | 2048 | 1994 | 2015 | **256** | 0.0012 | **0.1261** | 0.0076 |
| F2 s1 | 1.0000 | 2048 | 2010 | 2009 | **256** | 0.0016 | **0.1260** | 0.0078 |
| F2 s2 | 1.0000 | 2048 | 2012 | 2016 | **257** | 0.0011 | **0.1270** | 0.0082 |
| F2 s3 | 0.0063 | 2048 | 1977 | 1983 | **20** | 0.0025 | 0.0169 | 0.0076 |

¹ `G s2` ran before the value position was added to the counter; see the note below.

Per bucket at G s3: value mass 0.1316 / 0.1271 / 0.1242 / 0.1217 against uniform
0.0157 / 0.0081 / 0.0041 / 0.0025 — **6.4× to 49× uniform at every distance from d16 to d400**, with
one head of eight putting more than 0.5 of its mass on the answer position in 256 of 2,048 rows.

**This is the discriminating result.** In every failed run the read *executed* all 2,048 rows and
selected **essentially nothing** (value mass at or below uniform; ≤20 rows over 0.5). In every solved
run it executed the same 2,048 rows and selected the answer position at ~16× uniform. "No effect"
here cannot mean "never executed": the counter separates *the read fired and selected* from *the read
fired and selected nothing*, and it predicts the outcome of all six geometric runs.

*Note on the counter's design.* The first version measured only the key position `q−d` and reported
mass **below uniform** (0.0011 vs 0.0082) on a run that scored 0.9995. That reading is misleading:
the MQAR answer sits at the **value** position `q−d+1`, and measuring there shows the read selecting
it strongly. The key-only number was measured on `R2-G-rrarra-s2` and is reported only as the reason
the value position was added; it is not used as evidence.

**(b) `context_reachability` — arch-symmetric.** Every query's source key at `q−d` is replaced by a
filler token and the logits at `q` compared with the unmodified forward. A mechanism that never read
the context leaves them bit-identical.

| arm | rows | rows with any logit change | gold logit changed | argmax changed | max abs Δlogit |
|---|---|---|---|---|---|
| G s2 | 2048 | **2048 / 2048** | 2048 | 2028 | 21.61 |
| G s1 | 2048 | **2048 / 2048** | 2048 | 231 | 2.71 |
| C2 s1 / s2 / s3 | 2048 | **2048 / 2048** | 2048 | 114 / 108 / 118 | 1.94 / 1.76 / 1.57 |
| C6 s2 / s3 | 2048 | **2048 / 2048** | 2048 | 58 / 76 | 1.08 / 0.96 |

Both arms' mechanisms are reached on 100 % of scored positions. The **control has no `read_firing`
equivalent** — the library's transformer attention path does not populate the binding capture
(`geometric_stack.rs:6389-6391`) — so its reachability rests on (b) plus its own learning above
chance. That is an asymmetry, stated in §5.

---

## 5. Asymmetries and flaws found in my own comparison

Stated before the conclusion, because the result favours the geometric arm.

1. **The comparison does not isolate the geometric read.** `rrarra` is 4 recurrence layers + 2 read
   layers; the control is pure attention. The measured contrast is *geometric stack
   (recurrence + read) versus ordinary attention stack*, **not** *geometric read versus ordinary
   attention*. §4(a) sharpens this rather than resolving it: where the read works it does put
   6–49× uniform mass on the answer position, so it is doing content-based retrieval — but nothing
   here separates how much of the 0.9980 comes from the read and how much from the recurrence.
2. **Depth is a confound in opposite directions for the two controls.** C2 matches compute exactly but
   is 2 layers against 6; C6 matches depth but has +20 %/+36 % per-token compute and 3× the access
   positions. Neither is a fully matched control; both were run for that reason. The geometric arm
   loses to neither, but "matched" is a claim about axes, not a single fact.
3. **The controls may be undertrained.** Their training query NLL is still falling at step 1800
   (C2 s2: 5.798 → 3.427; C6 s2: 5.764 → 3.709) while their in-class accuracy has plateaued near
   0.23–0.26 since about step 900 and their held-out accuracy *peaks mid-run then collapses*
   (C2 s2: 0.215 at step 400 → 0.004 at 1800). They are learning the training-class prior, which is
   what the adversarial held-out-class panel is built to penalise. A longer budget is not excluded as
   a route to a better control.
4. **The geometric arm's mean is meaningless.** With 0.0073 / 0.9995 / 1.0000 the mean (0.6689) and
   the variance (0.3283) describe a bimodal draw, not a level of performance. The comparison at the
   failing seed is a **training failure**, not a mechanism comparison.
5. **`max_seconds` was raised, 1200 → 3600**, because the machine is shared and loaded. This does not
   change the recipe: the cap never bound — every run reports `steps_completed=1800` and
   `stopped_early_at_max_seconds=false`. Recorded because it is a deviation from the published `argv`.
6. **The panel is seed-derived, so the comparison is paired, not common.** Training sequences, curve
   sets, the final in-class and held-out sets and the probe set all derive from `seed` and a domain
   constant (`mqar-bench.rs:1258-1270`). Both arms at the same seed therefore see byte-identical data
   and identical scored positions — which is what makes the pairing valid — but different seeds are
   different panels, and no across-seed panel average is being reported.
7. **No saved weights.** `save_model=false` everywhere, as in the published runs, so there is no
   artifact to re-score; the evidence is the sealed report roots.
8. **The counter's coverage is one query per (sequence, bucket), not every scored position.** 2,048 of
   the panel's scored rows are measured (8 heads × 256 queries). The arch-symmetric ablation in (b)
   does cover all 2,048 scored positions of the panel, but the binding-mass counter does not.
9. **A coordination hazard external to the measurement.** A second agent in the same workspace ran
   its own copy of this matrix concurrently, SIGTERM'd several of my runs (rc=143) and renamed my
   report roots. The affected roots are listed in §7. Every run reported here completed with `rc=0`
   and `status: complete`.
10. **Concurrency is a net loss on this workload.** Three concurrent arms ran at 2.35–3.7 s/step each
    against ~0.47 s/step for one alone; every run here was sequential.

---

## 6. The named missing component (D20 §2 condition 4, §4)

Per D20, a gap must yield **one named missing component with the evidence that identifies it**. It is
a localiser, not a verdict.

> **The missing component is the acquisition path that makes the geometric read's content-based
> selection reliable across seeds.** The read is wired, reached, parameter-matched, compute-matched
> and *capable* — where it works it selects the answer position at 6–49× uniform mass. Whether it
> works is decided by the seed: 4 of 6 geometric runs select and solve, 2 of 6 select nothing and
> score at chance. The key/query lineage (F2) changes *which* seed fails (1.0000, 1.0000, 0.0063) and
> therefore is **a partial mitigation, not the fix**; the published claim that it removes the lottery
> was measured at 2 seeds.

**Evidence that identifies it** — the same instrument, on the same panel, at the same budget:

| run | outcome | read rows executed | mean mass on the answer position | rows >0.5 on the answer |
|---|---|---|---|---|
| G s1 | fail 0.0073 | 2048 | 0.0035 (**below** uniform 0.0078) | 0 |
| G s3 | solve 1.0000 | 2048 | 0.1261 (**16.6×** uniform 0.0076) | 256 |
| F2 s1 | solve 1.0000 | 2048 | 0.1260 (16.1× uniform) | 256 |
| F2 s2 | solve 1.0000 | 2048 | 0.1270 (15.6× uniform) | 257 |
| F2 s3 | fail 0.0063 | 2048 | 0.0169 (2.2× uniform) | 20 |

The mechanism always executes; only its *selection* is acquired or not. That is a property of the
training path, not of the geometry and not of the wiring.

**What this rules out, explicitly:**

- **Wiring.** Reached on 2,048/2,048 scored rows in every run; the arch-symmetric ablation moves the
  logits at 2,048/2,048 scored positions for both arms.
- **Capacity.** Parameters matched to −24 (−0.0018 %) against the control; compute matched to +0.73 %
  at t=256 and +0.93 % at t=511, with the access term exactly equal.
- **Budget or data.** 1800 steps, 460,800 supervised queries, identical panel and scored positions
  for every arm.
- **The read's ability to select.** It selects at 6–49× uniform at every distance bucket when it
  works.

**The next attempt this identifies** (D20 asks for a repair attempt, not a verdict): the decisive
unmeasured quantity is the difference between a seed that acquires the read's selection and one that
does not. The cheapest discriminator is already built — `read_firing` on the *final* panel is the
outcome; running it at `probe_steps=0,100,200` on the same counter would show whether the failing
seeds never move the selection or acquire it and then lose it. That is a ~10-minute instrumented run
per seed and it is the specific measurement this session's localiser calls for.

---

## 7. Durable outputs

**Worktree / source:** `~/uor-r4-worktrees/d20-control` (branch `deepseek/d20-control` @ `d2cae5ba`),
`CARGO_TARGET_DIR=$HOME/.cache/uor-r4-d20control`. Files:
`crates/uor-r4-training/examples/mqar-bench.rs` (**the only source change**),
`d20-battery-seq.sh`, `d20-battery-parallel.sh`, `d20-control-battery.sh`, `d20-analyze.py`,
`docs/labs/deepseek-d20-matched-control-2026-10-06.md` (this file).

**Sealed report roots**, parent `~/uor-r4-local/mqar-bench/d20-control-20261006-0130-seq/` (each
claimed exclusively before model work, sealed with a manifest; 1.3 MB total):

| arm | root | rc | wall s | peak RSS |
|---|---|---|---|---|
| G s1 | `R2-G-rrarra-s1` | 0 | 1617 | 2.15 GB |
| G s2 | `R2-G-rrarra-s2` | 0 | 1026 | 6.54 GB |
| G s3 | `R2-G-rrarra-s3` | 0 | 732 | 3.28 GB |
| C2 s1 | `R2-C2-matched-s1` | 0 | 1104 | 3.81 GB |
| C2 s2 | `R2-C2-matched-s2` | 0 | 706 | 3.03 GB |
| C2 s3 | `R2-C2-matched-s3` | 0 | 406 | 3.55 GB |
| C6 s2 | `R2-C6-matched-s2` | 0 | 1923 | 3.79 GB |
| C6 s3 | `R2-C6-matched-s3` | 0 | 696 | 2.88 GB |
| F2 s1 | `R2-F2-keyshift-s1` | 0 | 607 | 2.75 GB |
| F2 s2 | `R2-F2-keyshift-s2` | 0 | 599 | 2.70 GB |
| F2 s3 | `R2-F2-keyshift-s3` | 0 | 614 | 2.71 GB |

**Published root used as the incumbent reference:**
`key-shift-production-787c350d/B-rrarra-seed2` (read-only).

**Superseded / void roots, retained, never reused:** under
`d20-control-20261006-0110-battery/` — `G-rrarra-s1-VOID-killed-at-9s`,
`G-rrarra-s2-TRUNCATED-early-stop`, `G-rrarra-s1r-TRUNCATED-early-stop`,
`C2-matched-s2-TRUNCATED-early-stop` (all SIGTERM'd by the other agent, rc=143, no `report.json`),
plus interrupted `C6-matched-s1`, `G-rrarra-s3`, `C2-matched-s1`, `C2-matched-s3`, `F2-*`;
`d20-control-20261006-baseline-rrarra-s2` (killed at ~step 250);
`d20-control-20261006-002829-repro-rrarra-s2` (completed 1800 steps and reproduced 0.9995/1.0000,
then failed to write its report on the counter bug fixed in §4); `d20-smoke-004334-L6`,
`d20-smoke-004403-L2`.

**Reproduce the analysis:**
`python3 d20-analyze.py <sealed root> …` prints §3, §4 and the MAC table from the sealed records.

---

## 8. Cost

| | |
|---|---|
| Elapsed (this session, wall) | **3 h 25 min** (04:20 Z – 07:45 Z) |
| Compute of the reported runs | 11 × 1800-step runs, **10,029 s** summed wall (2 h 47 min), **sequential**, `RAYON_NUM_THREADS` unset |
| Threads | 8 per run (unset ⇒ Rayon default), one run at a time |
| Peak RSS | 2.15–6.54 GB per run (`/usr/bin/time -l`); the 6.54 GB peak is the pre-chunking firing counter. Chunking brought it to 2.7–3.8 GB |
| Build | release, 5 m 08 s cold; ~9 s per example-only rebuild |
| New storage | 0.7 GB target dir + 1.2 GB worktree checkout + **1.3 MB** of sealed report roots |
| Disk headroom | **32.99 GiB free at close against the 40 GiB declared reserve — a breach of ~7 GiB.** My own footprint is ~1.9 GB (0.7 GB target + 1.2 GB worktree) plus 1.3 MB of report roots; the remainder is other labs. It fell from 45.2 GiB at session start to 39.2 GiB after my build, and to 32.99 GiB while other labs built concurrently. I started no heavy job after measuring the breach. Flagging, not fixing — the reserve is the owner's and other labs' material is not mine to delete |
| GPU / pod | **none** — CPU only |
| External cost | **none** |
