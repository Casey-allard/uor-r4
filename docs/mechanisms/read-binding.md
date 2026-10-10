# Read binding (supervised value binding in the geometric read)
**The idea (first principles):** The geometric read is the model's native context access. It has scores, admission, age and a NoRead slot over R4-state sources. Read binding adds a teacher-only training signal: for a query whose next token belongs to a stated value, tell the read *which positions hold the asked value* (`bound`) and *which hold the other stated values* (`competing`). The loss is `-log(mass_on_bound + 1e-6)` on the best-binding head (`READ_SUPERVISION_FLOOR`). It rides on the read's exact scores, and the labels never touch a hidden state or logit. The read is then taught to bind *this* value to *this* question, which is the step that addressed memory needs. Phase binding (Step 7d/9, #820; `ReadLineage`) is the geometric route to the same goal: values carry a fixed phase or lineage, so binding becomes exact rather than learned similarity.
**What it replaces and why that matters:** It replaces hoping that next-token loss alone teaches the read to separate one stated value from another. Measured on M1: the remaining memory failures are mostly wrong-value failures. The read finds *a* value, but the wrong one.
**Where it lives in code:** `crates/uor-r4-training/src/geometric_stack.rs`:
- `ReadBinding` / `ReadBindingTarget` (exact source label for one layer and head)
- `ReadSupervisionGroup {bound, competing}`, `ReadSupervisionRow`, `ReadSupervisionTarget` (many queries, every head)
- `StackModel::read_supervised_loss` → `ReadSupervisedLoss {language, binding, bound, competing, heads}`
- the training flag `read_binding_supervision=W`, and `read_lineage` / `ReadLineage::RandomSo4`

**What has been tried, honestly:** #2155 fine-tuned one 29M chat artifact on generated dialogues.
- Own objective reached: bound mass 0.73→0.90 at W=0.1, and 0.93 at W=0.5.
- Headline flat: v5 memory 20→19/40 at W=0.1 and 17/40 at W=0.5. Reply panel 37→39/232.
- Wrong-value failures (the targeted failure mode): **17 → 12**, with nine rows repaired and four broken, row-paired against the anchor.
- It was counted as a negative under D21, although it met its own target. Under [D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened) it is reopened, with power.

That is one seed and a 40-row panel, so ±2–3 rows is noise. It was bolted on late, and the supervised head may not be the head the output actually uses. It shows the read *can* be steered. It does not show that steering it moves answers.
**What success looks like:** Own: bound mass rises, competing mass falls, and wrong-value failures drop. Headline: M1 #2029 v5 memory `check_pass` and reply `acceptable`, with no BPB regression.
**What failure looks like:** Bound mass near 1 on the head that causally feeds the output (shown by ablation), and still no memory gain over a matched control across seeds. That would mean correct binding is not the bottleneck.
**A fair test:**
- Supervise from the start of chat training, not as a late fine-tune.
- ≥3 seeds, ≥120 memory rows, a W=0 matched control.
- Report bound/competing mass on held-out rows, plus a head-ablation check that the bound head drives the answer.
- Bar: +5/40-equivalent over the control.

**Open design questions:**
- Supervise every head, or route the output through the binding head?
- Pair this with the pointer, so the bound positions feed the copy distribution directly?
- Phase or lineage tagging of values (exact binding) instead of learned mass?
- A competing-mass margin loss, rather than only maximizing bound mass?
