# Pre-registration - M1 (MISSING)

- date: 2026-10-13
- milestone: M1 #2029 - Language base: a geometric stack language model good enough to carry chat
- mechanism: MISSING
- mechanism brief: MISSING
- run: MISSING
- line: exact addressed memory - count 0/3 (D21/D22 count is owned by the milestone, not this sheet)

## Bar (pre-registered)
- bar: MISSING - a run without a pre-registered bar cannot be scored
- panel: `v5-reply` (grader: BLOCKED (D28) - chat-grade by qwen2.5:7b is a NEURAL grader and D28 prohibits neural graders until the owner declares the native model works end to end. D28 names this grader specifically: "This includes the v5 reply panel's grade-replies qwen2.5:7b grader." The panel is scored with DETERMINISTIC checks only (membership checks, exact-answer oracles); a bar that can only be scored by a neural grader is recorded BLOCKED and is NOT RUN., frozen: yes)
- panel noise: unstated for the reply panel; on the 40-row memory panel +/-3 rows is the noise seen (docs/labs/handoff-deepseek-2026-10-12.md: 'Bars are never narrower than the panel's noise (+/-3 rows on the 40-row memory panel)'). A bar must not be narrower than its panel's noise.
  - reply-acceptable: acceptable replies on the open reply panel 37/232 -> 116/232 acceptable replies [open]
  - v5-memory-check-pass: v5 multi-turn memory panel check_pass 20/40 -> 34/40 check_pass rows [open]
- no bar in this sheet may be narrower than the panel noise above (D22 section 3).

## Base artifact
- base: D10 (10% recall dose, adopted under D22 5f)

## Data split
- split: MISSING
- training data never includes the frozen panel rows; scoring uses the frozen panel and, for a KEEP, a fresh held-out draw (D22 section 4)

## Steps
- steps: MISSING

## Minimum test power (D22 section 3)
- seeds: MISSING (>= 2 seeds for a decisive run)
- fresh rows: MISSING (>= 200 fresh rows for a decisive run)
- decisive run: NO - D22 section 3 needs >= 2 seeds OR >= 200 fresh rows (raise --seeds or --rows before compute)
- trained in rather than bolted on: MISSING - state the pretraining or the full fine-tune that gives the base time to adapt (a short add-on is a probe, not a decisive run)
- fresh-transfer plan for a KEEP: MISSING - every development KEEP also reports a fresh, held-out transfer number (D22 section 4)
- noise floor: the bars above must not be narrower than the panel's stated noise; a single-row bar is not a bar

## Cost (charged cumulatively, before execution)
- preparation: MISSING (context/data windows, threads, peak RAM)
- run: MISSING (wall time and steps; the whole job, not the first attempt)
- controls: MISSING (the no-mechanism control at the same recipe, steps and seeds)
- evaluation: MISSING (each checkpoint scored, plus the fresh held-out draw for a KEEP)
- retries: MISSING (diagnose, correct and retry within the remaining budget; a self-set budget is not a stop rule)
- storage: MISSING (temporary, retained, and the 128 MiB storage stop margin)
- placement (D24): no model compute is planned; CPU-bound preparation, grading and checks run on the owner's laptop, and no GPU is requested for work that cannot change a decision
- pod/lease plan: MISSING - name the pod, the GPUs leased, `--hours` set to the whole job plus margin, the renew cadence, and the release-then-down step
- owner caps: <= 4 running pods and <= $8.00/h across all labs, all sessions; no new spending class without an owner decision (AGENTS.md, docs/labs/compute.md section Caps)

## Stop rule
- stop: MISSING
- self-set step/wall budgets are estimates, not stop rules: extend and renew rather than cancel a healthy run; the only hard stops are the owner caps, the disk floor and a failed or diverged run

## Owner rules that apply
- D21: Three consecutive merged PRs on one line that do not move the milestone headline force a pivot card.
- D22: A negative closes a configuration, never a mechanism. Decisive runs use >=2 seeds or >=200 fresh rows, train the mechanism in, and carry a TEST FITNESS: FIT review on the milestone before compute.
- D23: No softmax anywhere in the active project: training, serving, sampling, objectives, auxiliary losses, teachers, comparators. No table/renamed equivalent. Interlock: `uor-harness ratchet d23|d25` (Rust, #2346) - `scripts/check_no_softmax.py` is retired and kept only as dated evidence; a pre-registration sheet that names the Python script sends a lab to a file that no longer enforces anything.
- D24: No CPU model compute; no GPU on work that cannot change a decision. A host-side fallback is a defect, not a cost to report.
- D25: Current geometric operator phase: OP2 first -> OP1 G2 only -> OP3a -> lawful rung0 after hold release -> OP4/rung0b; OP3b later research. Training and independent M2 rental hold remain. FIT lifts no hold; preserve peer work and frozen constraint set.
- D28: No transformer or neural-network comparisons of ANY kind - comparators, controls, baselines, teachers, neural graders - until the owner declares the native model works end to end; that declaration is the owner's, not a lab's. Panels are scored with DETERMINISTIC checks only (membership checks, exact-answer oracles); a bar scorable only by a neural grader is recorded BLOCKED and is NOT RUN. This includes the v5 reply panel's grade-replies qwen2.5:7b grader.
- D29: No banned mechanism may exist in any compiled or loaded module - presence counts, not only execution. NVRTC (runtime compilation) is banned project-wide; all CUDA kernels compile ahead of time via nvcc into named PTX/cubin modules. Interlock: `uor-harness ratchet d25` and the d29_compiled_modules test.
- D31: No result is ruled out on an instrument technicality. A check must answer the four authority questions in pm-rules.md R12 before it has veto power; when a check fails anyway, the instrument artefact is diagnosed and repaired rather than reported as a model negative. Supersedes older text that allows a first-failure stop or treats an instrument artefact as a code blocker.
- D32: Discrete geometric substrate: model computation AND its instruments, including learning, are discrete/exact and GPU-only - no floating-point forward, gradients, optimizer or numerical model instrument, and no matrix multiplication. Use native SpiralCore/E8/FBS geometry with one integer input/carry/output writer. Supersedes D25's offline scalar floating-point allowance, which is now inactive. Both training holds remain HELD; a FIT review releases neither.
- D22: a negative closes this configuration only, never the mechanism; stopping a mechanism needs the owner.

## Panel target (milestone registry)
- headline: M1: reply acceptable 37/232 -> target 116/232
- panel: v5-reply
- source: docs/harness/milestones.json (issue #2029)

## Verdict
TEST FITNESS: PENDING
