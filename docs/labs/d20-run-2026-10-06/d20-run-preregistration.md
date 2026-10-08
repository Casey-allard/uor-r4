# D20 §2 pre-registration — ordinary parameter-matched control vs geometric arm

Written 2026-10-06 00:50 EDT, **before any training run was started by this session**.
Source under test: commit `5a37b981` (`deepseek/d20-control`), binary
`~/.cache/uor-r4-d20control/release/examples/mqar-bench` (6,946,112 bytes, 00:42).

## 1. Arms

Both arms share the task, the data, the training loop, the optimizer and the
scoring path; only the context-access mechanism differs.

* **G — geometric arm:** `arm=stack pattern=rrarra read=l2 rotation=true width=128 heads=4 mlp=384`
* **C — ordinary control:** `arm=transformer layers=6 width=128 heads=4 mlp=matched match_pattern=rrarra match_read=l2 match_rotation=true`

`rrarra` is the frozen reference named by the commit's own test and doc
(`mqar-bench.rs:530-532`, `:2420-2432`), MLP 384, width 128, heads 4, reference
parameter count **1,370,008**.

> **Correction, 00:52, before any of my own runs completed.** An earlier draft of
> this paragraph said "2 quaternion-recurrence layers + 4 read layers". That is
> backwards: `rrarra` is **4 recurrence (`r`) layers + 2 read (`a`) layers**
> (`read_layers=2, recurrence_layers=4`, as the reports' own arm record prints).
> I recomputed `StackConfig::shapes()` by hand and reproduce the frozen
> 1,370,008 exactly (65,664 global + 4×218,176 recurrence layers + 2×215,820 read
> layers). This also corrects the V3 expectation below: the geometric arm has
> **2** context-reading layers, the control **6**, so the control scores 3× the
> positions per query token, not 1.5×.

**Pattern ambiguity, stated before running.** The *completed* Stage-0 arrangement
(`~/uor-r4-worktrees/reports/stage0-rank-bench-l2-rerun-20261006-000040`) used the
geometric arm `pattern=aaaaaa` (6 read layers, 1,360,584 parameters) — a different
pattern from the control's frozen reference. I chose `rrarra` for **both** arms
because (a) the task brief and the implementation's frozen test both name
`rrarra` as the reference to be matched, (b) the brief's invocation template uses
one shared `<P>` for both arms, and (c) the parameter-match number that D20 §2
condition 1 demands (1,370,008) is the `rrarra` number. The `aaaaaa` arrangement
is therefore **not** what is reported here; this is the single largest
config-interpretation risk in the run and is recorded as such.

## 2. Shared configuration (the Stage-0 arrangement)

Taken from `settings()` defaults (`mqar-bench.rs:1647-1691`) and from the argv of
the completed Stage-0 run, reproduced identically for both arms:

| key | value | source |
|---|---|---|
| `context` | 512 | Stage-0 argv; `settings()` default |
| `steps` | 1800 | Stage-0 argv; default |
| `batch` | 8 | Stage-0 argv; default |
| `pairs_per_bucket` | 8 | not in Stage-0 argv → default 8 (4 buckets × 8 = 32 pairs) |
| `final_sequences` | 64 | Stage-0 argv; default (32 held-out-class) |
| `eval_every` | 100 | Stage-0 argv; default |
| `curve_sequences` | 8 | Stage-0 argv; default |
| `lr` / `warmup` / `min_lr` | 1e-3 / 100 / 0.1 | Stage-0 argv; default |
| `weight_decay` / `clip` | 0.1 / 1.0 | Stage-0 argv; default |
| `device` / `layout` | cpu / synthetic | Stage-0 argv; default |
| `probe_steps` | `0,600,final` | `settings()` default (Stage-0 used the default) |
| seeds | 1, 2, 3 | ≥3 required by D20 §2 condition 2 |
| **`max_seconds`** | **3600** | **DEVIATION — see below** |

**`max_seconds` deviation, declared before running.** The default is 1200 and the
completed Stage-0 rerun used 2400; Stage-0's *first* attempt at the default did
not produce a report. `max_seconds` is a wall-clock *truncation* cap, not a
recipe parameter: if it binds, `steps_completed < 1800` while
`cosine_rate(..., s.steps, step)` (`:1863`) still anneals over the full 1800, so
the truncated arm is trained on a different schedule. I set 3600 so that the cap
cannot silently create unequal training budgets, and I verify per run that
`steps_completed == 1800` and `stopped_early_at_max_seconds == false`. The
machine is currently under load ≈ 54 from other labs' compiles, so this headroom
is deliberate. If a run still truncates it is reported as VOID for V1, not as a
loss.

## 3. Metric

**Primary:** `results.final_in_class_fresh_pairings.accuracy` — argmax over the
full 512-token vocabulary at every scored query position, on 64 **fresh** in-class
sequences (pairings drawn fresh from the training classes, never seen in
training). Identical panel for both arms.

**Secondary (reported, not thresholded):** per-bucket accuracy
(`d16,d64,d200,d400`), `final_held_out_class_pairings.accuracy` (adversarial
prior-vs-context class), training query NLL at step 1800, and the
data-only recency baseline (`mode=task-baselines`) for the same panel.

## 4. Decision thresholds (fixed before the numbers)

With `G_i`, `C_i` the primary accuracy at seed *i*:

* **Control wins:** `mean(C) − mean(G) > +0.05` **and** `min(C) > max(G)` (complete
  per-seed separation).
* **Geometric arm wins:** `mean(G) − mean(C) > +0.05` **and** `min(G) > max(C)`.
* **Tie:** `|mean(G) − mean(C)| ≤ 0.05`.
* **Inconclusive:** `|Δmean| > 0.05` but the per-seed ranges overlap.

0.05 is ~20 % relative to the 0.2554 in-class accuracy the Stage-0 geometric arm
reached, and is the smallest difference I am willing to call decision-relevant at
3 seeds. Per-seed values and the within-arm spread are reported regardless.

## 5. Validity conditions (VOID, not loss)

* **V1 equal training.** Every run: `steps_completed == 1800`,
  `stopped_early_at_max_seconds == false`, and equal
  `supervised_queries_seen` (= 1800 × 8 × 4 × 8 = 460,800). A run failing this is
  VOID.
* **V2 firing counter non-zero on scored positions, both arms.**
  * G: `read_firing.status == "MEASURED"` and
    `read_firing.rows_nonzero_mass_on_key > 0`. Otherwise G is VOID.
  * C: **`read_firing` is UNAVAILABLE for the control by construction** —
    verified in source before running: `StackArm::read_heads()` returns empty for
    `StackArch::Transformer` (`mqar-bench.rs:1003-1005`), `read_firing` then
    returns `UNAVAILABLE` (`:1168-1174`), because the library's binding capture is
    only populated on the geometric read path (`geometric_stack.rs:6333`).
    Condition 3 for C therefore rests on the arch-symmetric counter
    `context_reachability.rows_with_any_logit_change > 0` (same forward-pass key
    ablation for both arms). If that is 0, C is VOID.
  * This is a **pre-registered deviation from the task brief**, which expected a
    non-zero `read_firing` for the control. The brief's premise does not hold for
    this implementation; I do not treat UNAVAILABLE as a pass and I do not treat
    it as a zero.
* **V3 scored positions.** `context_access.positions_scored_per_query_token_mean`
  is recorded for both arms. It is *not* expected to be equal (G: 2 read layers ×
  4 heads × (t+1); C: 6 attention layers × 4 heads × (t+1) — measured on the
  control's smoke: mean over the window 6,156 and 12,288 at the last position,
  i.e. **3× the geometric arm's 2,052 / 4,096**). This is an asymmetry that
  **favours C** and is reported, not a voiding condition.

## 6. Asymmetries to check before concluding (both directions)

1. unequal training budget or data between arms (V1);
2. unequal scored positions (V3 — expected, favours C);
3. a control that is architecturally hobbled (inspect: RoPE, strict causal mask,
   SwiGLU, pre-norm are all present in C; C has *more* context-reading layers than
   G, so hobbling is not expected — checked, not assumed);
4. a geometric arm whose firing counter is zero (V2 — voids the comparison).

A result favouring G is treated as equally suspect as one favouring C.

## 7. Report roots (each NEW, exclusively claimed, never reused)

All under `~/uor-r4-worktrees/reports/`:

```
d20run-20261006-0102-geo-rrarra-s1   d20run-20261006-0102-ctl-rrarra6-s1
d20run-20261006-0102-geo-rrarra-s2   d20run-20261006-0102-ctl-rrarra6-s2
d20run-20261006-0102-geo-rrarra-s3   d20run-20261006-0102-ctl-rrarra6-s3
d20run-20261006-0102-baselines
```

Attempt 0 (00:46, roots `d20run-20261006-0050-*`) was refused in 0.02 s with
`AlreadyExists` because the driver created the directory before the binary's
exclusive `report_output::claim`. No model work ran (exit 1, 6.5 MB peak RSS).
Those two roots are consumed and are kept as-is; the run roots above are new.
Cost logs for every attempt live outside the sealed roots in
`~/uor-r4-worktrees/d20-run/cost/`.

## 8. Amendments (each made BEFORE the run it affects, with the reason)

**00:56 — I run exactly one root: the geometric seed 1 the parent's battery lost.**
The parent session's battery `d20-control-20261006-0110-battery` is running the
same pre-registered comparison sequentially in
`~/uor-r4-local/mqar-bench/d20-control-20261006-0110-battery/`. Its
`G-rrarra-s1` was SIGTERM'd after 9 s (`driver.log`: `rc=143`) and its `record()`
skips any root that already exists (`d20-control-battery.sh:19-21`), so that root
will never be produced by the battery. To avoid duplicating four other runs and
to keep this host's contention from voiding them, **this session runs only
`G-rrarra-s1`**, into a new root
`~/uor-r4-worktrees/reports/d20run-20261006-0056-geo-rrarra-s1`. The complete
primary comparison is therefore their 5 roots plus my 1.

Two deviations for that single run, both recorded:

* `probe_steps=none` instead of the Stage-0 default `0,600,final`, so that this
  run's configuration is **identical** to the five battery runs it is merged
  with. The read probe is observation-only (a separate forward pass that touches
  no parameter, no optimizer state and no training RNG — the training batch is
  reseeded per step from `(seed, step, batch)` at `mqar-bench.rs:1866`), and
  `prober.seconds` is excluded from the training cap (`:1858`). The cost is the
  secondary binding-mass probe at steps 0/600/1800; the condition-3 evidence
  (`read_firing`) is unaffected.
* `max_seconds=5400` instead of 3600. The host was at load 36-54 from other
  labs and 7.6/9.2 GiB of swap was in use when this was set; the cap is a
  truncation guard, not a recipe parameter, and raising it only prevents an
  unequal training budget. Measured at the time of launch, the control arm was
  running at 0.90 s/step (≈1,790 s for 1,800 steps), so neither cap binds in
  quiet conditions.
