# D20 §2 — the geometric read vs an ordinary parameter-matched control

**Status: MEASUREMENT COMPLETE for the pre-registered 3x3. Verdict:
INCONCLUSIVE (overlapping ranges, not a tie).** No verdict on the mechanism, and
nothing is retired, deprecated or relabelled. The arm with the higher mean is
reported together with the confound that most plausibly explains it and with the
seed failure that forbids calling it a win.

> **PRE-REGISTERED RESULT (3 seeds per arm, all six runs sealed).**
>
> | arm | s1 | s2 | s3 | mean | sd | variance | range |
> |---|---|---|---|---|---|---|---|
> | G `rrarra` | **0.0073** | **0.9995** | **1.0000** | 0.6689 | **0.5730** | **0.328307** | **0.9927** |
> | C6 matched | 0.2529 | 0.2510 | 0.2563 | 0.2534 | **0.0027** | **0.0000075** | 0.0054 |
>
> **Verdict by the rule fixed before the numbers: INCONCLUSIVE.** Mean difference
> (control − geometric) = **−0.4155**, which exceeds the 0.05 threshold, but
> `min(G) = 0.0073` is *below* `max(C6) = 0.2563`, so per-seed separation fails;
> a control win is equally impossible (`min(C6) = 0.2510 < max(G) = 1.0000`). A
> tie is excluded too (`|Δmean| = 0.4155 > 0.05`).
>
> V1 equal training holds for all six runs: `steps_completed` 1800,
> `stopped_early_at_max_seconds` false, `supervised_queries_seen` 460,800.
> V2 condition 3 holds for both arms: geometric `read_firing` MEASURED with
> 1981 / 2002 / 1994 non-zero rows of 2048; control `read_firing` UNAVAILABLE by
> construction, with its ablation counter at 2048/2048 rows changed on all three
> seeds. V3 asymmetry confirmed on both sides: the control scores 12,288
> positions at the last token against the geometric arm's 4,096.
>
> **The arms differ far more in reliability than in level.** The control's
> per-seed variance is `7.5e-6` (sd 0.0027, range 0.0054) against the geometric
> arm's `0.328` (sd 0.573, range 0.9927) — a factor of ~44,000. At seed 1 the
> geometric arm sits at 0.0073, below the 0.1240 non-learning recency floor and
> below every control seed; at seeds 2 and 3 it is at 0.9995 and 1.0000 with
> 99.0 % and 99.4 % of its argmaxes flipping under key ablation. A single-seed
> report of either number would be a serious error — this is precisely what D20
> §2 condition 2's ≥3-seed requirement exists to catch.
>
> Held-out-class panel: G 0.0000 / 1.0000 / 0.9980 (mean 0.6660); C6 0.0000 /
> 0.0010 / 0.0010 (mean 0.0007). Step-1800 train query NLL: G 5.1353 / 0.0019 /
> 0.0013; C6 3.6375 / 3.7086 / 3.5502.

Evidence labels: **MEASURED** (sealed report root), **DERIVED** (by hand from
source arithmetic), **READ** (source/argv), **ASSUMED** (not verified).

**How the runs were divided.** The parent session owns the battery
(`~/uor-r4-local/mqar-bench/d20-control-20261006-0130-seq`, runner
`~/uor-r4-worktrees/d20-control/d20-battery-seq.sh`) and produced the geometric
arm. This session produced the **entire control arm**: seed 1 filled a root that
had been killed unsealed at step 1300/1800 and that neither parent script
scheduled, and seeds 2-3 were run here because the battery places both C6 cells
last (positions 10-11 of 11). Four of this session's benches were SIGTERM'd from
outside within ~5 min (the seed-1 pair at 00:47, the geometric seed-1 redo at
00:55 killed at 296 s, and the geometric seed-2 duplicate at 01:26 which I killed
myself on seeing the parent had the identical run in flight 8.5 min ahead). I
hold no `kill`/`pkill` in this session; the external kills are read as the parent
owning the CPU. Nothing here depends on a killed run.

Regenerate every table from the sealed roots:

```bash
python3 ~/uor-r4-worktrees/d20-run/d20-run/analyze.py
```

---

## (i) Pre-registered arms, config, metric, seeds, thresholds

Full text and amendments: `d20-run-preregistration.md` (written 00:50, before any
run of mine completed).

