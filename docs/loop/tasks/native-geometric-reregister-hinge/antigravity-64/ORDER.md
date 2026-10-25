# ORDER: native-geometric-reregister-hinge, antigravity-64

Lab: antigravity. Item: [native-geometric-reregister-hinge](../../../../roadmap/items/native-geometric-reregister-hinge.md). Base: `188af113f`.

## Goal

The item's done-when reads:

> A pre-registered hinge-loss sheet passes FIT and, once the hold lifts, runs once with fresh-row results reported.

Because `native-geometric-reregister-hinge` (`#2871`) has no pre-registration sheet (`docs/harness/prep/native-geometric-reregister-hinge*`), `native-geometric` is `PARKED` (`CHARTER.md` §2.4, `docs/mechanisms/native-geometric.md:3`), its 15 October CUDA kernels are quarantined under `D29` (`crates/uor-r4-training/src/native_geometric_cuda_kernels.rs:23-25`), and the item's `scope:` excludes all Rust crates, this unit:
1. Authors and FIT-reviews `docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md` (`CHARTER.md` §4.5, `docs/mechanisms/native-geometric.md:92-99`, `docs/harness/panels/m2-fresh256.json`) replacing softplus pooled ranking with the lawful rank-margin hinge over cross-state Q4 H4 banks (`H_hinge_q4` vs `C_parent177`, `C_rival_shuf`, `C_question_blind`) across two seeds on `m2-reply512` (`>= 256/512`) and `m2-fresh256` (`pairs_both_complete_distinct >= 52/128`, `complete >= 104/256`).
2. Documents the audit, hinge-loss formulation, six controls (`I-1..I-6`), and reopen prerequisites in `docs/labs/native-geometric-reregister-hinge/README.md`.
3. Sets `native-geometric-reregister-hinge` (`#2871`) to `status: held` under a fresh-context D68 decision review until the D29 kernel rebuild, shared-writer join, and M2 route preconditions (`PLAN.md` lines 75–77) are met.

## Cancel probe (run first)

Ran the repository probe on `native-geometric-reregister-hinge` at base `188af113f`:
1. `ls docs/harness/prep/native-geometric-reregister-hinge* docs/labs/native-geometric-reregister-hinge 2>/dev/null` -> no sheet or lab directory exists (`instrument`).
2. `crates/uor-r4-core/src/native_support.rs:1-15` -> historical `native_geometric` executors are excluded via the `native_support.rs` facade (`CHARTER.md` §2.4 keeps `native-geometric` `PARKED`) (`instrument`).
3. `crates/uor-r4-training/src/native_geometric_cuda_kernels.rs:23-25` -> `HISTORICAL_KERNELS` bails under `D29`, and `docs/mechanisms/native-geometric.md:55,96` requires rebuilding the 15 quarantined kernels without hidden dense contractions before any GPU run (`instrument`).
4. `docs/roadmap/items/native-geometric-reregister-hinge.md` line 10 restricts `scope:` to docs/prep paths (`instrument`).

Classification: **`instrument`**. Authoring the FIT-reviewed sheet, protocol, and D68 `status: held` transition proceeds inside scope.

## Files in scope

- `docs/harness/prep/native-geometric-reregister-hinge*`
- `docs/labs/native-geometric-reregister-hinge/**`
- `docs/loop/tasks/native-geometric-reregister-hinge/**`
- `docs/roadmap/items/native-geometric-reregister-hinge.md` (status line, always allowed)
- `docs/roadmap/README.md` (shared registration file)

## Done when

- `docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md` is written and passes a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2871`.
- `docs/labs/native-geometric-reregister-hinge/README.md` documents the codebase audit, the lawful rank-margin hinge formulation replacing softplus pooled ranking, the six finite controls `I-1..I-6`, the `m2-reply512` + `m2-fresh256` dual-panel evaluation protocol, and the reopen prerequisites.
- `docs/roadmap/items/native-geometric-reregister-hinge.md` is set to `status: held` after a fresh-context D68 decision review posted on `#2871`, with `docs/roadmap/README.md` re-rendered and all checks green.

## Stop and ask if

- Any proposed design uses softmax, softplus, log-sum-exp, squashing, or dense matmul (`CHARTER.md` §3.1–§3.6).
- Any change outside the declared scope is needed (`stop and decide` via D68 decision review, never wait).
