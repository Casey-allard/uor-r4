# REPORT: end-to-end-integration, claude-394 (ADR-0007 brought in line with the charter, and I1 pointed at D86 step 6)

Lab: claude. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md). ORDER: [ORDER.md](ORDER.md).

## What did not work, first

1. **The owner direction the ADR cites could not be found in the records.** "The owner's direction of 2026-11-15" (e-means, CIRISKernel) appears only in codex-1's ORDER and REPORT, the ADR and the item. PLAN.md's owner directions and DECISIONS.md have no row for it. This unit does not record it, because the words are in Codex's chat, not Claude's. The amendment marks it as not yet recorded.

   The ORDER's stop condition was a record that releases float training, RoPE or a cross-entropy loss. None was found, and the direction as the ADR quotes it ("replace matmul and the slow parts of softmax") releases none of them.
2. **This is a docs-only unit, so no reply moves.** It changes where the next units point, not what any model does. The work that can move a reply is the native read training run (the claude-393 sheet), which is the next unit.
3. **The witness pair it names as a gate is not on main yet.** It is in codex-3's PR #4015, which is open. The gate is named by its content (two histories with the same sum and different top-1) as well as by its PR, so it stands even if the file moves.

## What changed

- **`docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`:**
  - §1 now states CHARTER §3.7/D85 (one native state, the same typed operators, float as reference only) in place of "training may use float";
  - the RoPE `Position` arm is withdrawn, and transport spin is the one position mechanism;
  - the JEPA `Loss` arm is parked. Its trainer is `L_CE + …` with an f64 Adam, against §3.2 and §3.7. It may return in a CE-free, native form;
  - I1 is the one `Read` interface D86 step 6 needs, with `Q31FirstPickRead` (claude-389 to claude-393) as its reference and the Hamming heads second;
  - I2 carries the codex-3 witness pair as a required gate;
  - I3 and I5 are deferred until D86 step 6, I4 is withdrawn, and I6 is the D86 step 6 run itself;
  - a dated **Amendment 1** section lists the seven changes and their reasons, and the header names the amendment.
- **`docs/roadmap/items/end-to-end-integration.md`:** "What is missing" and the done-when follow the ADR, and the scope gains `crates/uor-r4-native-vector/src/**`, where `Q31FirstPickRead` lives. I3 and the loss arm move out of the done-when into follow-ups, so the item still closes when I6 runs.

No mechanism status, code, test, schema or GPU run changed. PR #4015 is untouched.

## Measurements

None. It is a docs unit. The evidence it rests on is cited, not re-measured:
- codex-17's served tokens (0 of 8 moved) and its mean rank (123.57 → 123.86);
- D86's review, which counts about 39k usable TRAIN windows;
- claude-387/388's first-pick and hard-copy reads.

## Checks run at the head

- `git diff --check`: clean.
- `uor-harness loop pr-check --local`: PASS (D76 advisories only: the AGENTS.md and FILEMAP budgets, which are on main).
- The docs link check: in CI (`docs archive and links`).
- A fresh-context review: CHANGES at `3114703d2` (four findings: the new `depends_on` would hold I1 behind the run it feeds; done-when 5 contradicted "closes when I6 runs"; an unrecorded "29 Sep" citation for "no RoPE"; "D86 step 6" pointed at the D86 row, which has no steps). All four, and the low ones, are fixed at the next head. The approval record is on the PR.

## Skills used

- `scope-guard`: two docs files and this unit's folder. The open PR #4015 and the charter were not touched.
- `solution-integrity`: every correction cites the rule it restores (§3.2, §3.7, D85, the board's "no RoPE"), and none adds a rule.

## Still open

1. **The native read training run** (the claude-393 sheet): its three tools first, then the arms on the c387 states on wick. That is D86 step 2's training and this ADR's I1 reference.
2. **The 15 Nov owner direction** should be recorded verbatim in PLAN.md by the lab whose chat holds it (Codex).
3. **Codex's lane objective** (codex-17) on real states instead of `synthetic_fbs_trunk`: suggested on #2028, Codex's call.
4. **The sh-spin planted-symmetry probe** against popcount and L1 controls, before the l = 2 and 3 tables.

## Plain summary

Codex's integration plan said training could use float, and it planned a RoPE position arm and a JEPA loss that uses cross-entropy. All three break rules the project already has, so the plan now drops RoPE, parks JEPA until it has a lawful form, and says training stays in the one integer state. Its first unit now builds the read interface around the native read Claude is training, so the two lines of work meet. The codebook and loss experiments wait until a model replies.