| item | value |
|---|---|
| G (geometric) | `arm=stack pattern=rrarra read=l2 rotation=true width=128 heads=4 mlp=384` |
| C6 (parameter-matched control) | `arm=transformer layers=6 width=128 heads=4 mlp=matched match_pattern=rrarra match_read=l2 match_rotation=true` |
| shared config | `device=cpu context=512 batch=8 pairs_per_bucket=8 steps=1800 lr=0.001 warmup=100 min_lr=0.1 weight_decay=0.1 clip=1.0 eval_every=100 curve_sequences=8 final_sequences=64 probe_steps=none` |
| safety cap | `max_seconds` 3600 (battery) / 5400 (this session's fill-in run) — a truncation guard, not a recipe parameter |
| seeds | 1, 2, 3 (≥3 required by D20 §2 condition 2) |
| primary metric | `results.final_in_class_fresh_pairings.accuracy` — argmax over the full vocabulary at every scored query position, 64 fresh in-class sequences (32 on the held-out-class panel) |
| secondary | per-bucket accuracy, `final_held_out_class_pairings.accuracy`, step-1800 train query NLL, `context_reachability` |

**Thresholds, fixed before the numbers:** control wins if
`mean(C6) − mean(G) > +0.05` **and** `min(C6) > max(G)`; geometric wins if
`mean(G) − mean(C6) > +0.05` **and** `min(G) > max(C6)`; tie if `|Δmean| ≤ 0.05`;
otherwise inconclusive. A verdict is emitted only at ≥3 seeds per arm.

**Config interpretation, stated before running.** Stage-0's arrangement used the
geometric arm `pattern=aaaaaa`; the control's frozen reference (the commit's own
test, and the task's shared `<P>`) is `rrarra`, so `rrarra` is the pre-registered
pair. `rrarra` is **4 recurrence (`r`) + 2 read (`a`) layers** — I corrected my own
first draft, which had this backwards, by recomputing `StackConfig::shapes()`.

**The commit did not change the shared training path.** `git show 5a37b981` over
the example is additive in `run()` (`@@ -1451,0 +1940,21 @@`,
`@@ -1470,0 +1980,3 @@` — 0 deleted) and the only deletions are two doc lines, the
`"geometric_stack"` kind literal, and the `access_note` string. Data generation,
loss, scoring, optimizer and LR schedule are identical, so the Stage-0 `aaaaaa`
root is directly comparable to tonight's runs.

**The primary metric has a high non-learning floor. MEASURED** — same panel, same
task settings, `mode=task-baselines`, no arm and no training
(`d20run-20261006-0056-baselines`):

| bucket | d16 | d64 | d200 | d400 | overall |
|---|---|---|---|---|---|
| recency rule, in-class | **0.4453** | 0.0508 | 0.0000 | 0.0000 | **0.1240** |
| recency rule, held-out class | 0.4922 | 0.0430 | 0.0000 | 0.0000 | 0.1338 |

"Answer the value written most recently before the query" — a rule that never
reads a key — already scores 0.445 in `d16` and 0.124 overall. A `d16`-heavy
headline is not evidence of key-addressed recall.

---

## (ii) Exact commands

Executed by the parent session's sequential battery
(`~/uor-r4-worktrees/d20-control/d20-battery-seq.sh`, root
`~/uor-r4-local/mqar-bench/d20-control-20261006-0130-seq`):

```bash
BIN="$HOME/.cache/uor-r4-d20control/release/examples/mqar-bench"
COMMON="device=cpu context=512 batch=8 steps=1800 lr=0.001 warmup=100 eval_every=100 \
curve_sequences=8 final_sequences=64 max_seconds=3600 probe_steps=none"
GEOM="pattern=rrarra read=l2 rotation=true width=128 heads=4 mlp=384"
CTRL="width=128 heads=4 mlp=matched match_pattern=rrarra match_read=l2 match_rotation=true"
/usr/bin/time -l "$BIN" out=$PARENT/R2-G-rrarra-s2   $COMMON $GEOM seed=2
/usr/bin/time -l "$BIN" out=$PARENT/R2-C2-matched-s2 $COMMON arm=transformer layers=2 $CTRL seed=2
# ... G s1,s3; C2 s1,s3; F2-keyshift s1..s3; C6 s2,s3
```

This session's fill-in of the one primary cell neither parent script schedules
(`C6-matched-s1`), a NEW sealed root:

```bash
~/uor-r4-worktrees/d20-run/d20-run/run_one.sh ctl 1 \
  ~/uor-r4-worktrees/reports/d20run-20261006-0136-ctl6-rrarra-s1 5400 none
#  = $BIN out=<root> device=cpu context=512 batch=8 steps=1800 lr=0.001 warmup=100 \
#    min_lr=0.1 weight_decay=0.1 clip=1.0 eval_every=100 curve_sequences=8 \
#    final_sequences=64 seed=1 max_seconds=5400 probe_steps=none \
#    arm=transformer layers=6 width=128 heads=4 mlp=matched \
#    match_pattern=rrarra match_read=l2 match_rotation=true
```

Diagnostics and reference runs (seconds each):

```bash
for n in 64 8 2; do $BIN out=~/uor-r4-worktrees/reports/d20run-repro-fs$n-20261006 \
  device=cpu context=512 pattern=rrarra read=l2 rotation=true width=128 heads=4 \
  mlp=384 batch=8 steps=2 lr=0.001 warmup=1 eval_every=1 curve_sequences=2 \
  final_sequences=$n seed=2 max_seconds=600 probe_steps=none; done
$BIN out=~/uor-r4-worktrees/reports/d20run-20261006-0056-baselines \
  device=cpu context=512 mode=task-baselines final_sequences=64 seed=1
```

Refused/killed roots, kept as evidence and never reused: six under
`~/uor-r4-worktrees/reports/d20run-20261006-0050-*`, `-0102-*`, `-0056-*`,
`-0128-*`. Attempt 0 was refused in 0.02 s (`AlreadyExists`) because my driver
created the report root before the binary's exclusive `report_output::claim`.

---

## (iii) Parameter-match table (condition 1: a number, not an assertion)

**MEASURED** in sealed roots and independently **DERIVED** from
`StackConfig::shapes()` (`geometric_stack.rs:1276-1350`); they agree exactly.

| arm | mlp | parameters | reference | residual |
|---|---|---|---|---|
| G `rrarra` (reference) | 384 | **1,370,008** | — | — |
| C6, `mlp=matched` (this session, seed 1) | **395** | **1,370,496** | 1,370,008 | **+488** (**+0.0356 %**) |
| C2, `mlp=matched` (parent, seed 2) | **1527** | **1,369,984** | 1,370,008 | **−24** (−0.0018 %) |
| G `aaaaaa` (Stage-0 comparison arm) | 384 | 1,360,584 | — | — |

Solve, mirroring `matched_control_mlp`: base at mlp=1 is 462,720; per MLP column
`3·width·layers = 2,304`; `(1,370,008 − 462,720)/2,304 = 393.79` → round 394,
`+1` → 395 → 1,370,496. Composition: reference = 65,664 global + 4×218,176
(recurrence) + 2×215,820 (read); C6 = 65,664 + 6×217,472. Condition 1 is
satisfied as a number. It is a **total-count** match and is silent on allocation
(see (vi)).

---

## (iv) Per-seed results

**Primary metric, all sealed runs** (`final_in_class_fresh_pairings.accuracy`):

| arm | seed | in-class | d16 | d64 | d200 | d400 | held-out | params | root |
|---|---|---|---|---|---|---|---|---|---|
| G `rrarra` | 1 | **0.0073** | 0.014 | 0.008 | 0.002 | 0.006 | 0.0000 | 1,370,008 | `0130-seq/R2-G-rrarra-s1` |
| G `rrarra` | 2 | **0.9995** | 1.000 | 1.000 | 0.998 | 1.000 | 1.0000 | 1,370,008 | `0130-seq/R2-G-rrarra-s2` |
| G `rrarra` | 3 | **1.0000** | 1.000 | 1.000 | 1.000 | 1.000 | 0.9980 | 1,370,008 | `0130-seq/R2-G-rrarra-s3` |
| C6 matched | 1 | **0.2529** | 0.762 | 0.201 | 0.010 | 0.039 | 0.0000 | 1,370,496 | `d20run-20261006-0136-ctl6-rrarra-s1` (this session) |
| C6 matched | 2 | **0.2510** | 0.754 | 0.217 | 0.006 | 0.027 | 0.0010 | 1,370,496 | `d20run-20261006-0152-ctl6-rrarra-s2` (this session) |
| C6 matched | 3 | **0.2563** | 0.758 | 0.215 | 0.012 | 0.041 | 0.0010 | 1,370,496 | `d20run-20261006-0152-ctl6-rrarra-s3` (this session) |
| C2 depth-matched | 1 | 0.2490 | — | — | — | — | — | 1,369,984 | `0130-seq/R2-C2-matched-s1` |
| C2 depth-matched | 2 | **0.2500** | 0.744 | 0.211 | 0.014 | 0.031 | 0.0010 | 1,369,984 | `0130-seq/R2-C2-matched-s2` |
| G `aaaaaa` (not pre-registered) | 1 | **0.2554** | 0.807 | 0.176 | 0.010 | 0.029 | 0.0000 | 1,360,584 | `stage0-rank-bench-l2-rerun-20261006-000040` |

**Pre-registered verdict: INCONCLUSIVE** — mean(control − geometric) = −0.4155
exceeds the 0.05 threshold, but the per-seed ranges overlap
(`min(G) = 0.0073 < max(C6) = 0.2563`) and no control win is possible either
(`min(C6) = 0.2510 < max(G) = 1.0000`).

| arm | n | mean | sd | variance | min | max | range |
|---|---|---|---|---|---|---|---|
| G `rrarra` | 3 | 0.6689 | 0.5730 | 0.328307 | 0.0073 | 1.0000 | 0.9927 |
| C6 matched | 3 | 0.2534 | 0.0027 | 0.0000075 | 0.2510 | 0.2563 | 0.0054 |

So the two arms differ in **mean by 0.4155 and in variance by a factor of ~44,000**.
The geometric arm is bimodal on this task: near-total failure (seed 1, worse than
the non-learning floor) or near-total success (seeds 2-3, held-out 0.998-1.000).
Every control seed, at both depths (2 and 6 layers: 0.2490, 0.2500, 0.2510,
0.2529, 0.2563), lands in a band 0.0073 wide.

**Same-seed, compute-matched pair** (identical scored positions, 4,096 at the
last position and 2,052 mean): G `rrarra` s2 0.9995 vs C2 s2 0.2500, and C2 s1
0.2490. The geometric arm wins those pairs by ~0.75 — but that pair does not
isolate the geometric read; see (vi).1.

**Mechanism counter (`context_reachability` argmax flips under key ablation)**,
which tracks whether the model's *decision* uses the source key:

| run | flips / 2048 | in-class accuracy |
|---|---|---|
| G s1 | 231 (11.3 %) | 0.0073 |
| G s2 | 2,028 (99.0 %) | 0.9995 |
| G s3 | 2,036 (99.4 %) | 1.0000 |
| C6 s1 / s2 / s3 | 83 (4.1 %) / 58 (2.8 %) / 76 (3.7 %) | 0.2529 / 0.2510 / 0.2563 |

Note it is a *sensitivity* counter, not a correctness one: G s1 flips more rows
than any control (11.3 % vs 3-4 %) while scoring far worse.

---

## (v) Firing counters (condition 3)

**Geometric arm — MEASURED on all three seeds**
(`0130-seq/R2-G-rrarra-s1..s3`):

| field | s1 (0.0073) | s2 (0.9995) | s3 (1.0000) |
|---|---|---|---|
| `read_firing.status` | MEASURED | MEASURED | MEASURED |
| `read_heads` | 8 | 8 | 8 |
| `queries` / `batch_items` / `rows_evaluated` | 256 / 256 / 2,048 | same | same |
| `rows_nonzero_mass_on_key` | **1,981** | **2,002** | **1,994** |
| `rows_over_half_on_key` | 0 | 0 | 0 |
| `context_reachability` rows changed | 2,048/2,048 | 2,048/2,048 | 2,048/2,048 |
| **argmax changed** | 231 (11.3 %) | 2,028 (99.0 %) | 2,036 (99.4 %) |
| max abs logit delta | 2.71 | 21.61 | 23.50 |

Condition 3 is met on every seed — and note what it is worth: the **failed**
seed-1 model fires the counter just as strongly (1,981 / 2,048 non-zero) as the
**perfect** seed-3 model (1,994). A firing counter, by itself, cannot separate a
working mechanism from a broken one; the ablation *argmax* rate does.

**Control arm — `read_firing` is UNAVAILABLE by construction.** `read_heads()`
returns empty for `StackArch::Transformer` (`mqar-bench.rs:1003-1005`), so
`read_firing` returns `UNAVAILABLE` (`:1168-1174`); the library populates the
binding capture only on the geometric read path (`geometric_stack.rs:6333`).
Condition 3 for the control rests on the arch-symmetric ablation counter:

| arm | ablation rows changed | **argmax changed** | max abs logit delta |
|---|---|---|---|
| G `rrarra` s2 (0.9995) | 2,048/2,048 | **2,028 (99.0 %)** | 21.61 |
| C6 s1 (0.2529) | 2,048/2,048 | **83 (4.1 %)** | 1.31 |
| C2 s2 (0.2500) | 2,048/2,048 | **108 (5.3 %)** | 1.76 |

Every row's logits move a little for every arm, but only the arm that solves the
task changes its *decision* when the source key is ablated. The counter therefore
separates "wired to the input" from "using the input", and it is the strongest
condition-3 evidence in this report.

**Defect found in the counter that must be fixed before it is quoted as
condition-3 evidence: `read_firing` measures the wrong slot.** It scores mass on
`q − d`, the gold **key** occurrence (`mqar-bench.rs:1197`). The task writes the
value at `p + 1`, so the answer is read from `q − d + 1`. The bench's own
`read_probe` measures both offsets and shows where the mass is (Stage-0, `aaaaaa`,
step 1800, MEASURED): `d16 key 0.021 value 0.232` against uniform 0.0163 — the
**value** slot carries 11× the key slot and 14× uniform (d64 0.125 vs 0.011,
d200 0.045 vs 0.005, d400 0.027 vs 0.008). So on a model that scores 0.9995 and
flips 99 % of its argmaxes under key ablation, `read_firing` reports mean mass
0.00114 and **0 rows over half** — not because the read is weak, but because it
is looking one position to the left of the operative slot. `rows_over_half_on_key
= 0` must not be read as "the read barely fires".

Cosmetic: for the control, `probe_line` writes
`probe step 0: d16 key NaN value NaN (uniform NaN)` into `log.txt` although the
JSON result is the documented `UNAVAILABLE`; the NaN line reads like a broken
mechanism.

---

## (vi) Asymmetries and flaws, stated before any conclusion

**What IS matched:** for a given seed both arms see **identical batches in
identical order** — the batch RNG is
`Rng::new(s.seed, TRAIN_DOMAIN, (step * s.batch + b))` (`mqar-bench.rs:1866`),
with no arm term — and identical evaluation panels from fixed domains
(`:1915-1930`). Same 1,800 steps (`supervised_queries_seen` = 460,800 for every
sealed run), same LR schedule, same optimizer/weight-decay/clip, same loss
weighting (query positions only), same metric. Condition 1 is a measured number.

**What is NOT matched, and it decides how the result may be read:**

1. **The pattern confound — the most important flaw here.** `rrarra` bundles
   **4 quaternion-recurrence layers, which read no context at all**, with its 2
   geometric reads. Every arm *without* recurrence is pinned in a 0.0073-wide
   band around 0.25 regardless of its context mechanism or depth: G `aaaaaa`
   (6 geometric reads) 0.2554, C6 (6 attention layers) 0.2529 / 0.2510 / 0.2563,
   C2 (2 attention layers) 0.2490 / 0.2500. Only arms *with* the 4 recurrence
   layers ever exceed that band — and only on 2 of 3 seeds (0.9995, 1.0000) while
   failing completely on the third (0.0073). So the one large effect is
   confounded with the presence of the recurrence layers, the geometric read is
   **not isolated by any arm that exists**, and the effect is an
   optimisation/seed phenomenon rather than a deterministic architectural
   advantage. The exact experiment that would separate them is in (vii).
2. **The control cannot express the matching architecture.** `StackArch::Transformer`
   forces every layer to attention regardless of `pattern`
   (`geometric_stack.rs:1276-1300`, `layers_hooked`: `(Transformer, _) => attn`),
   so "same pattern, ordinary attention reads" is unrepresentable. There is also
   no "recurrence + attention-read" arm. This is the missing component (vii).
3. **Unequal scored positions, favouring the control.** MEASURED: C6 12,288 at
   the last position and 6,156 mean over the window, against G's 4,096 / 2,052 —
   **3× the positions per query token**, all 6 layers reading context against 2.
   (C2 is the depth-matched exception: 4,096 / 2,052, identical to G.) This
   asymmetry pushes *against* the geometric arm's win, so the win cannot be
   explained by the control being starved of context access.
4. **Parameter match is a total-count match.** C6 spends 6×65,536 on q/k/v/o;
   G spends 4×70,592 on recurrences plus 2×68,236 on reads. Condition 1 is silent
   on allocation.
5. **Not a hobbled control (checked, not assumed).** Same fused kernel, same
   strict causal mask and softmax, same pre-norm, SwiGLU MLP, optimizer, clip,
   weight-decay and LR schedule. The control has RoPE; the geometric read instead
   has a learned per-distance `read.age` table, a `NoRead` slot and an `L2` score
   with learned `log_beta`/`offset` — the mechanism under test, not a defect.
6. **Initialisation is not identical** (impossible across shapes); only the seed
   is shared.
7. **Two roots were destroyed by SIGTERM** after substantial training and are not
   evidence (unsealed roots have no result): `G-rrarra-s1` (9 s) and
   `C6-matched-s1` (step 1300/1800, ~36 min). The battery's `record()` skips any
   root that already exists, so a killed run silently becomes a permanent hole —
   it would have printed `BATTERY COMPLETE` with the arm short a seed. Both holes
   were reported and one was re-run; `C6-matched-s1` is covered by this session.
8. **A near-perfect number must not be quoted from the wrong root.**
   `d20-control-20261006-002829-repro-rrarra-s2` is `status: failed`
   (`invalid reference request: read binding needs one in-range query per labelled
   batch item`) yet its log ends `final in-class fresh acc 0.9995 ... held-out
   1.0000`. That run used the binary as it existed at 00:28-00:41; the file at
   that path was rebuilt 00:42:43 and the commit is 00:42:54, so it ran an
   intermediate build. Re-running its exact panel with the committed binary
   completes with `read_firing: MEASURED` (roots `d20run-repro-fs*`), so the
   failure is not in the committed code — and the parent's `R2-G-rrarra-s2`
   reproduces 0.9995 from the committed build. Quote the sealed root.
9. **CPU contention is load-bearing.** Host load 36-54 on 8 cores from other
   labs, swap 7.6/9.2 GiB, and three concurrent benches measured 2.35-3.7 s/step
   against 0.44-0.66 s/step for one alone. Because `max_seconds` is wall-clock
   and the LR schedule always anneals over 1,800, contention can silently void a
   run.

---

## (vii) Named missing component (D20 §2 condition 4)

**Missing component 1 — an arm that isolates the mechanism under test: the same
`rrarra` pattern with ordinary attention in place of the geometric read** (or,
equivalently, a recurrence-plus-attention arm). As written, `StackArch::Transformer`
forces every layer to attention and ignores the pattern, so the parameter-matched
control differs from the geometric arm in *two* ways at once: the read mechanism
**and** the presence of 4 recurrence layers. Every arm without recurrence sits at
0.2500-0.2554 and the only arm with recurrence sits at 0.9995; until an arm with
recurrence and an ordinary read exists, the observed effect cannot be attributed
to the geometric read.

**Missing component 2 — a mechanism-internal firing counter for the ordinary
control.** Condition 3 asks for proof the mechanism executed. The geometric arm
has one; the control has none (its attention path does not populate the binding
capture), so the control's condition-3 evidence is an input-ablation counter —
which proves the logits depend on the source key, not that any read weight landed
on it.

**Missing component 3 — the firing counter must measure the operative slot.**
`read_firing` counts mass at `q − d`; the value is at `q − d + 1`, where the
bench's own `read_probe` shows 11× the mass. Count both offsets.

What would change the answer: a recurrence-plus-attention arm (removes the
confound); a `ReadBinding` capture on the transformer's softmax rows (symmetric
condition 3); the value-slot fix in `read_firing`; and **more seeds on the
geometric arm**, because with an outcome that is 0.0073 on one seed and 0.9995 /
1.0000 on the next two, three seeds cannot estimate either its mean or its
failure rate — five to ten seeds are needed before any claim about the
mechanism's reliability is defensible. Also useful: the `layers=2` control at all
3 seeds and the `aaaaaa` geometric arm at the same 3 seeds (separates depth from
mechanism — 4 of those 6 cells already exist here: C2 s1 0.2490 and s2 0.2500,
`aaaaaa` s1 0.2554), and a second independent build of the committed source to
bind binary to commit by behaviour rather than by symbols (not done: the
preserved binary was reused as instructed; it carries the commit's added symbols
and the frozen parameter count).

