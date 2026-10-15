# Pre-registration - M1 (discrete-rung0-admission)

- date: 2026-10-15
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: discrete geometric stack — writer-admission width/policy (the B3 attribution line of the rung-0 completion sheets)
- mechanism brief: docs/mechanisms/discrete-geometric-learning.md
- run: **Stage A** — zero-update kept-cells (K) sweep ceiling instrument over the preserved seed-1 quarter checkpoints (evaluation only, no training). **Stage B** — successor training run at a K shown reachable by Stage A (registered here; HELD; no FIT requested now).
- line: starling read / exact addressed memory — count 0/3 (D21/D22 count is owned by the milestone, not this sheet)
- owner decision: #2029 comment 6187197075 (2026-10-15 ~07:55Z): seed-2 as registered is HELD; the rung-0 line iterates the admission objective; **the admitted-row ceiling must exceed the decisive bar counts before any run is released**; bars never narrower than panel noise (D22 §3).

## Why this instrument, and why now (seed-1 evidence read)

Seed-1 completed rc=0 at 16,404 batches (first attempt, 15.46 h), PM-verified EXACT (#2029 6186067850): top-1 **149** (bar 1,941), top-8 **747** (bar 4,775), guard strict wins **105** (bar ≈525), admitted strict 65, **target-admitted rows 1,920**, B3 nesting exact (65 ≤ 149 ≤ 1,920), `instrument_invalid: false`. The bars were **arithmetically unreachable**: `discrete_evaluate` ranks only over admitted cells (`discrete.cu:371-381`), so top-1 and top-8 ≤ target-admitted rows, and 1,920 < 1,941.

The ceiling fell as the codebook trained: 5,680 (baseline) → 2,180 (pilot end) → 2,201 / 2,131 / 1,954 / 1,920 (Q1–Q4), while occupancy diversified from ≤16 to 189 occupied cells; the 8-slot admission budget was fully used on every row (114,296 = 14,287 × 8) at every checkpoint. The final-quarter Route moves raised top-1 96→149 and admitted strict 44→65 while *lowering* admitted rows 1,954→1,920 — the address-objective trade-off the completion sheet's B3 clause names (m1-discrete-rung0-completion-2026-10-14.md:312-317). Route learning was observed for the first time (16/20 slots accepted). Seed-1 is iteration evidence, not a mechanism negative (D22).

The only pure config admission knob is `spec.kept_cells` (K=8; valid 1..=16, ABI buffers exactly 16 — `lib.rs:49-51`, `discrete.cu:234/365`, `device.rs:862`). No admission threshold, budget, rate or schedule exists anywhere in `DriverConfig`/`SchedulePolicy`. Whether any K ≤ 16 admits enough target cells at a trained codebook is **not computable from sealed receipts** (rows never expose the target cell's address rank): bounded 1,920..14,287, and near-uniform address ranks at Q4 occupancy (16 of ~189 cells) would give ≈1,210 even at K=16. The owner's gate therefore requires a **measurement**, and that measurement is minutes of GPU — it must precede any ~15.5 h / ~$14/seed training decision.

## Scope

- **Stage A (this FIT request):** the K-sweep ceiling instrument — evaluation-only, zero updates, on preserved seed-1 artifacts.
- **Stage B (registered, HELD, no FIT requested):** the successor training run at the smallest K that passes the Stage-A gate, fresh lineage (K is spec-bound; `validate_checkpoint`/`admit_model` refuse a K≠8 resume — `training.rs:324-325, 729-733`), guard re-based at its own checkpoint 0. Stage B gets its own amendment and Claude FIT after Stage A lands.
- The B2 schedule successor (second pass / frequency-ordered revisit) stays **recorded, not adopted**: no schedule change can lift the hard admission ceiling, so the ceiling gates first. Named policy-level successors (address metric, selection rule, 8·token+32·address objective weights) stay recorded, not adopted — they are the redirect if Stage A closes the width family.
- Nothing in this sheet changes the frozen panel, reference, bars, guard form, dose, or schedule of the rung-0 completion line.

## Stage A design

- **Instrument:** evaluation-only re-score of the four preserved seed-1 quarter checkpoints at `kept_cells` K ∈ {8, 9, …, 16} over the frozen 14,287-row dev panel (224 batches per (checkpoint, K) pair; 36 evaluations). Optional additional points, not gates: the pilot lineage's checkpoint 0 if accessible in the pilot artifact tree (the untrained ceiling's K-slope next to the trained ones), and checkpoint 1,640 if its sha matches the pilot pin (`46e8ed5e…`) at instrument-build time.
- **Zero updates:** no training, no checkpoint mutation; artifacts are read-only inputs; outputs are new derived evaluation records written into their own claimed report directory (never beneath a sealed root; `report_output::claim` discipline).
- **Artifact pins (pod `/workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1/`, archive `rung0-completion-seed1-20261015.tgz` sha256 `300246ab16a1e0adaba547d43590e08264912f59ee29f35d7e024d77591e63a1`):**
  - checkpoint-04101.json sha256 `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd`
  - checkpoint-08202.json sha256 `dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822`
  - checkpoint-12303.json sha256 `102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982`
  - checkpoint-16404.json sha256 `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` (PM-verified against the pod record and the evaluation pin at Q4)
  - evaluation-16404.json sha256 `4fff1d3c…` + `.pin.json` (full shas in the sealed verification comments)
- **Lawful device qualification (stored expected values, no CPU model reference):** at K=8 the instrument MUST reproduce the sealed quarter metrics exactly — Q4: top-1 149, top-8 747, strict wins 105, admitted strict 65, target-admitted rows 1,920; Q1–Q3 per the sealed verification comments (6174365494, 6178914118, 6182703265). Any mismatch is an instrument fault → diagnose and repair (D31), never a finding.
- **Code surface:** one evaluation-only admission diff — a kept-cells override in the evaluation path producing a derived-instrument binary. `spec.kept_cells` lives inside the sealed snapshot (`device.rs:271-287`), so the diff must construct the eval device with the override explicitly; **no training-path change**; the diff states its `driver_source_sha256` impact (any change in `corpus.rs/training.rs/controller.rs/completion.rs/completion_baseline.rs/main.rs` voids existing pins and must be avoided or disclosed). The seed-2 checkpoint-zero guard's `admitted_cells ∈ 1..=8` assertion (`completion_baseline.rs:70`) is out of Stage A's path (no baseline run); it is a Stage-B dependency.
- **Measures per (checkpoint, K), all exact integers from the same kernel evaluator:** admitted-cell rows, address-rival rows, target-admitted rows (the ceiling), top-1, top-8, strict wins, admitted-strict wins.

## Bar (pre-registered) — the reachability gate

- **Gate G-A:** at checkpoint 16,404, at least one K ∈ {9..16} shows target-admitted rows ≥ **4,894** (bar (ii)'s count 4,775 + one Wilson half-width 119 ≈ 0.83pp at n = 14,287), and the ceiling trajectory across 4,101 → 8,202 → 12,303 → 16,404 at that K does not end below 4,894.
- **G-A PASS →** the smallest passing K becomes the Stage-B candidate; this sheet is amended with the Stage-B training pre-registration (fresh lineage, re-based guard, revised `1..=8`→`1..=16` guard assertion, cost) and goes to Claude for TEST FITNESS. Training stays HELD until that amendment is FIT and the owner releases.
- **G-A FAIL (no K reaches 4,894 at 16,404):** closes **eval-time widening of the K=8-trained codebook** — a configuration closure under D22, never a mechanism negative. The sweep re-scores codebooks *trained at K=8*; a K>8-trained lineage sees a different objective (the address hinge ranks rivals among the K admitted cells), so its ceiling is not bounded by this instrument. **K>8-trained lineages remain an untested configuration, recorded as a named successor** alongside the admission policy/objective redirect (address metric, selection rule, objective weights), as the completion sheet's B3 clause names; a new sheet is written; **no training runs under this sheet**.
- **Diagnostic, not a bar:** top-1/top-8/strict-win curves over K per checkpoint, and the guard-direction interaction (best rival distance is a min over admitted cells, so strict wins can typically only fall as K rises — though a zero-rival row at K can gain its first rival at K+1 and become a strict win; report the count change) — reported, not scored.
- **Stage-B bars (carried over unchanged, registered for the amendment):** top-1 ≥ 1,941 and top-8 ≥ 4,775 vs the frozen masked-TRAIN count reference (1,226 / 4,060), guard ≥ max(1%, min(5u, u+5pp)) with u re-measured at the new configuration's checkpoint 0 (never reused; `completion_baseline.rs:279`).
- **Noise:** Wilson 95% half-width < 0.83pp (≈119 rows) at n = 14,287; 2-SE row floors 20 (top-1) / 63 (top-8) / 87 (B3). Bars and the G-A margin are ≥ 4× the noise. No measured seed-to-seed spread exists on this line (seed 2 never ran); none is claimed.
- panel: the rung-0 completion line's frozen 14,287-row dev copy-successor panel (grader: exact integer kernel evaluator + independent `completion-scoring-audit`, frozen: yes) — NOT the milestone default v5-reply panel; carried over verbatim from the parent sheets.
- no bar in this sheet may be narrower than the panel noise above (D22 section 3).

## Base artifact

- The preserved seed-1 lineage artifacts pinned above (checkpoints, evaluation + pin, archive tgz). No artifact inheritance from any F32 trunk (D28 comparators only). Stage B's base is `Device::new(spec-with-chosen-K, seed, ordinal)` at fresh initialization — no resume from the K=8 lineage.

## Data split

- split: carried over verbatim from the parent sheet — train `mw-balp/train/` (7,512,797 tokens; 2,425,925 response-masked targets; member shas pinned there), dev 42,856 payload tokens / 14,287 eligible targets, tokenizer sha256 `d36d3e87…`, context 192, one target per window, 224 dev batches, BOS-segmented, wholly-masked batch refuses.
- training data never includes the frozen panel rows; Stage A reads only the frozen panel and the preserved artifacts; for a Stage-B KEEP, a fresh held-out draw (D22 section 4) is bound by prospective amendment before it runs.

## Steps

1. Codex implements the evaluation-only K-sweep diff (dependency D-A below). No other code change; any other change voids this sheet's FIT, per the line's admission rule.
2. Claude reviews that exact diff against this sheet (TEST FITNESS seat).
3. Kimi binds the instrument binary/PTX/source/artifact identities in an instrument packet (same packet discipline as `docs/labs/rung0-pilot-release-2026-10-14/`).
4. Instrument run on a leased RTX 5090 (minutes; one GPU, one job, lease/release per `scripts/pod/uor-pod` discipline).
5. Kimi publishes the sweep table and the G-A verdict on #2029 with the derived records in a canonical lab record directory.
6. Decision per the gate above.

## Minimum test power (D22 section 3)

- seeds: N/A for Stage A (no training; an exact integer re-score of preserved artifacts). Stage B (amendment): the seed plan is registered at amendment time with the two-seed vs one-then-decide question put to the owner at release, as on the completion line.
- fresh rows: Stage A scores all 14,287 frozen panel rows per (checkpoint, K) — exact full-panel measurement, ≥ 200 rows, no sampling.
- decisive run: Stage A is decisive for its gate (full-panel exact integers at 36 points + a stored-expected-value qualification at K=8). Stage B trains the mechanism in at the full registered dose (16,404 batches) — no bolt-on probe.
- trained in rather than bolted on: Stage B is a full fresh-lineage dose at the chosen K; the K change is inside the training objective's candidate set (`discrete.cu:281-302`), not an eval-time wrapper.
- fresh-transfer plan for a KEEP: UNAVAILABLE / NOT_RUN at this sheet; a Stage-B KEEP-with-transfer claim needs a prospective amendment binding source, sha256, token count and scored count ≥ 2,000 before it runs (D22 §4).
- noise floor: the G-A margin (119 rows) and both Stage-B bars (715 rows) are ≥ 4× the stated noise; a single-row bar is not a bar.

## Cost (charged cumulatively, before execution)

- preparation: this sheet + the D-A diff (laptop, CPU-only; context/data windows none beyond the repo; threads ≤ build default; peak RAM < 8 GiB).
- run: Stage A instrument — 36 full-panel evaluations on one RTX 5090, estimated < 15 minutes GPU (< $0.25 at $0.908/h); lease `--hours 1` with margin, release + down immediately after. Stage B (contingent, not requested): ~15.5 h / ~$14 per seed at the seed-1 rate.
- controls: the K=8 stored-expected-value reproduction at each checkpoint IS the instrument control (exact equality, stored values, no CPU reference). No no-mechanism control applies to a ceiling measurement.
- evaluation: included in the run; derived records published with the sweep table.
- retries: instrument faults are diagnosed and repaired within the lease (D31); a self-set budget is not a stop rule.
- storage: < 50 MiB derived records; 128 MiB stop margin respected; artifacts stay on the pod volume/iCloud per the storage rules.
- placement (D24): the instrument is GPU evaluation of the native kernel evaluator — no CPU model compute, no host fallback. Sheet/diff/review work is laptop CPU.
- pod/lease plan: one RTX 5090 (existing pod if Claude's lease survives, else `uor-pod up` within caps), GPU 0, `--hours 1`, release-then-down after the sweep and artifact upload.
- owner caps: ≤ 4 running pods and ≤ $8.00/h across all labs; no new spending class.

## Stop rule

- stop: Stage A stops at the published sweep table + G-A verdict (there is no training to stop). Stage B (amendment) carries the completion line's stop rule: the only hard stops are the owner caps, the disk floor and a failed or diverged run; self-set step/wall budgets are estimates, not stop rules.

## Owner rules that apply

- D21: three consecutive merged PRs on one line that do not move the milestone headline force a pivot card.
- D22: a negative closes a configuration, never a mechanism; stopping a mechanism needs the owner. Decisive runs use ≥ 2 seeds or ≥ 200 fresh rows, train the mechanism in, and carry a TEST FITNESS: FIT review before compute.
- D23: no softmax anywhere; interlock `uor-harness ratchet d23|d25`.
- D24: no CPU model compute; no GPU on work that cannot change a decision. This instrument decides a ~$14/seed training question for < $0.25 — it exists precisely because it changes the decision.
- D25: operator-phase order unchanged; FIT lifts no hold.
- D28: no transformer or neural-network comparisons of any kind; deterministic checks only.
- D29: no banned mechanism in any compiled or loaded module; AOT nvcc PTX only; no NVRTC.
- D31: no result ruled out on an instrument technicality; instrument faults are repaired, not reported as model negatives.
- D32: discrete/exact GPU-only substrate; no floating-point forward/gradients; no matrix multiplication.

## Dependencies (must land before the FIT verdict)

- **D-A — the evaluation-only K-sweep diff (owner: Codex).** kept-cells override in the eval path, derived-instrument binary, disclosed `driver_source_sha256` impact, no training-path change. Reviewed by Claude against this sheet; identities recorded before the run.
- **D-B — artifact availability.** The four pinned checkpoints accessible on the pod volume or restored from the sealed archive (`300246ab…`, HF store / iCloud).
- **D-C — owner go for the instrument run.** Stage A is evaluation, not training: the training HOLD is untouched, but the pod minutes are a (sub-$1) spend the owner approves after FIT lands. Stage B requires the owner's explicit release separately.

## Outcome decisions

- **G-A PASS →** Stage-B amendment at the smallest passing K (Claude FIT, then owner release; training HELD until both).
- **G-A FAIL →** closes eval-time widening of the K=8-trained codebook (D22 configuration closure, never a mechanism negative); K>8-trained lineages stay open as a named successor; redirect sheet for the admission policy/objective; no training under this sheet.
- **Instrument fault →** diagnose and repair (D31); never a finding.
- **No automatic closure:** R12 attribution order governs; stopping the mechanism needs the owner. Named successors (address metric, selection rule, objective re-weighting, collision-free init, frequency-ordered schedule) stay recorded, not adopted.

## Review (R12's four questions, pre-answered)

1. **What it measures:** whether writer-admission width can admit enough target cells at a trained codebook for the registered rank bars to be reachable — the exact gate the owner set on 2026-10-15 (6187197075).
2. **Computable now:** yes — exact integer re-score on preserved artifacts with a qualified evaluator, stored K=8 expected values, and full-panel coverage.
3. **Can the instrument answer in principle:** yes — admission is monotone in K for a fixed codebook, and the sweep spans the ABI-maximum range (8..16) at four training states; the answer it cannot give (a fresh lineage's trajectory at K>8) is exactly what Stage B would measure, and Stage B only runs if the ceiling exists to reach.
4. **Should it run at this stage:** yes — it is the cheapest decisive check (< $0.25, minutes) standing in front of a ~$14/seed training decision; seed-1 already proved that skipping this gate spends the training budget on an arithmetically impossible configuration.

## Verdict

TEST FITNESS: FIT (design, Stage A) — Claude lab, #2029 comments 6192409142 (FIT conditional R1) and 6192687624 (R1 met, FIT confirmed @ `ba42b8c0f`, sheet byte-identical at merge head). FIT releases nothing by itself: Codex's D-A diff still requires Claude exact-source review before the owner-GO'd instrument runs; Stage B has no FIT; training and M2 stay HELD.
