---
name: experiment-prereg
description: Pre-register a run before any compute — bar, panel noise, base artifact, split, seeds/rows, steps, stop rule and complete cost — then obtain the TEST FITNESS review on the milestone.
version: 1
harness: 1
kind: research
applies-to: [docs/mechanisms/*.md, docs/harness/milestones.json, docs/labs/*/README.md]
when: [pre-register, pre-registration, TEST FITNESS, before compute, bar, stop rule]
requires: [cost-and-compute-admission, panel-scoring]
outputs: [one pre-registration sheet posted on the milestone issue, one TEST FITNESS review comment, a prep receipt in docs/harness/ledger]
forbidden: [compute before the FIT review, a single-row bar, a bar narrower than panel noise, bolting a mechanism onto a base that never trained with it, changing the bar after the result]
milestones: [m1, m2, m3, m4]
---

## When to use

Use before every model run, with no exception for a "quick" one. The gate is
D22 §3 (minimum test power) and §4 (transfer before KEEP); the requirement is in
AGENTS.md ("Every mechanism pre-registration gets a `TEST FITNESS: FIT` review on
its milestone before compute") and session-goal.md rule 4.

## Do this

1. **Name the mechanism and read its brief.** Every run tests one mechanism from
   [docs/mechanisms/](../../../mechanisms/README.md); read that page's "what a
   fair test needs" before writing the bar. A run that is not tied to a brief is
   an ordinary lever, and ordinary levers cannot be a novel line's decisive run.
2. **Draft the sheet** with all nine fields:
   `uor-harness prep new --milestone m1 --mechanism exact-addressed-memory --run "<one line>" --bar "<metric and threshold>" --seeds 2 --rows 0 --base "<artifact id>" --split "<train/eval split>" --steps "<count>" --stop "<rule>" --cost "<GPU-hours, pods, USD, local time, storage>"`.
   A missing field is a defect in the sheet, not a detail for the run log.
3. **Fill each field from a source, not from memory.**
   - **Bar:** one number and its unit, with the comparison it is against.
   - **Panel noise:** the frozen panel's seed-to-seed spread or a binomial
     standard error (the panel files in [docs/harness/panels/](../../panels/)
     record it per panel).
     The bar must be **wider** than that noise.
   - **Base artifact:** an existing, identity-bound artifact. Never a historical
     number; "the saved parent" is not an identity.
   - **Data split:** the exact rows/corpus and which of them are held out.
   - **Seeds/rows:** ≥ 2 seeds, or ≥ 200 fresh rows, or both (D22 §3).
   - **Steps and stop rule:** what stops the run, including the divergence and
     failure conditions — not a wall-clock guess.
   - **Cost:** complete preparation, build, fit, controls, evaluation, retries,
     checkpointing, delivery, wall time, threads, peak RAM, new and retained
     storage, charged to the cumulative ledger by `cost-and-compute-admission`.
4. **Run the fitness check yourself before you ask for the review. What makes a
   pre-registration NOT FIT:**
   - the bar equals or undercuts the panel's noise, or is a single-row difference
     (D22 §3: no single-row bars, no bar narrower than noise);
   - no noise figure is stated at all, so the bar cannot be judged;
   - the mechanism is bolted on to a base that never trained with it and the plan
     is a short add-on — a probe, not a decisive run;
   - the base is a historical number or an artifact with no bound identity;
   - the evaluation rows were used to choose the design, or there is no held-out
     split and no fresh-transfer plan for a KEEP (D22 §4);
   - the run cannot change a decision: whatever comes out, the next step is the same;
   - the configuration is already excluded by D23 (any softmax, log-softmax,
     implicit cross-entropy softmax or renamed equivalent) or D24 (CPU model
     compute, GPU work that cannot change a decision, or a baseline those rules
     forbid) — such a campaign is cancelled, not pre-registered;
   - the prior failures of this *configuration* are not addressed: a sheet that
     does not fix the recipe, dose, window, key, objective, base or steps that
     failed is a repeat;
   - the cost projection is incomplete or omits retries, evaluation or storage.
   Fix every one of these before posting, then post the sheet on the milestone
   issue and ask for the review as a comment on that same issue. Record the
   verdict explicitly:
   `uor-harness prep fit --milestone m1 --verdict FIT --sheet <id> --reviewer <name>`
   (a harness receipt; the review comment on the milestone is what actually gates
   the run).
5. **Only then run.** Model compute starts after the `TEST FITNESS: FIT` comment
   exists on the milestone. Before training, read the mechanism brief again for
   the property the run is supposed to move and how it will be observed.
6. **Report a change of plan as a new pre-registration**, not as an edit to the
   old one. The old sheet keeps its verdict and its scope.

## Output

- One pre-registration sheet comment on the milestone issue, with the nine fields
  above and the mechanism brief linked.
- One `TEST FITNESS: FIT` or `NOT FIT` review comment on the same issue, naming
  who reviewed it.
- On FIT: a `prep` receipt in `docs/harness/ledger/<YYYY-MM>.jsonl`.
- On NOT FIT: the corrected sheet, posted as a new pre-registration.

## Do not

- Do not compute before the FIT comment exists; a self-assessment is not a review.
- Do not widen or narrow the bar after seeing a result.
- Do not treat FIT as authorization: FIT is necessary, not sufficient, and the
  owner's compute holds and caps still apply.
- Do not keep two pre-registrations open on one line; retire the superseded one
  in the issue with a one-line reason.
- Do not restate the run in the result record instead of linking the sheet; a
  result whose bar cannot be traced back to a FIT sheet is a reading, not a result.
