# ORDER: read-binding-on-lawful-read, antigravity-70

Lab: antigravity. Item: [read-binding-on-lawful-read](../../../../roadmap/items/read-binding-on-lawful-read.md) (#2889). Base: `c091584658e7496f70ec44fc98c76f192ff96679`.

## Goal

The item's done-when reads:

> Read binding trained on a Flock or Relation read reports wrong-value errors on the same 120-item panel against the unbound read over 3 seeds.

This unit prepares `DiscreteReadBindingHingeV1` porting read-binding supervision (`docs/mechanisms/read-binding.md`, `archive/labs/read-binding-powered-2026-10-11/README.md`) to `Flock` (`MODE_FLOCK = 1`, `s[8] = 1`) and `Relation` (`MODE_RELATION = 2`, `s[8] = 2`) in `crates/uor-r4-discrete/src/discrete.cu`, authors `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` and `docs/labs/read-binding-on-lawful-read/README.md`, runs a fresh-context `TEST FITNESS: FIT` review (CHARTER §4.5) and `D68` decision review (CHARTER §5.1) to set `status: held` until `#2836` (`m1-bvg-k-stage-t-training`) and `#2844` (`m1-trunk-assembly-and-training`) complete, and logs the owner review on `#2889`.

## Cancel probe (run first)

Ran `command grep -rn "read_binding" docs/ crates/ archive/labs/read-binding-powered-2026-10-11/` and inspected `docs/mechanisms/read-binding.md`, `archive/labs/read-binding-powered-2026-10-11/README.md`, and `crates/uor-r4-discrete/src/discrete.cu` (`lines 193-326`).
- **Probe result (classified `mechanism` — configuration gap between archived softmax-mass read-binding supervision and the `D32` discrete `Flock`/`Relation` path):**
  1. In `archive/labs/read-binding-powered-2026-10-11/README.md`, read-binding supervision (`read_binding_supervision = 0.1` from step 0 across 3 seeds `20261009/10/11` vs matched $W = 0$ controls on `chat-29m-B`, `120` memory rows) raised bound mass `0.732 -> 0.896/0.899/0.905`, lowered competing mass `0.140 -> 0.090/0.086/0.081`, cut wrong-value errors `47 -> 30 / 120` (`-17`), and raised memory accuracy `57 -> 65 / 120` (`48 -> 64 / 120` at 8,000 steps). However, that formulation supervised continuous softmax probability mass $m_{\text{bound}}^{(h)} = \sum_{j \in B} a_j^{(h)}$ via $\mathcal{L}_{\text{bind}} = -\log(\max_h m_{\text{bound}}^{(h)} + 10^{-6})$ on a 29M dense matmul float stack (`crates/uor-r4-training/src/geometric_stack.rs`), banned by `CHARTER §3.2` (`D23`) and `§3.3` (`D25`/`D30`).
  2. On the active `D32` discrete path (`crates/uor-r4-discrete/src/discrete.cu`, `flock_vector` lines 193-321), `MODE_FLOCK` (`s[8] = 1`) and `MODE_RELATION` (`s[8] = 2`) select up to $K_{\text{kept}} = s[9] \in [1, 8]$ causal nodes (`top[0..found]`, lines 282-308) ordered by `(rel[node] desc, distance asc, hamming asc, last[node] desc, node asc)` with zero float, softmax, or dense matmul, and no read-binding supervision term exists in `row_loss` (`lines 364-379`).
  3. Furthermore, `read-binding-on-lawful-read` is scoped to `[docs/harness/prep/read-binding-on-lawful-read*, docs/labs/read-binding-on-lawful-read/**, docs/loop/tasks/read-binding-on-lawful-read/**]` (excluding `crates/uor-r4-discrete/**`) and is an M2/M3 memory mechanism study (`PLAN.md` lines 75-86). Completing the `DiscreteReadBindingHingeV1` specification, pre-registration sheet, `TEST FITNESS: FIT` review, and `D68` hold is the exact non-blocking step now.

## Files in scope

- `docs/harness/prep/read-binding-on-lawful-read*`
- `docs/labs/read-binding-on-lawful-read/**`
- `docs/loop/tasks/read-binding-on-lawful-read/**`

## Done when

- `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` and `docs/labs/read-binding-on-lawful-read/README.md` are committed with a passed fresh-context `TEST FITNESS: FIT` review and `D68` decision review logged on `#2889`, setting `docs/roadmap/items/read-binding-on-lawful-read.md` to `status: held` (`References #2889`).

## Stop and ask if

- Any proposed read-binding objective uses softmax, log-softmax, sigmoid, tanh, GELU, normalized exponential, dense matmul, or float arithmetic in `crates/uor-r4-discrete`.