---

## (viii) Complete cost

| item | value |
|---|---|
| binary | `~/.cache/uor-r4-d20control/release/examples/mqar-bench`, 6,946,112 B, md5 `21856850c6b1d4bd8ae66a0d76b0e0f7`, mtime 00:42:43, commit `5a37b981` (never rebuilt or overwritten by this session) |
| threads | `RAYON_NUM_THREADS` unset (library default over 8 logical cores); 90-360 % CPU observed per bench |
| host | Apple M1, 8 cores, 16 GiB RAM; load 36-54 from other labs' compiles; swap 7.6/9.2 GiB |
| **this session's CPU** | 7 benches launched, **3 sealed** (the whole control arm: `C6-matched-s1..s3`), 4 killed. Sealed wall times 1,309 s + 2,274 s + 2,276 s = **5,859 s ≈ 1.63 h** of one core; peak RSS 3.14 GB / 2.19 GB / 5.08 GB. Killed runs ≈ 400 s. Repro + baseline runs ≈ 45 s. **Total ≈ 1.75 h**, inside the 2-3 h budget; the rest was the parent's battery |
| other sealed runs quoted | G s1 wall 1,616.2 s (train 1,487.2 s); G s2 1,025.9 s (train 908.0 s, median step 0.44 s, RSS 6.54 GB); G s3 732.3 s (train 678.9 s); C2 s2 train 696.5 s |
| disk | 41.36 GiB free at 00:45 → 34.29 GiB at 00:58 → 32.2-39 GiB after (other labs' build churn, not this work; my three sealed roots are 128-132 KiB each). Hard gate 25.12 GiB, never approached. Checked with `df -h /System/Volumes/Data` and `statvfs` throughout |
| new storage, this session | 13 roots under `~/uor-r4-worktrees/reports/` (2 refused, 4 killed, 7 sealed) ≈ 1.5 MiB; cost logs in `~/uor-r4-worktrees/d20-run/cost/` |
| elapsed | 00:43 → 02:55 EDT (~2 h 12 min), of which ~1.75 h was this session's own sealed compute |

---

## (ix) Durable output paths

* pre-registration (with amendments): `~/uor-r4-worktrees/d20-run/d20-run-preregistration.md`
* this report: `~/uor-r4-worktrees/d20-run/d20-run-report.md`
* analysis + scripts: `~/uor-r4-worktrees/d20-run/d20-run/{analyze.py,run_one.sh}`; machine-readable `~/uor-r4-worktrees/d20-run/analysis.json`
* sealed roots, this session: `~/uor-r4-worktrees/reports/d20run-20261006-0136-ctl6-rrarra-s1` (C6 s1), `~/uor-r4-worktrees/reports/d20run-repro-fs{64,8,2}-20261006`, `~/uor-r4-worktrees/reports/d20run-20261006-0056-baselines`
* sealed roots, parent: `~/uor-r4-local/mqar-bench/d20-control-20261006-0130-seq/{R2-G-rrarra-s2,R2-C2-matched-s2,...}`
* Stage-0 comparison arm: `~/uor-r4-worktrees/reports/stage0-rank-bench-l2-rerun-20261006-000040`
* void/unsealed, kept as evidence: `~/uor-r4-worktrees/reports/d20run-20261006-{0050,0102,0056,0128}-*`; `~/uor-r4-local/mqar-bench/d20-control-20261006-0110-battery/_poisoned-20261006/`
* cost logs: `~/uor-r4-worktrees/d20-run/cost/*.cost.txt`

---

## (x) Push status

Branch `deepseek/d20-run`, based on `5a37b981` (`deepseek/d20-control`), pushed to
`origin`. No PR opened or queued. Worktree `~/uor-r4-worktrees/d20-run`; the
preserved target dir `~/.cache/uor-r4-d20control` was never written to.
