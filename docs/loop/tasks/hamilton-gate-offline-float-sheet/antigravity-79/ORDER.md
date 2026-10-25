# ORDER: hamilton-gate-offline-float-sheet, antigravity-79

Lab: antigravity. Item: [hamilton-gate-offline-float-sheet](../../../../roadmap/items/hamilton-gate-offline-float-sheet.md). Base: `bf92c9228`.

## Goal

The item's done-when reads:

> An offline-float (D37) sheet reports a language fit on the integer DEV metrics and an export gap, and states whether the gate suppressed signal.

## Cancel probe (run first)

Inspected `docs/roadmap/items/hamilton-gate-offline-float-sheet.md`, `docs/mechanisms/hamilton-gate.md`, `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/hamilton-gate.json`, `docs/labs/r-hamilton-gate-suppression/README.md`, `docs/labs/r-secant-surrogate-descent/README.md`, `crates/uor-r4-float-train/src/geometric_stack_native.rs`, and `crates/uor-r4-integer/src/h4_classifier.rs`.

- **Cancel probe result (`instrument`):** Both prerequisites (`float-training-module` `#2823` and `r-secant-surrogate-descent` `#2886`) are `done` on `main`, and the pinned `mw-balp/train` and `ft/dev` (`14,287` eligible response targets across `400` documents) inputs exist in `caseyallard/uor-r4-store:w2-inputs/2026-10-25/`. No pre-registration sheet `docs/harness/prep/hamilton-gate-offline-float-sheet*` or runner in `crates/uor-r4-float-train` exists yet to run the D37 offline-float Hamilton gate sheet, measure its language fit on the 14,287-row integer DEV metrics and 120-icosian + integer Q12 export gap, and measure empirical gate signal suppression. Proceed to author the prep sheet and runner, obtain fresh-context FIT/D68/D67 reviews, run on a leased RTX 5090 pod, and record the results.

## Files in scope

- `crates/uor-r4-float-train/**`
- `crates/uor-r4-integer/**`
- `docs/mechanisms/hamilton-gate.md`
- `docs/harness/prep/hamilton-gate-offline-float-sheet*`
- `docs/labs/hamilton-gate-offline-float-sheet/**`
- `docs/loop/tasks/hamilton-gate-offline-float-sheet/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/hamilton-gate-offline-float-sheet.md`
- `docs/roadmap/README.md`

## Done when

1. One-page pre-registration sheet `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md` passes fresh-context `TEST FITNESS: FIT`, D68, and D67 pre-run reviews.
2. `crates/uor-r4-float-train` implements `src/hamilton_sheet.rs` and `src/bin/hamilton-gate-sheet.rs` comparing `Hamilton_Active`, `Hamilton_IcosianHard` (hard 120-icosian snap + straight-through bridge), `IdentityGate_Control` ($q=(1,0,0,0)$), and `RandomGate_Null` across seeds `1` and `2` on `mw-balp/train`, evaluated on the `14,287`-row `ft/dev` split in both continuous `f32` and D11-compatible integer export (120-icosian snap + Q12 `i32` L1 `NativeVectorWriter`) with full gate suppression telemetry (`|g|_2`, `rho(g,u)`, `1/r`, and downstream projection norm ratio).
3. Two independent `nvcc` builds execute the probe ladder and full 2-seed run on a leased `uor-pod` RTX 5090, artifacts are archived to `caseyallard/uor-r4-store:results/antigravity/hamilton-gate-offline-float-sheet-2026-10-25/`, the pod is torn down, and lab record, mechanism page, notebook row, roadmap item (`status: done`), and `REPORT.md` are committed.

## Stop and ask if

- Any instrument check (`I-Finite-CUDA`, `I-DEV-Denominator`, `I-Icosian-Closure`, `I-Q12-Int-Parity`) fails and cannot be repaired inside `crates/uor-r4-float-train`.
- Any edit outside declared scope or any `CHARTER.md` §3 / GPU cap change is required.
