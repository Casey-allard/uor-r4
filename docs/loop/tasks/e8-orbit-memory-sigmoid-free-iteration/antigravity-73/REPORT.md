# REPORT: e8-orbit-memory-sigmoid-free-iteration, antigravity-73

Lab: antigravity. Item: [e8-orbit-memory-sigmoid-free-iteration](../../../../roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md) (#2822). Order: [ORDER.md](ORDER.md).
References #2822.

## What did not work

- **Direct GPU execution of `e8-orbit-memory-sigmoid-free-iteration` (`#2822`) is blocked by item scope, missing CUDA kernels in `crates/uor-r4-float-train`, and the unbuilt D11 float-trunk exporter (`instrument` — scope & dependency barrier, not a mechanism negative):**
  1. **Docs-only scope on `#2822`:** `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md:10` declares `scope: [docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*, docs/labs/e8-orbit-memory-sigmoid-free-iteration/**, docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**]` (`0` crate paths in `crates/uor-r4-float-train/**` or `crates/uor-r4-integer/**`).
  2. **Legacy `crates/uor-r4-training/src/e8_memory.rs` has three `CHARTER.md` §3 violations and is absent from `crates/uor-r4-float-train` (`#2823`):**
     - `crates/uor-r4-training/src/e8_memory.rs:415-421` weights each slot by a banned logistic sigmoid `.neg()?.exp()?.affine(1.0, 1.0)?.recip()?` (`CHARTER.md` §3.2 / D23);
     - `crates/uor-r4-training/src/geometric_stack.rs:5995` computes queries via a banned dense matmul `self.linear(u, p.layer(layer, "e8.query.weight")?)?` (`CHARTER.md` §3.3 / D25/D30);
     - `crates/uor-r4-training/src/e8_memory.rs:339, 351, 405` copies `query` to the host (`to_vec1::<f32>()`) and runs `e8_select` on the CPU via Rayon `into_par_iter()` with `k` restricted to `{1, 2, 4, 8}` (`CHARTER.md` §3.5 / D24). Meanwhile `crates/uor-r4-float-train` (`#2823`) contains only the 24 base geometric-stack and `spiralcore_fbs` kernels (`KERNEL_ENTRY_POINTS: [&str; 24]`) without an `e8_memory` module.
  3. **`d11-exporter-float-trunk` is still `status: ready` (`docs/roadmap/items/d11-exporter-float-trunk.md`):** Both `CHARTER.md` §3.7 (D37) and `docs/mechanisms/e8-orbit-memory.md:42, 85-87` require every offline-float candidate to export to a D11 integer served form (`UORLUT01`) and report its float-to-integer export gap and integer `tokens/s`.

## What changed

- **Size (`uor-harness loop pr-check --local` / `git diff --stat origin/main...HEAD`):** 6 files changed (`tests 0 file(s) +0/-0, code 0 file(s) +0/-0, docs 5 file(s) +298/-1, generated 1 file(s) +3/-3` across `ORDER.md`, `REPORT.md`, `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md`, `docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md`, `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md`, and `docs/roadmap/README.md`).
- **Pre-registration sheet (`docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md`, `116` lines):**
  - Pre-registers the 5-arm $2 \times 2$ factorial + no-memory control GPU study (`E8h_sigfree_k8_fixed`, `E8h_sigfree_k16_fixed`, `E8h_sigfree_k8_offset`, `E8h_sigfree_k16_offset`, and `C_hamilton_trunk` per L-010) across `2` seeds (`seed = 1`, `seed = 2`, `2,000` steps) in `crates/uor-r4-float-train`;
  - Replaces the logistic sigmoid gate with the clamped-linear gate $w_{h,r} = R[h][r]\cdot\mathrm{clamp}(\beta_h s_{h,r} + \gamma_h, 0, 1)$, dense query matmul with `QuaternionWeights::SpiralcoreFbs`, and host Rayon selection with on-device CUDA $O(8)$ $E_8$ decode + warp top-$k$ selection ($k \in \{8, 16\}$) and Voronoi-bounded per-orbit residual frame offsets $\delta_{h,o,m} \in \mathbb{R}^8$ ($\|\delta_{h,o,m}\|_2 \le 0.5$);
  - Compares against the stored Rung-1 capacity-matched learned-key reference `P` (`1.4175` mean BPB, `E8h - P = +0.0231`, `pooled_sd = 0.0054` in `archive/labs/e8-orbit-memory-2026-10-11/result.json`) and the lawful unigram (`1,226 / 14,287`) and T2 successor (`6,300 / 14,287`) count tables on `ft/dev` (`14,287` rows) through the D11 integer export, leading with integer `tokens/s`;
  - Carries all 8 required `prep.rs` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS: FIT`) and passed fresh-context non-author review (`c5fc78f4-35da-4fa3-b450-d6316fc72d3c`, [`#2822#issuecomment-6398487538`](https://github.com/UOR-Foundation/uor-r4/issues/2822#issuecomment-6398487538)).
- **Companion design & CUDA specification (`docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md`, `92` lines):** Documents the cancel-probe audit, exact forward/backward equations, CUDA kernel signatures (`e8_select_fwd_k8_f32`, `e8_select_fwd_k16_f32`, `e8_memory_gather_fwd_f32`, `e8_memory_scatter_bwd_f32`), and D11 `UORLUT01` integer export layout.
- **D68 status transition (`docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md`, `docs/roadmap/README.md`):** Following fresh-context D68 decision review (`c5fc78f4-35da-4fa3-b450-d6316fc72d3c`, `DECISION REVIEW: APPROVE`, logged at [`#2822#issuecomment-6398487538`](https://github.com/UOR-Foundation/uor-r4/issues/2822#issuecomment-6398487538)), set `status: held` and re-rendered `docs/roadmap/README.md`.
- **Suborders issued:** `local/antigravity/antigravity-73/orders/1.md` (read-only fresh-context non-author `TEST FITNESS` and D68 decision review by subagent `c5fc78f4-35da-4fa3-b450-d6316fc72d3c`) and `local/antigravity/antigravity-73/orders/2.md` (fresh-context D66 exact-head review of PR `#3219`).

## Measurements

- **Cancel-probe static inspection (`d9dfbdb401f66d749afe4215afdb0a1370168852`):**
  - `crates/uor-r4-float-train/src/` entry count for `e8_memory`: `0` files (`24` base geometric-stack / `spiralcore_fbs` kernels only);
  - `crates/uor-r4-training/src/e8_memory.rs:419` banned `.exp()?` count: `1` (logistic sigmoid rank gate);
  - `crates/uor-r4-training/src/geometric_stack.rs:5995` banned `linear(u, e8.query.weight)` count: `1` (dense query matmul);
  - `crates/uor-r4-training/src/e8_memory.rs:339` host transfer `to_vec1::<f32>()` in `e8_select`: `1` (CPU Rayon fallback);
  - Stored Rung-1 reference baselines (`archive/labs/e8-orbit-memory-2026-10-11/result.json`): `D256 = 1.4078` (`1.4054`, `1.4101`), `P = 1.4175` (`1.4143`, `1.4207`), `E8h = 1.4406` (`1.4406`, `1.4406`), `E8a = 1.5237` (`1.5172`, `1.5302`), `E8h - P = +0.0231`, `E8h - D256 = +0.0329`, `pooled_sd = 0.0054`.
- **Pre-registration completeness check (`uor-harness check`):** `0` errors on `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md` (`8 / 8` required blocks present, `TEST FITNESS: FIT`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap render`: `PASS` (`92` items rendered)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: `PASS`
- `uor-harness loop report --check --order docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/antigravity-73/ORDER.md`: `PASS`
- `uor-harness loop pr-check --local`: `PASS` (`0` errors, `0` warnings)
- `uor-harness loop gates --full`: `PASS` (`11 / 11` gates `GREEN`)

## Skills used

- `scope-guard`: Kept all edits strictly inside `#2822`'s declared `scope:` globs plus the item's `status:` line and the shared `docs/roadmap/README.md` registration file, refusing to touch `crates/uor-r4-float-train/**` or `crates/uor-r4-integer/**` outside scope.
- `solution-integrity`: Classified the missing `crates/uor-r4-float-train` E8-memory CUDA kernels and missing `d11-exporter-float-trunk` as `instrument` / prerequisite barriers (`CHARTER.md` §4.2), designed a full $2 \times 2$ factorial (`k8_fixed`, `k16_fixed`, `k8_offset`, `k16_offset`) plus no-memory control (`C_hamilton_trunk`, L-010) to separate slot bandwidth from per-orbit frame offsets, and completed both the non-author `TEST FITNESS: FIT` review and the D68 decision review before setting `status: held`.
- `gates`: Verified `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` (`11 / 11` `GREEN`).

## Still open

- **Landing the compliant `e8_memory` module and CUDA kernels in `crates/uor-r4-float-train`:** Needs a code-scoped unit on `crates/uor-r4-float-train` implementing `e8_select_fwd_k8_f32`, `e8_select_fwd_k16_f32`, `e8_memory_gather_fwd_f32`, `e8_memory_scatter_bwd_f32`, `SpiralcoreFbs` query projection, and Voronoi-bounded per-orbit residual offsets $\delta_{h,o,m} \in \mathbb{R}^8$.
- **Landing `d11-exporter-float-trunk` (`docs/roadmap/items/d11-exporter-float-trunk.md`):** Required by `CHARTER.md` §3.7 (D37) and `docs/mechanisms/e8-orbit-memory.md:42, 85-87` so the float-trained checkpoints export to `UORLUT01` and report integer `tokens/s` and export gap on the 14,287-row DEV panel.
- **Executing the 5-arm two-seed GPU run (`#2822`):** Once those two code prerequisites land, `#2822` reopens from `status: held` and runs under `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md` (`TEST FITNESS: FIT`) after a D67 pre-run review.

## Plain summary

E8 orbit memory (`e8-orbit-memory`) is a way to replace the expensive feed-forward layers in a language model with a fast table lookup indexed by the 240 special points (roots) of the 8-dimensional E8 lattice, grouped into 15 orbits of 16 points under SpiralCore's 64 signed-axis symmetries (`Cl(0,6)`). In an earlier CPU test (Rung 1 on 11 October 2026), replacing half the feed-forward layers (`E8h`) used about 10 times fewer arithmetic operations and stayed within the allowed quality margin (`+0.033` bits per byte, or BPB, a measure of prediction error where lower is better), but a control table with learned addresses (`P`) beat the fixed E8 addresses by `0.0231` BPB. Moreover, the old code in `crates/uor-r4-training` used three things the project's charter (`CHARTER.md` §3) forbids: a logistic sigmoid squashing function to weight the selected table rows (violating D23), a dense matrix multiplication to form the query (violating D25/D30), and a host CPU loop (`Rayon`) to pick the top rows (violating D24, GPU-only research compute). Because roadmap item `#2822` (`e8-orbit-memory-sigmoid-free-iteration`) is scoped only to documentation and pre-registration files, and because both the new `e8_memory` CUDA kernels in `crates/uor-r4-float-train` (`#2823`) and the integer exporter for float trunks (`d11-exporter-float-trunk`, required by D37 so every float-trained model is exported and tested as an integer artifact) are not yet built, this unit wrote and adversarially reviewed (`TEST FITNESS: FIT`) the one-page pre-registration sheet `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md` and the companion technical specification `docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md`. The design replaces the sigmoid with a clamped-linear gate (`clamp(beta * score + gamma, 0, 1)`), replaces the query matrix multiplication with `SpiralcoreFbs` geometric routing, moves top-$k$ slot selection onto CUDA, and sets up a 5-arm, 2-seed factorial experiment testing both of the Rung-1 record's named improvements — doubling kept rows per head from $k = 8$ to $k = 16$ and adding a learned 8-dimensional residual offset per orbit — alongside a no-memory geometric control (`C_hamilton_trunk`) and the lawful count-table baselines. After passing a fresh-context whole-project decision review (D68), `#2822` was set to `status: held` until the `crates/uor-r4-float-train` CUDA kernels and `d11-exporter-float-trunk` land.
