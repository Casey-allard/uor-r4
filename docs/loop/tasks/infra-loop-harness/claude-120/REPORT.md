# REPORT: infra-loop-harness, claude-120 (release `d11-exporter-float-trunk`)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2820.

## What did not work, first

- **The hold reason is still literally true.** No two-seed Stage 3 float-trunk checkpoint exists. None of the four done float sheets saved a model, although `StackModel::save` and `load` exist and only a writer test calls them. The release rests on a different fact: producing the checkpoints, and everything else left, is now the lab's own work.
- **The merged exporter does not fit the trained trunks.** antigravity-75's exporter (#3221) takes dense mixer, MLP, read and head matrices. The Stage 3 sheets trained SpiralCore/FBS-projection recurrence trunks with the Hamilton gate and the native vector writer head. Mapping one onto the other would break CHARTER §3.3 and §3.6. So the item's remaining work is larger than "run Stage 4": it needs an FBS and writer export path first.
- **The Stage 4 sheet FIT'd for antigravity-75 does not carry over.** It assumes a flock read and a `rec_out` planted fault, which an FBS trunk lacks, and its band must be measured again. A new sheet and a fresh FIT come before any GPU run.
- **The decision review overstated one fact.** It said only `Hamilton_IcosianHard` passed its export gap; the Hamilton record says all its arms did. The item text and the owner log use the record's wording.
- **The scope grew beyond the ORDER's text.** The ORDER named only `crates/uor-r4-float-train/**`. The D68 review also required the item's own prep, lab, task and notebook paths, as its sibling items carry. The owner log (#2820 comment `6406906219`) approves all five.
- **`loop next` gave no GO.** Every ready item was claimed or overlapped antigravity's `crates/uor-r4-discrete/**` claim (#3239), and held items count only as a reason to wait. The owner directed this unit; the gap is logged in `local/claude/FRICTION.md`.

## What changed

Size: docs 1 file, generated 1 file (+35/−9 together), plus this task folder. The ORDER gave no estimate.

- **`docs/roadmap/items/d11-exporter-float-trunk.md`:**
  - `status: held` → `ready`.
  - `needs_gpu: false` → `true`, since the retrain and the panel scoring are GPU work.
  - The scope adds:
    - `crates/uor-r4-float-train/**`;
    - `docs/harness/prep/d11-exporter-float-trunk*`;
    - `docs/labs/d11-exporter-float-trunk/**`;
    - `docs/loop/tasks/d11-exporter-float-trunk/**`;
    - `docs/research/notebook/m4-serving.md`.
  - A dated "What is missing" records the three remaining gaps, with file:line evidence and the prior art to reuse: no saved checkpoint, no FBS/writer export path, no GPU evaluator for the container.
  - "Done when" names the trunk Stage 3 trains.
  - Step 4 replaces the retired typed owner release with the D67 pre-run review and a fresh FIT, and says the old FIT does not carry over.
  - "Rules that apply" warns: no dense materialization of FBS routes, no full-vocabulary head scan, and a one-way float-train → integer dependency.
- **`docs/roadmap/README.md`:** re-rendered (`xtask roadmap render`).
- **The decision review record** is the D68 owner log on #2820, comment `6406906219`.

## Measurements

None. This is a status and scope decision. The review's evidence was:
- the store listings of the four Stage 3 folders, with no model file;
- `crates/uor-r4-integer/src/stack/format.rs:690-772`, and the absence of FBS or writer code in `stack/format.rs` and `stack/session.rs`;
- `crates/uor-r4-float-train/src/hamilton_sheet.rs:1640-1658`;
- `geometric_stack_native.rs:402`, `:1293` and `:1337`.

## Checks run at the head

See the exact-head review record on the PR: `xtask roadmap check` (93 items OK), `loop report --check` and `loop pr-check --local`.

## Skills used

- **holding-the-goal.** The cancel probe and the review each tested the premise that the hold was gone before anything changed, because claude-113 once released an item on a false premise.

## Still open

- **The exporter work itself.** The next claude unit claims `d11-exporter-float-trunk` once this merges. It needs:
  - the FBS `(atom, action)` projection and vector-writer head in the export and integer runtime;
  - a converter in `crates/uor-r4-float-train` that saves a model and converts it, depending on the integer crate and never the reverse;
  - finite CPU controls;
  - then a new Stage 4 sheet with a fresh FIT, a D67 pre-run review, and a cheap two-seed retrain with saving.
- **`loop next` should offer held work** whose dependencies are done as "GO after your own pre-run review". It now counts it only as a reason to wait (FRICTION line).

## Plain summary

The project's float-training path ends at a trained model it cannot yet serve. This roadmap item is the missing step: the exporter that turns a float-trained model into the whole-number file the product runs. Another lab built a first exporter, then parked the item until earlier training runs produced saved models. Those runs are now finished.

A fresh reviewer checked the whole chain. None of those runs saved a model. The first exporter also expects an older model layout, with dense weight matrices, which the project's rules forbid at serving time. The newer models instead route each output through a small geometric lookup. Nothing outside this lab blocks the remaining work, so this change marks the item ready, widens its scope to the float-training crate, and writes down exactly what is left.
