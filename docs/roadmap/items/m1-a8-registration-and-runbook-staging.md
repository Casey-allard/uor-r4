---
id: m1-a8-registration-and-runbook-staging
title: Register the finite qualification and stage the admission-preserving A8 runbook
milestone: M1
area: tooling
status: done
open_to: anyone
needs_gpu: false
depends_on: []
scope: [docs/harness/prep/m1-a8-registration-and-runbook-staging*, docs/labs/m1-a8-registration-and-runbook-staging/**]
---

## Why it matters

The next A8 dose, the admission-preserving rerun, cannot run until the finite qualification is registered and the owner has a staged release command to type. This is host and paperwork work, with no GPU. It sits ahead of the A8 rerun in the discrete-attention line; see [where we are](../../start/where-we-are.md).

## What is missing

[`NOW.md`](../../integration/NOW.md) names two items as current work: staging the audited-dose runbook and executor, and a separate prospective finite-qualification registration. Registration must keep the immutable scientific FIT sheet identity. That page is stale on #2785: the PR merged on 20 October 2026 (`c6eb05265`). It restored the admission-preserving sheet `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` byte-exact (TEST FITNESS: FIT, SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`) and moved the qualification binding to its own record, [`docs/labs/admission-preserving-qualification-2026-10-19/README.md`](../../labs/admission-preserving-qualification-2026-10-19/README.md) (protocol `train61-synthetic1-interval64-two-arm-throughput-v1`, 384 attempts per real arm). That record is a binding, not a reviewed sheet, and `NOW.md` says the finite stage needs its own FIT. The admission controller and finite qualification producer (#2782) and the host consumer verifier (#2783) are already on main. Real CUDA qualification, restoration and timing remain NOT_RUN. The A8 qualification and dose keep the sheet-bound source `fd409fcdf`, not later main. The first A8 dose already ran and stopped at its registered Q1 diagnostic stop (Relation arm 551 of 3,847 admitted against 1,351 at init; see [`docs/labs/a8-dose-2026-10-19/README.md`](../../labs/a8-dose-2026-10-19/README.md)), using the runbook [`docs/labs/a8-dose-runbook-2026-10-19/README.md`](../../labs/a8-dose-runbook-2026-10-19/README.md). What is still open is the finite-stage sheet with its FIT, and the runbook and executor for the admission-preserving dose.

## Done when

Finite-qualification sheet registered with its own binding and its own FIT, with the admission-preserving sheet identity (`fcb770e6…`) unchanged (#2785 is already merged); runbook and executor staged; the exact owner-release command staged for Casey to type.

## How to start

1. Read `docs/integration/NOW.md` lines 18-34 (stale on #2785, see above), then the lab pages: `docs/labs/admission-anchor-controller-2026-10-19/README.md`, `docs/labs/address-cells-consumer-verifier-2026-10-19/README.md` and `docs/labs/a8-training-configs-2026-10-19/README.md`. Also read `docs/labs/admission-preserving-qualification-2026-10-19/README.md`. Confirm #2785 is merged with `gh pr view 2785 --json state,mergedAt`.
2. Post a one-line intent note on the issue. First deliverable: a short plan that lists the sheet identity (file and hash) that must not change (the admission-preserving sheet above, `fcb770e6…`), and the new binding the finite-qualification sheet needs.
3. Draft the finite-qualification registration as a one-page sheet with its own binding. It must pin source `fd409fcdf` and state the bounded-update and restoration controls. The bar is not yet registered here; take it from the FIT sheet and the binding record (for example the readiness formula `R = added_gate_cost / ungated_two_arm_dose_cost ≤ 15%`) and say where it came from.
4. Check that the original FIT sheet identity is unchanged: run `~/.cargo/bin/cargo run -q -p uor-r4-harness --bin uor-harness -- check` (or `uor-harness check`) from the repository root, and compare `shasum -a 256 docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` before and after. It must print `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`.
5. Stage the audited-dose runbook and executor as files in a new lab folder, following `docs/labs/a8-dose-runbook-2026-10-19/`. The finite stage needs its own FIT, costed owner approval and PM dispatch; full dose is a separate release.
6. Stage the exact owner-release command in the runbook for Casey to type. Do not type it. The owner types one release per pre-registered sheet (CHARTER section 5.1).
7. Open a PR that says `Closes #<this issue>` (or `References` if partial)
, and add one row to `docs/research/notebook/m1-discrete-attention.md`.


## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:18-34`](../../../docs/integration/NOW.md)

