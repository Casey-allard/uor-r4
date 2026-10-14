# Pre-registration - M1 (discrete-rung0-completion)

- date: 2026-10-14
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: discrete-geometric-learning
- mechanism brief: docs/mechanisms/discrete-geometric-learning.md
- run: Rung 0 COMPLETION on the DISCRETE stack — finish the registered full coordinate pass
  whose design is FIT at `docs/harness/prep/m1-discrete-rung0-training-2026-10-13.md`
  (Amendment 2): resume seed 1 from the completed pilot's checkpoint 1,640 through batch
  16,404, then run seed 2 from initialization through 16,404. Bars (i)/(ii) and the guard are
  decided at 100% on both seeds, exactly as pre-registered. No mechanism, law, schedule,
  initialization, data or evaluator change.
- line: evaluation integrity - count 1/3 (D21/D22 count is owned by the milestone, not this
  sheet; no count is reset by this sheet)
- parent sheet: `m1-discrete-rung0-training-2026-10-13.md` (FIT (design), Amendment 2). This
  sheet registers the completion dose only; every bar, guard, data pin, evaluator and outcome
  decision below is carried over from the parent unchanged unless this sheet says otherwise.
- pilot evidence: `docs/labs/a2-pilot-2026-10-14/README.md` (merged #2449, `10578e4c9`);
  completion handoff
  [#2029 6162275408](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6162275408);
  PM acknowledgment
  [6162397952](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6162397952).

Training remains HELD by the owner; a FIT verdict on this sheet lifts no hold. The M1
reply-quality headline remains SUSPENDED under R14. D32 governs: no floating point anywhere
in model computation, learning or instruments.

## Why this run, and why this shape (pilot evidence read)

The A2 pilot (seed 1, batches 0..1,639, VectorBlock 0..1639, diagnostic-only) completed exit 0
with the receipt audit PASS. Its readings bound what the completion must and must not change:

1. **The learning law works mechanically in its native substrate.** 1,179/1,640 proposals had
   at least one candidate cost different from identity; 236 accepted strictly improving moves;
   DEV top-1 17→82, top-8 173→1,169, strict distance wins 105→149; distinct full vectors
   16→155; occupied writer cells 16→124. A completion dose is not a rescue: there is measured
   first learning to continue.
2. **The pilot slice is biased by schedule, not by mechanism.** 236 of the 410 visited token
   identities have zero masked-TRAIN count; the slice holds 1,440,407/2,425,925 masked targets
   (~59% of mass but the rarest tokens). The remaining 90% of the pass covers every frequent
   token the count reference scores with. The decisive bars live there; the pilot could not
   have decided them by construction. No schedule reorder is made here — the parent's
   frequency-order recommendation stays a named successor, because changing the schedule now
   would break comparability with the pilot's 236 accepted updates and confound attribution.
3. **The rank-vs-loss divergence is an instrument question, registered, not a verdict.**
   Token hinge sum improved 679,708→671,712 while address hinge sum worsened
   261,824→265,948; the aggregate numerator rose 13,816,032→13,884,032 while every rank
   measure improved. The completion records both hinge sums and the numerator at every
   checkpoint as diagnostics; R12 attribution order (instrument → fixture → bar → mechanism)
   applies before any mechanism language.
4. **The guard constant is now measured for seed 1.** Untrained strict-win rate
   u = 105/14,287 ≈ 0.735%, so the seed-1 guard threshold is max(1%, min(5u, u+5pp)) ≈ 3.67%
   (≈ 525 strict wins). Pilot end read 149 (≈ 1.04%) — above the 1% floor, below the guard,
   at 10% of the pass; diagnostic only. Seed 2's u is measured at its checkpoint 0 before any
   apply, per the parent.
5. **Initialization collisions stay.** The parent records <= 16 distinct full-vector patterns
   at initialization (xorshift low-4-bit root indices); learning separated 16→155 during the
   pilot. The collision-free integer init remains the named successor configuration (D22
   configuration change, its own sheet) and is NOT adopted here — adopting it now would
   discard the pilot's measured trajectory and restart attribution from zero.

## Scope (carried over and narrowed)

- **Seed 1: resume from checkpoint 1,640** (SHA256
  `46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440`, preserved in
  `icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar`, MD5
  `fb81eaa335a1d21ca1d05bde9771b3e3`) and complete batches 1,640..16,404 (14,764 batches).
  The 236 accepted pilot updates are part of the seed-1 trajectory; restarting seed 1 from
  initialization would waste a measured, admitted, paid-for prefix.
- **Seed 2: full pass from initialization**, batches 0..16,404, same deterministic schedule
  and corpus order as seed 1 (parent Amendment 1 item 2), `u` measured at its checkpoint 0.
- **Parameters:** VectorBlock 0..16,383 then Route 0..19 in schedule order. The pilot's Route
  row (attempted 0 / NOT_RUN) becomes a real sensitivity row when the pass reaches it.
  Gate (kind 3) and the flock read remain reserved (parent item 8).
- **Decisive checkpoints:** 0 / 4,101 / 8,202 / 12,303 / 16,404 per seed (parent Amendment 2
  item 3); bars, guard and any transfer decided at 16,404 only. Seed 1's checkpoint 1,640 is
  an extra diagnostic point already measured; intermediate checkpoints are never selected
  post hoc.

## Base artifact

- seed 1: the pilot's checkpoint 1,640 (SHA256
  `46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440`), resumed exactly —
  the admitted, receipt-audited seed-1 trajectory; no artifact inheritance from the F32
  trunk (D28 comparators only).
- seed 2: `Device::new(spec, seed=2, ordinal)` at the pinned spec — the same initializer the
  parent registers, including its documented <= 16-pattern collision limitation; no
  initialization change in this sheet.

## Bar (pre-registered) — carried over unchanged

- **Bar (i):** raw-score top-1 on the 14,287 pinned dev targets beats the count predictor by
  >= 5 percentage points **on both seeds** (reference top-1 8.58% → bar ≈ 13.6% ≈ 1,941/14,287).
- **Bar (ii):** recall@8 over the declared kept list beats the count predictor by >= 5
  percentage points **on both seeds** (reference 28.42% → bar ≈ 33.4% ≈ 4,775/14,287).
- **Reference:** the frozen masked-TRAIN count predictor, recomputed in-run by exact integer
  counting; never reads dev targets. Frozen-sheet verified numbers: 1,226 top-1 / 4,060 top-8
  on the same 14,287 targets; a recomputation mismatch is reported, not absorbed.
- **Guard (Amendment 2 form):** strict distance-win rate >= max(1%, min(5u, u+5pp)) on both
  seeds, u per seed at checkpoint 0 (seed 1: measured, 105/14,287 ≈ 0.735% → threshold
  ≈ 3.67%). u > 95% → INSTRUMENT-INVALID and bars decide alone; never a negative.
- **Noise:** Wilson 95% half-width < 0.83 points at n = 14,287; bars wider than noise.
- **Fresh transfer: UNAVAILABLE / NOT_RUN** (parent Amendment 2 item 2). A KEEP-with-transfer
  claim needs a prospective amendment binding source, sha256, token count and scored count
  >= 200 before it runs.
- Pilot-observed position for calibration (not a bar): seed-1 at 10% of the pass reads top-1
  82, top-8 1,169, strict wins 149 vs reference 1,226/4,060 — below bar with 90% of the pass,
  including every frequent token, unvisited. Nothing about the bar is re-based on this
  observation.

## Data split, dev policy, evaluator — carried over verbatim

As the parent sheet: train `mw-balp/train/` (7,512,797 tokens; 2,425,925 response-masked
targets; member shas pinned there), dev 42,856 payload tokens / 14,287 eligible targets,
tokenizer sha256 `d36d3e87…`, context 192, one target per window, 224 dev batches, scored set
window-independent, BOS-segmented batches, wholly-masked batch refuses. Spec V4096/B4/C192/K8,
batch 64, schedule `canonical_vector_then_read_v1`, bos `<|bos|>`/0. D3–D32 owner rules apply
as listed in the parent (D21/D22/D23/D24/D26/D28/D29/D31/D32, R12/R13/R14).

## Steps

1. Codex admission diff v2 (dependency D5 below): dose extension beyond the pilot cap
   (`ValidatedRelease::max_batches` is the pilot's 1,640 today), resume-from-checkpoint-1640
   admission for seed 1, seed-2 fresh initialization under the same schedule, and a new
   OwnerReleasePlan scope for the completion. No other code change; any other change voids
   this sheet's FIT, per the parent's admission rule.
2. Claude reviews that exact diff against this sheet.
3. Kimi binds the final executable/PTX/source/config/lineage identities in a completion
   OwnerReleasePlan (same packet discipline as
   `docs/labs/rung0-pilot-release-2026-10-14/`).
4. Owner release (D4 or its delegated equivalent, owner decision) for the completion scope.
5. Seed 1 resumes on the admitted pod; checkpoints at 4,101/8,202/12,303 scored; 16,404
   scored decisively; seed-1 outcome recorded.
6. Seed 2 runs the full pass on the same pod/lease discipline; checkpoints likewise; 16,404
   scored decisively.
7. P1/P2-style reports at every checkpoint: per-kind sensitivity (attempted / any-candidate /
   accepted), distinct full-vector patterns and occupied writer cells, token/address hinge
   sums and the aggregate numerator, count-reference recomputation check.

## Minimum test power (D22 §3)

- seeds: 1 (completion) and 2 (full), paired — satisfies >= 2 seeds.
- fresh rows: 14,287 scored dev targets (>= 200); per-seed dose 1,049,856 eligible targets.
- decisive run: YES at 100% on both seeds — the parent's decisive experiment, completed.
- trained in rather than bolted on: the carrier itself trains; no bolted-on head.
- noise floor: bars 5pp vs < 0.83pp noise.

## Cost (charged cumulatively, before execution)

Measured on the pilot (replaces the parent's UNVERIFIED projection with pilot-measured rates;
still marked estimate, not invoice):

- seed-1 remainder: 14,764 batches at the pilot's measured 3.7457 s/batch average
  (6,143.011 s / 1,640 — a controller wall that already includes admission, evals and I/O,
  so a conservative per-batch figure) ≈ **15.36 h ≈ $13.70** at $0.892/h, plus dev evals
  (~19.5 s each × 3 quarter + 1 final per seed), checkpoints and preservation.
- seed 2 full pass: 16,404 batches ≈ **17.07 h ≈ $15.23**, plus its untrained eval, quarter
  evals, count-reference construction, checkpoints.
- controls: none separate — count predictor and untrained constants are in-run (parent).
- combined: ≈ **32.4 h GPU ≈ $29**, one RTX 5090 pod, one job per GPU, lease discipline with
  renewals per the stop rule; inside owner caps (<= 4 pods, <= $8/h). Setup/build,
  preservation (~6 MB archives), iCloud round-trips and the release round ride the same
  lease; retries inside the same lease.
- preparation: laptop source/sheet work only; no GPU. storage: checkpoints/reports ~ MB;
  iCloud `cloud-store put` with MD5 round-trip; 128 MiB stop margin.

## Stop rule

Owner hold release is the start gate. Hard stops: owner caps, disk floor, a failed or
diverged run. Self-set budgets are estimates — extend and renew rather than cancel a healthy
run; record the extension on #2029. A seed-1 100% reading that is far below every bar and
guard is recorded and reported before seed 2 launches; it is iteration evidence, not a stop
and not a negative, and it does not silently change the registered two-seed design — the
owner decides whether seed 2 proceeds on the seed-1 record.

## Dependencies (must land before the FIT verdict)

- **D5 — completion admission diff (owner: Codex).** Dose cap extension, resume admission
  from checkpoint 1,640, seed-2 initialization path under the same schedule, new release
  scope. Reviewed by Claude against this sheet; identities recorded before the first apply.
- **D6 — owner release decision for the completion scope.** Not a sheet dependency for FIT;
  required for execution only. The owner also decides the dose-sequencing question in the
  Stop rule (two-seed continuous vs seed-1-then-decide) at release time.

## Outcome decisions (carried over, restated for the completion)

- **KEEP:** bars (i)/(ii) and the guard on both seeds → report fresh transfer per the
  transfer clause (prospective amendment first) and open the successor-dose question
  (rung 0b / named successor configurations).
- **Iterate:** movement above untrained but below bar → same-configuration dose/schedule
  change only; the pilot's sensitivity and coverage readings guide which.
- **Blocker:** instrument or arithmetic fault → repair and re-run (D31); never a negative.
- **No automatic closure:** R12 attribution order governs; a below-bar result closes this
  configuration only after the attribution chain, and stopping the mechanism needs the owner
  (D22). Named successors (collision-free init, frequency-ordered schedule, stage-indexed
  BRANCH generators, orbit-indexed counterfactuals, flock read capacity) stay recorded, not
  adopted.

## Review (R12's four questions, pre-answered)

1. **What it measures:** whether the discrete enumerated-move law, given its registered full
   dose, beats a train-split frequency reference by 5pp on exact integer rank measures — the
   decisive form of the question the pilot could only sample.
2. **Computable now:** yes — every measure is an exact integer comparison; the count
   reference is known; the evaluator is qualified; the pilot demonstrated the full pipeline
   end to end with a receipt-audited PASS.
3. **Can the instrument beat the bar in principle:** yes — the frequent-token mass the count
   reference scores with is exactly the mass the remaining 90% of the pass trains on; the
   pilot's 4–6× rank improvement came from the 10% rarest tokens. If the full dose cannot
   beat a frequency table by 5pp, that is a real configuration finding (never a mechanism
   negative, D22).
4. **Should it run at this stage:** yes — the pilot consumed ~5% of the registered cost to
   verify the pipeline, the law's mechanical function, the schedule bias and the guard
   constant; the remaining uncertainty is precisely the one the registered experiment was
   designed to decide. Not running it leaves the registered question permanently open after
   its cheapest check passed.

## Verdict

TEST FITNESS: PENDING — requested from Claude lab, with Codex source check on D5 when posted.
