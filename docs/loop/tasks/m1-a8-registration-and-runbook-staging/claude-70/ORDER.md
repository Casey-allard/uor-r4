# ORDER: m1-a8-registration-and-runbook-staging, claude-70 (Stage 1 erratum, records and plan template)

Lab: claude. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md). Base: `origin/main`. Casey in Claude's chat, 2026-10-24: "build the 16 InheritedObservation records and the QualificationPlan from the pod-1 outputs, and write the second step-1.1 erratum (claude-70) for a targeted FIT re-confirmation". His ruling "Template approval (Recommended)" is recorded at #2842 6379923206.

## Goal

A8 pod 1 (#2842 6379753507) ran Steps 1.1–1.2 at `fd409fcdf`. It produced 97 PASS and 0 FAIL, and saved the whole root with its binary to the HF store at `results/claude/a8-stage1-pod1-20261024-r2/`. It found two things the runbook must absorb:

1. **The step-1.1 source pin cannot be reproduced.** `compiled_identity.source_sha256` hashes ptxas reports that contain `Compile time = N ms`, so it changes on every build (6379702498). The second erratum pins the deterministic digests instead (PTX, cubins, kernel, decomposition kernel, geometry, driver source), plus the saved binary's own `executable_sha256` and `source_sha256`. Pods 1 and 2 share that binary.
2. **Sign-offs within 300 s are impractical.** `validate_plan` refuses a resource observation older than 300 s. The runbook gains Casey's template-approval procedure: signers approve a plan template hash with the live resource fields removed, plus bounds, and Claude binds a fresh observation on pod 2.

This unit also commits the reviewable evidence the signers need:
- the 16 `uor-r4.attention-audit-inherited-command/1` records, whose paths are the pod paths under `/workspace/a8s1` and whose hashes come from the saved tar;
- the plan template, with its hash.

## Cancel probe (run first)

At the base, does Step 1.1 still pin `compiled_identity.source_sha256 == 177f1b3d…`, and does no committed plan template exist? If either has changed, the task is void. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-registration-and-runbook-staging/**`
- `docs/harness/prep/m1-a8-registration-and-runbook-staging*`
- This task folder

## Done when

1. **Erratum.** Step 1.1 and the sheet pin the deterministic digests and the saved binary, with the reason and the evidence. The sheet's FIT block is untouched.
2. **Procedure.** The runbook's Steps 1.3–1.4 describe the template-approval procedure, citing 6379923206.
3. **Evidence.** The 16 records are under the lab folder's `evidence/inherited/`, and every file pin resolves to a file and hash in the HF tar. *Amended before review:* the plan template moves to the next unit. Four plan inputs (`anchor_manifest`, `accepted_initlock_disposition`, `residual_fit`, `g_init_comparison_or_waiver`) exist nowhere, so the stop rule below applies. They are named on #2842 (6379986044).
4. **Checks pass:** `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Stop and ask if

- A plan field needs an artifact that does not exist on main or in the HF store. Name it rather than invent it.
- A file outside scope must change.
