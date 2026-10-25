# Native Geometric Engine Re-Registration with Rank-Margin Hinge (`native-geometric-reregister-hinge`, `#2871`)

- **Date:** 2026-10-25
- **Lab:** `antigravity`
- **Roadmap item:** [`native-geometric-reregister-hinge`](../../roadmap/items/native-geometric-reregister-hinge.md) ([`#2871`](https://github.com/UOR-Foundation/uor-r4/issues/2871))
- **Pre-registration sheet:** [`docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`](../../harness/prep/native-geometric-reregister-hinge-2026-10-25.md)
- **Mechanism pages:** [`docs/mechanisms/native-geometric.md`](../../mechanisms/native-geometric.md) (`PARKED`), [`docs/mechanisms/rank-margin.md`](../../mechanisms/rank-margin.md) (`ACTIVE`)
- **Fresh held-out panel:** [`docs/harness/panels/m2-fresh256.json`](../../harness/panels/m2-fresh256.json) ([`docs/labs/m2-fresh-heldout-panel/README.md`](../m2-fresh-heldout-panel/README.md))
- **Status:** Pre-registered (`TEST FITNESS: FIT`, [`#2871` comment `6396265555`](https://github.com/UOR-Foundation/uor-r4/issues/2871#issuecomment-6396265555)) and **held** (`status: held` under D68 decision review) pending D29 kernel rebuild, shared-writer join, and M2 route preconditions (`PLAN.md` lines 75–77).

---

## 1. Codebase Audit (Cancel Probe Findings)

Before authoring the pre-registration sheet, we audited the repository state of the `native-geometric` line at base `188af113f`:

| Component | Repository Path | Current State | Blocker to Immediate GPU Execution |
|---|---|---|---|
| **September & October `native_geometric` core** | [`crates/uor-r4-core/src/native_support.rs:1-15`](../../../crates/uor-r4-core/src/native_support.rs) | Historical `native_geometric` executors are excluded from the active build via the `native_support.rs` facade. `CHARTER.md` §2.4 states that legacy crates are repaired to build only and `native-geometric` stays `PARKED`. | Excluded from active build (`cfg(any())` / facade). |
| **October native-context CUDA kernels** | [`crates/uor-r4-training/src/native_geometric_cuda_kernels.rs:23-25`](../../../crates/uor-r4-training/src/native_geometric_cuda_kernels.rs) | `HISTORICAL_KERNELS` bails at launch with `D29: ... is a historical exp/log-normalising kernel excluded from every compiled module`. | Reopen condition 3 ([`docs/mechanisms/native-geometric.md:96`](../../mechanisms/native-geometric.md)) requires rebuilding and re-qualifying the 15 quarantined kernels without hidden dense contractions (`D29`) or exponential normalizers (`D23`). |
| **Pooled-token ranking objective (`saved177 -> saved437`)** | [`archive/labs/m2-pooled-rank-2026-10-10/README.md:21-27`](../../../archive/labs/m2-pooled-rank-2026-10-10/README.md) | Used $\ell = \operatorname{softplus}(-\log(M[y]/M[r]))$, `softmax(ell)` token weights, and $\log(\frac{1}{T}\sum_t \exp(\ell_t))$ episode aggregation (`177/512 -> 437/512` exposed dev, `0/128` fresh in [`archive/labs/m2-fresh-437-2026-10-10/README.md`](../../../archive/labs/m2-fresh-437-2026-10-10/README.md)). | Banned by `CHARTER.md` §3.2 (`D23`/`D25`: no softplus, softmax, or log-sum-exp anywhere). |
| **Replacement fresh held-out panel (`m2-fresh256`)** | [`docs/harness/panels/m2-fresh256.json`](../../harness/panels/m2-fresh256.json) | Frozen (`256` rows / `128` opposite-role pairs across `16` blocks, `0` exact/word/2-gram overlap with all `74` historical literals), `TEST FITNESS: FIT`, uploaded at `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` (`fb8c5e4f`). | Ready (`NOT_RUN`); awaits a D23/D29/D37-compliant candidate. |
| **Roadmap item scope** | [`docs/roadmap/items/native-geometric-reregister-hinge.md:10`](../../roadmap/items/native-geometric-reregister-hinge.md) | `scope: [docs/harness/prep/native-geometric-reregister-hinge*, docs/labs/native-geometric-reregister-hinge/**, docs/loop/tasks/native-geometric-reregister-hinge/**]` | Docs-only scope: excludes `crates/uor-r4-core/**`, `crates/uor-r4-training/**`, and `crates/uor-r4-integer/**`. |

---

## 2. Lawful Rank-Margin Hinge Replacement (`H_hinge_q4`)

[`docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`](../../harness/prep/native-geometric-reregister-hinge-2026-10-25.md) replaces every banned operation in the October `saved177 -> saved437` recipe while keeping the exact `saved177` starting point (`dfb945d5...`, `177 / 512` complete replies) and `115,200` Q4 cross-state $H_4$ bank architecture (`8` lanes $\times 120 \times 120$):

1. **No exponential masses (`M[v]` removed)**: Candidate actions (Generate tokens and Copy aliases) are ranked directly by their exact Q24 integer score $s(v) = s_{\text{base}}(v) + \Delta s(v)$, where $\Delta s(v) = 2^{22} \sum_{l=0}^7 E_l[f_l^{-1}\pi_{v,l},\ p_l^{-1}\pi_{v,l}]$ is computed via the 120-element icosian group table (`2I`, exact $\mathbb{Z}[\varphi]$ arithmetic).
2. **Linear rank-margin hinge over top-$k$ rivals (`k = 8`, $\delta = 2^{22}$)**:
   $$\ell_{\text{hinge}}(e, t) = \frac{1}{k} \sum_{r \in \mathcal{R}_k(e, t)} \max\!\big(0,\ \delta - (s(y) - s(r))\big), \qquad J(e) = \frac{1}{T_e} \sum_{t=1}^{T_e} \ell_{\text{hinge}}(e, t).$$
   Both `softplus(-d)` and `log-mean-exp` / `softmax(ell)` episode weighting are replaced by piecewise-linear comparisons and uniform episode averaging (`0` transcendental functions).
3. **Dual-panel promotion bar (`B_native_hinge`) across two seeds (`seed = 1001, 1002`)**:
   - **Development retention (`m2-reply512`)**: `complete_replies >= 256 / 512` (`50.0%`, beating `C_parent177 = 177/512` by $> 12\times$ the cluster SE of `6.24` rows) with D37 float-to-Q4 export gap `<= 12` rows (`<= 2 * SE`).
   - **Sealed fresh transfer (`m2-fresh256`)**: `pairs_both_complete_distinct >= 52 / 128` (`40.625%`, $12.0\times$ the cluster-adjusted SE of `4.32` pairs above the `0 / 128` question-blind controls) and single-row `complete_replies >= 104 / 256`, beating `C_rival_shuf` by $\ge 9$ pairs ($2\text{SE}$).

---

## 3. Prerequisites to Unhold and Run on GPU

`native-geometric-reregister-hinge` (`#2871`) is ready to execute on a single RTX 5090 pod (`~$0.25`, `~25 minutes`) under [`docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md`](../../harness/prep/native-geometric-reregister-hinge-2026-10-25.md) once:
1. **D29 kernel rebuild & shared-writer join (`docs/mechanisms/native-geometric.md:96-97`)**: A dedicated code item (with crate scope in a new D37-scanned module) rebuilds the quarantined native-context CUDA kernels without hidden dense contractions or exponential normalizers and joins token/action emission to `native_vector_writer.rs`.
2. **M1 close (`#2029`) & M2 route open (`PLAN.md` lines 75–77)**: Per `PLAN.md` ("Work on a later milestone runs only when the step it needs is done, or when nothing on an earlier path is free"), GPU training on the M2 reply line runs after the M1 path completes.
