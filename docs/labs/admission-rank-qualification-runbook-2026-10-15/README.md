# D-R Qualification Runbook — Step-0 CUDA Build/Clippy Gate, `C-R1..C-R6`, `G-Pre` (`K=16` Checkpoint-0 Ceiling), and 100-Batch `K=16` Synchronized Timing (`NOT_TRAINING`, 15 October 2026)

**Current execution dependency — 16 October:**
The [owner instrument+optimize-first decision](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6210107580)
defers 100-batch qualification timing to the optimized stack. The
[pending proposal diagnostic](../propose-subphase-diagnostic-2026-10-16/README.md)
does not discharge any qualification or training gate; its checkpoint-0
identity bridge remains a reviewed prerequisite. Historical execution steps
below do not authorize a restart. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE.

**Latest blocker:** [restart 5 at `812459c7f`](../k16-qualification-restart5-2026-10-16/README.md)
passes all six step-0 commands, G-Pre and C-R1..C-R6. The timing binary exits 0,
but emits only 99 timed batches per checkpoint-0 cell after skipping its warmup;
the declared 100-batch receipt check fails and execution stops. Preserve the
99-batch observations as diagnostics, not qualified throughput. No patch/retry,
training or corpus apply; Q4 remains NOT_EXECUTABLE. Repair/review/merge and
PM disposition precede any further execution.

**Earlier blocker, repaired by #2537:** [restart 4 at `5fc68a9a6`](../k16-qualification-restart4-2026-10-16/README.md)
passed all CUDA builds but halted at fixture clippy: `Parameter::VectorBlock(3 * 4 + 0)`
triggers `clippy::identity_op` under the declared `-D warnings` gate. No model
commands ran. Do not waive, patch or retry silently; repair/review/merge precede
full step-0 re-entry. G-Pre and all runtime gates remain unmet.

**Earlier blocker, repaired by #2536:** the [restart at merged `bb536cf68`](../k16-qualification-restart-2026-10-16/README.md)
stopped before execution: C-R6 expects an `evaluation` key absent from the
driver's `aggregate`-shaped G-Pre receipts. Do not rename sealed receipt fields
to bypass this defect. Schema repair/review/merge precede another full step0
restart; no qualification gate is claimed passed by this source finding.

**Execution update, 16 October:** [PM order 6205684341](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205684341)
sets step 0 → **G-Pre → C-R1..C-R6** → checkpoint-0-only timing (C-R6 consumes
G-Pre receipts), superseding the earlier numbered ordering below. The
[first attempt](../k16-qualification-2026-10-16/README.md) at `3c651df97` halted
at the fixture CUDA build, before clippy or model execution; all later gates
remain NOT_RUN. Q4 remains NOT_EXECUTABLE. Repair/review/merge precede a new
full step-0 attempt; no failed gate may be skipped.

- **Lab:** `antigravity`
- **Date:** `2026-10-15`
- **Line:** `starling read / exact addressed memory` — `count 1/3` (unchanged; runbook specification only)
- **M1 headline:** `reply acceptable 37/232 -> target 116/232` (unchanged; suspended under D28/R14)
- **Execution scope:** **`NOT_TRAINING` — source/documentation runbook only (`0` GPU hours, `0` pods, `0` model executions in this PR)**. When the owner releases a bounded qualification lease, this runbook governs **(0) the mandatory pod-side CUDA build and clippy step-0 gate, (1) finite GPU fixture qualification (`C-R1..C-R6`), (2) zero-update `K=16` checkpoint-0 ceiling evaluation (`G-Pre`), and (3) zero-update 100-batch `K=16` synchronized counterfactual timing** with **zero `.apply` updates on any corpus row**.
- **Governing authorities:**
  - Pre-registration sheet (`TEST FITNESS: FIT (design)` at `8a4388424`, merged in [#2515](https://github.com/UOR-Foundation/uor-r4/pull/2515) as `5188f2e38`): [`docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md)
  - `D-R` source slice (merged in [#2518](https://github.com/UOR-Foundation/uor-r4/pull/2518) as `2fa663b54`): [`docs/labs/admission-rank-hinge-2026-10-15/README.md`](../admission-rank-hinge-2026-10-15/README.md), [`crates/uor-r4-discrete/src/objective.rs`](../../../crates/uor-r4-discrete/src/objective.rs), [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu), [`crates/uor-r4-discrete/src/device.rs`](../../../crates/uor-r4-discrete/src/device.rs), [`crates/uor-r4-discrete/src/training.rs`](../../../crates/uor-r4-discrete/src/training.rs), [`crates/uor-r4-discrete/src/main.rs`](../../../crates/uor-r4-discrete/src/main.rs)
  - `D-Packet` data/config pins (merged in [#2523](https://github.com/UOR-Foundation/uor-r4/pull/2523) as `27c82c0c8`): [`docs/labs/admission-redirect-packet-2026-10-15/README.md`](../admission-redirect-packet-2026-10-15/README.md) and [`docs/labs/admission-redirect-packet-2026-10-15/configs/`](../admission-redirect-packet-2026-10-15/configs/)
  - `K=16` synchronized 100-batch timing instrument & step-0 gate (merged in [#2524](https://github.com/UOR-Foundation/uor-r4/pull/2524) as `f8af41057`; step-0 ruling in [#2522 comment 6202519677](https://github.com/UOR-Foundation/uor-r4/pull/2522#issuecomment-6202519677)): [`docs/labs/k16-timing-2026-10-15/README.md`](../k16-timing-2026-10-15/README.md) and [`docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`](../k16-timing-2026-10-15/instrument/Cargo.toml)
  - `K=16` redirect admission validators (merged in [#2525](https://github.com/UOR-Foundation/uor-r4/pull/2525) as `7b26da89d`): [`docs/labs/k16-admission-validators-2026-10-15/README.md`](../k16-admission-validators-2026-10-15/README.md) and [`crates/uor-r4-discrete/src/redirect_admission.rs`](../../../crates/uor-r4-discrete/src/redirect_admission.rs)
  - Owner `G-Pre` confirmation: [#2515 comment 6198442894](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198442894)
  - Compute & storage protocol: [`docs/labs/compute.md`](../compute.md), [`scripts/pod/uor-pod`](../../../scripts/pod/uor-pod), `~/.local/share/uor-r4/bin/cloud-store`
- **Hold status:** Global training hold, M2 training/rental hold, and Rung-0 Stage B remain **HELD**. Completing this qualification runbook on a released qualification lease satisfies the step-0 CUDA build/clippy gate, `G-Code` (`C-R1..C-R6`), `G-Pre`, and the `D-Packet` measured-throughput slot, after which training still requires completed-packet PM acceptance and the owner's separate personal hold release (`R8` / `R15`).

---

## 0. Pinned Source, Corpus, and Checkpoint Identities

Before running any GPU command on the qualification pod, the executor must verify the exact source, geometry, corpus, and checkpoint SHA-256 digests below:

| Artifact / Pin | Path / Function | Expected Digest (`SHA-256` / `BLAKE3`) & Contract |
|---|---|---|
| `training::driver_source_sha256()` | `crates/uor-r4-discrete/src/training.rs:19-29` (7 driver/controller files: `corpus.rs`, `training.rs`, `controller.rs`, `completion.rs`, `completion_baseline.rs`, `redirect_admission.rs`, `main.rs`) | `ed4655a22dbec24316cfe5df16a43407860651ce6aa7a690ee8fcd5bedaf05d8` (post-`#2525`; at `#2518` base source `2fa663b54` before `redirect_admission.rs`, `3487506d0d6b9197e8b99963dfe92e91b5c3e33e0801d9116bf525cdc9b17fb7`) |
| `discrete.cu` (raw file bytes) | `crates/uor-r4-discrete/src/discrete.cu` | `f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008` |
| `objective.rs` (raw file bytes) | `crates/uor-r4-discrete/src/objective.rs` | `b813e855be6c4ea460c8711493527d52f1caf1a02bb09b046145f88c7f4ee4d0` |
| `device::compiled_identity()` | `crates/uor-r4-discrete/src/device.rs:391-423` (emitted on pod by `discrete-stack completion-build-identity [CONFIG]`, `src/main.rs:53-65`, with `cuda_context_created: false`) | Emits `source_sha256` (`device.rs:398-409`, composite SHA-256 over `SCHEMA`, `device.rs`, `discrete.cu`, `metadata.rs`, `lib.rs`, `objective.rs`, `../build.rs`, and compiled `PTX`), `kernel_sha256` (`device.rs:391-396`, SHA-256 of compiled `discrete.ptx`), and `geometry_sha256` (`46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`) |
| `OPERATOR_KAPPA` | `crates/uor-r4-discrete/src/lib.rs:20` | `blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b` |
| `COMPOSITION_KAPPA` | `crates/uor-r4-discrete/src/lib.rs:22` | `blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0` |
| `tokenizer` (`FilePin`) | `local/driver/data/tokenizer.json` (`configs/k16-{A,R}-seed{1,2}.json`, `completion.rs:321`) | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` (`bos = {"token": "<|bos|>", "id": 0}`) |
| `train.tokens` (`FilePin`) | `local/driver/data/mw-balp/train/tokens.u16` (`completion.rs:288-291`) | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (`expected_payload_tokens = 7,512,797`, `expected_eligible_targets = 2,425,925`) |
| `train.mask` (`FilePin`) | `local/driver/data/mw-balp/train/response_mask.u8` (`completion.rs:293-295`) | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
| `dev.tokens` (`FilePin`) | `local/driver/data/ft/dev/tokens.u16` (`completion.rs:306-309`) | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`expected_payload_tokens = 42,856`, `expected_eligible_targets = 14,287`) |
| `dev.mask` (`FilePin`) | `local/driver/data/ft/dev/response_mask.u8` (`completion.rs:311-313`) | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| `k16-A-seed1.json` | `docs/labs/admission-redirect-packet-2026-10-15/configs/k16-A-seed1.json` | Raw file SHA-256: `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` (typed `serde_json::to_vec(&DriverConfig)` `config_sha256`: `58b99b595aa6110c8250e8183b3875e93d19d887a941d295c20937dea73b370d`; always use the emitted `completion-build-identity` receipt) |
| `k16-A-seed2.json` | `docs/labs/admission-redirect-packet-2026-10-15/configs/k16-A-seed2.json` | Raw file SHA-256: `bed46548bdb15e883386044442999584696e6cd9acddaf8c2cc5a8be6b578ee4` (typed `serde_json::to_vec(&DriverConfig)` `config_sha256`: `bf9791597a16e72de012d5367c7f0d77446450466ea2bd5865c694883846e2ba`; always use the emitted `completion-build-identity` receipt) |
| `k16-R-seed1.json` | `docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed1.json` | Raw file SHA-256: `25ef452f78cb612bdedd8135461836ec459d1f7c899e2e5940e830c3dffc61b7` (typed `serde_json::to_vec(&DriverConfig)` `config_sha256`: `4257bfc44b7e6b3f84f112f99a06a0eb83d618fe3fe0246d4b76782b9b395e3a`; always use the emitted `completion-build-identity` receipt) |
| `k16-R-seed2.json` | `docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed2.json` | Raw file SHA-256: `fe9f172b428ee131fac4a3865402f55ef80c3749ef32f1d98a830718ad361156` (typed `serde_json::to_vec(&DriverConfig)` `config_sha256`: `0726f80e4a94ccce3226e02d5a9cecfdd7ce72bdaf8be2ea241f26bfd4616714`; always use the emitted `completion-build-identity` receipt) |
| `checkpoint-16404.json` (Q4 occupancy wrapper) | `icloud:UOR-R4/results/codex/20261015T060354Z-rung0-stage-a-pilot-seed1.tar` (`CompletionCheckpoint` wrapper, `completion.rs:364`) | Wrapper file SHA-256: `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` (`.checkpoint.model_sha256 = 29ba5517a6b1b6d3c7ce54d593221560b446c5048a0b0db882ad53b5ec75bbbf`, historical pilot `.checkpoint.model.source_sha256 = e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a`, `.checkpoint.model.kernel_sha256 = 931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b`, `189` occupied cells) |

---

## 1. `C-R1..C-R6` Fixture Execution on GPU (Analytic/Stored Expected Values & Pass/Fail Criteria, Zero CPU Model Reference)

### 1.1 Policy, Kernel Invariants, and Host-Scaffold vs GPU-Fixture Separation (D24 / D32)

1. **Zero CPU model reference:** No host function may compute token distance, address distance, cell rank, candidate loss, or row aggregation as a runtime fallback or reference oracle. Every `C-R1..C-R6` fixture executes through the 11 admitted CUDA `__global__` kernels in [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`discrete_init`, `discrete_encode`, `discrete_forward`, `discrete_counterfactual`, `discrete_choose`, `discrete_apply`, `discrete_validate`, `discrete_evaluate`, `discrete_evaluation_aggregate`, `discrete_count`, `discrete_count_score`) and checks the GPU output against **hand-derived analytic integer identities or sealed stored values**.
2. **Host contract scaffolds (`objective::tests`) are NOT GPU qualification receipts:**
   - [`crates/uor-r4-discrete/src/objective.rs:429-550`](../../../crates/uor-r4-discrete/src/objective.rs) (`objective::tests`) verifies static `C-R1..C-R6` fixture metadata, status constants, and `ObjectiveEvaluationSummary` layout on the host without creating a CUDA context.
   - Per [#2029 comment 6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519), [`docs/labs/admission-redirect-packet-2026-10-15/README.md:89-93`](../admission-redirect-packet-2026-10-15/README.md), and [`crates/uor-r4-discrete/src/redirect_admission.rs:18, 357-367`](../../../crates/uor-r4-discrete/src/redirect_admission.rs), `uor-r4.redirect-qualification/1` requires six ordered GPU control receipts `C-R1..C-R6` (`status == "PASS"`, each with a non-empty digest-verified `FilePin`). Host `cargo test` output must **never** be sealed as `C-R1..C-R6` GPU receipts; if the dedicated GPU `C-R1..C-R6` fixture runner binary/subcommand has not yet landed and passed exact-head review on `main`, pod qualification is blocked on landing that GPU runner first.
3. **PTX admission gate (`admit_ptx`, `crates/uor-r4-discrete/build.rs:25-83`):** At build time under `--features cuda` and `UOR_NVCC_ARCH=compute_120` (`nvcc V12.8+`), `admit_ptx` verifies that `discrete.ptx` defines all 11 kernels and contains zero prohibited tokens (`".f16"`, `".f32"`, `".f64"`, `".bf16"`, `".tf32"`, `"cvt.f"`, `"cvt.rn.f"`, `"cvt.rz.f"`, `"cvt.rm.f"`, `"cvt.rp.f"`, `"ex2."`, `"lg2."`, `"rcp."`, `"sqrt."`, `"rsqrt."`, `"sin."`, `"cos."`, `"tanh"`, `"mma."`, `"wgmma"`).
4. **Zero regression on existing finite GPU suites (`crates/uor-r4-discrete/src/main.rs:9-28, 66-69`):** Each existing `qualify*` subcommand takes a `NEW_REPORT_DIR` positional argument (`args.len() == 2`) and atomically creates `NEW_REPORT_DIR/report.json`:
   - `discrete-stack qualify "${OUT_DIR}/qualify"` (`11/11` `C1..C11` controls, `0` corpus `.apply` calls),
   - `discrete-stack qualify-flock "${OUT_DIR}/qualify-flock"` (`6/6` `F1..F6` controls),
   - `discrete-stack qualify-relation "${OUT_DIR}/qualify-relation"` (`8/8` `C1..C8` / `10` checks),
   - `discrete-stack qualify-driver "${OUT_DIR}/qualify-driver"` (`8/8` `C-D1..C-D8` controls, capped at the `11` disposable synthetic fixture `.apply` calls enforced by `crates/uor-r4-discrete/src/lib.rs:233-267`).

### 1.2 Analytic E8 Orbit Root Geometry Used by All `C-R1..C-R5` Fixtures

In `geometry::ROOTS`, `geometry::BRANCH`, and `geometry::BFS` (`GEOMETRY_SHA256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`, `crates/uor-r4-discrete/build.rs:216-325`):

- `root 0 = [ 2,  2,  0,  0,  0,  0,  0,  0]` (anchor root; `BRANCH[0..3] = [0, 14, 3]`)
- `root 1 = [-2, -2,  0,  0,  0,  0,  0,  0]` (antipode of `root 0`; `BFS[0*16 + 1] = 16`)
- `root 2 = [-2,  2,  0,  0,  0,  0,  0,  0]` (`BFS[0*16 + 2] = 7`)
- `root 3 = [ 0,  0, -2,  0, -2,  0,  0,  0]` (`BFS[0*16 + 3] = 42`; `root 3..14` all have `0` in coordinates `0, 1` and two `±2` entries in coordinates `2..7`)
- `root 15 = [ 2, -2,  0,  0,  0,  0,  0,  0]` (antipode of `root 2`; `BFS[0*16 + 15] = 34`)

In `crates/uor-r4-discrete/src/discrete.cu`:
- `root_cost(x, g, root, &distance, &hamming)` (`discrete.cu:37-40`): unscaled L1 coordinate displacement $\text{distance} = \sum_{d=0}^{7} |x_d - g_{\text{roots}}[\text{root}, d]| \in \{0, 4, 8, 12\}$ on orbit roots.
  - Relative to `root 0`: `root_cost(root 0, 0) = 0`; `root_cost(root 0, 2) = 4`; `root_cost(root 0, 15) = 4`; `root_cost(root 0, r) = 8` for $r \in \{1, 3..14\}$.
  - Relative to `root 2`: `root_cost(root 2, 2) = 0`; `root_cost(root 2, 0) = 4`; `root_cost(root 2, 1) = 4`; `root_cost(root 2, r) = 8` for $r \in \{3..15\}$ (`13` roots at single-block distance `8`).
  - Relative to `root 3`: `root_cost(root 3, 3) = 0`; `root_cost(root 3, r) = 8` for $r \in \{0, 1, 2, 15\}$.
- `token_cell(table, g, s, token, &first, &last)` (`discrete.cu:286-290`) = $a \cdot 16 + b \in [0, 255]$, where $a = \text{root\_id}(\text{block } 0)$ and $b = \text{root\_id}(\text{block } s[1]-1)$.
- `address_distance(q, g, s, cell)` (`discrete.cu:274-276`) = $\text{root\_cost}(q_{\text{first}}, \lfloor \text{cell}/16 \rfloor) + \text{root\_cost}(q_{\text{last}}, \text{cell} \bmod 16) \in [0, 24]$.
- `cell_precedes(d, c, other_d, other_c)` (`discrete.cu:301-303`) = `(d < other_d) || (d == other_d && c < other_c)`.
- `target_cell_rank` (`discrete.cu:329-334`) = $1 + |\{c \in 0..255 : \text{occupied}[c] \neq 0 \land \text{cell\_precedes}(d(c), c, \text{ad}, \text{target\_cell})\}| \in [1, 256]$.
- `cell_rank_hinge` (`discrete.cu:340`) = $R = \max(0, \text{target\_cell\_rank} - K) \in [0, 255]$ (and $\in [0, 240]$ at $K = 16$).

### 1.3 Fixture-by-Fixture GPU Specifications, Expected Values, and Pass/Fail Criteria

Every fixture below satisfies `Spec::validate` (`crates/uor-r4-discrete/src/lib.rs:47-58`: `vocabulary: 2..=8192`, `blocks: 2..=16`, `context: 1..=243`, `kept_cells: 1..=16`, and `kept_cells <= vocabulary`).

| Control | Governing Sheet & Source | GPU Fixture Construction (Executed via `Device` / `discrete.cu`, Zero CPU Model Reference) | Analytic / Stored Expected Values | Pass / Fail Criteria (`TEST FITNESS: FIT`) |
|---|---|---|---|---|
| **`C-R1`** `rank_hinge_exact_admission_equivalence` | Sheet line 90; `objective.rs:85-91`; `discrete.cu:291-345, 449-504` | **Fixture 1A (5-cell distance + tie sweep across $K = 1..=16$):** `Spec { vocabulary: 16, blocks: 4, context: 3, kept_cells: K }` for $K \in 1..=16$ (`kept_cells <= vocabulary = 16`, `ReadSpec::Route`, `BinderPlan::new(3)` has `stages = 2, nodes = 4, frontier = [3]`). Context window: 3 tokens `[token 0, token 0, token 0]` (`Identity { token: 0, occurrence: 1..=3, version: 1 }`), `target_token = 2`.<br>• **Variant 1A-i (mixed distances `[(0, 0), (4, 2), (4, 15), (4, 32), (8, 255)]`):** `routes = [0; 8]` (`IDENTITY = 0` at `stage ∈ {0, 1}` across `block ∈ 0..4`), so `route_vector` (`discrete.cu:41-76`) selects `port = 0` (`BRANCH[0] = 0, total = 0`) and outputs $q = [\text{root } 0, \text{root } 0, \text{root } 0, \text{root } 0]$. Complete 16-token table (`occupied_cells == 5` on `{0, 2, 15, 32, 255}`): `token 0 = [r0, r0, r0, r0]` (`cell 0`, $d = 0 + 0 = 0$); `token 1 = [r0, r0, r0, r2]` (`cell 2`, $d = 0 + 4 = 4$); `token 2` (target) `= [r0, r0, r0, r15]` (`target_cell = 15`, $\text{ad} = 0 + 4 = 4$); `token 3 = [r2, r0, r0, r0]` (`cell 32`, $d = 4 + 0 = 4$); spare `tokens 4..16` (`12` tokens) all placed at `[r15, r0, r0, r15]` (`cell 255`, $d = 4 + 4 = 8$, sharing `cell 255` so no 6th cell is occupied).<br>• **Variant 1A-ii (5-way tie `[(16, 0), (16, 1), (16, 15), (16, 16), (16, 255)]` from `objective.rs:87`):** `routes = [0, 42, 0, 42, 0, 42, 0, 42]` (`stage 0 = 0 (IDENTITY)`, `stage 1 = 42 (BFS[0*16 + 3])` across `block ∈ 0..4`). On `[token 0, token 0, token 0]` (`table[0] = [r0, r0, r0, r0]`), `stage 0` selects `node 3` (`root = 0, total = 0`) and `stage 1` maps `query = r0` via frame `42` to `r3`, matches `port = 2` (`BRANCH[0*3 + 2] = 3`), composes `total = BFS[3] = 42`, and applies `42` to `carrier = table[0] = r0`, outputting $q = [\text{root } 3, \text{root } 3, \text{root } 3, \text{root } 3]$ without any token occupying `cell 51`. Complete 16-token table (`occupied_cells == 5` on `{0, 1, 15, 16, 255}`): `token 0 = [r0, r0, r0, r0]` (`cell 0`, $d = 8 + 8 = 16$); `token 1 = [r0, r0, r0, r1]` (`cell 1`, $d = 8 + 8 = 16$); `token 2` (target) `= [r0, r0, r0, r15]` (`target_cell = 15`, $\text{ad} = 8 + 8 = 16$); `token 3 = [r1, r0, r0, r0]` (`cell 16`, $d = 8 + 8 = 16$); spare `tokens 4..16` (`12` tokens) all placed at `[r15, r0, r0, r15]` (`cell 255`, $d = 8 + 8 = 16$).<br>• **Fixture 1B (256-cell full-occupancy maximum-rank boundary at $K = 16$):** `Spec { vocabulary: 256, blocks: 4, context: 1, kept_cells: 16 }`, `routes = [0; 4]`, token $c \in 0..256$ placed in cell $c = a \cdot 16 + b$ (`block 0 = root a`, `blocks 1..2 = root 2`, `block 3 = root b`), 1-token window `[token 34]` (`34 = 2 * 16 + 2`, so $q = [\text{root } 2, \text{root } 2, \text{root } 2, \text{root } 2]$), target token `255` (`target_cell = 255`, $\text{ad} = 8 + 8 = 16$). | • **Fixtures 1A-i & 1A-ii ($K = 1..=16$):**<br>  - `occupied_cells == 5` (both 1A-i and 1A-ii)<br>  - `target_cell == 15`<br>  - `target_cell_rank == 3` at every $K \in 1..=16$<br>  - `cell_rank_hinge` across $K = 1..=16$: **`[2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`** (`objective.rs:88`)<br>  - `target_admitted == (admitted_cells - address_rivals)`: **`[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`**<br>  - `weighted_cell_rank_hinge_sum`: `32 * R = [64, 32, 0, ..., 0]`<br>• **Fixture 1B ($K = 16$, 256 occupied cells):**<br>  - `occupied_cells == 256`<br>  - `admitted_cells == 16`, `address_rivals == 16`, `target_admitted == 0`<br>  - `target_cell_rank == 256` (`87` cells at $d \in \{0, 4, 8, 12\}$ + `168` tied cells at $d = 16$ with `cell_id < 255` precede `cell 255`)<br>  - `cell_rank_hinge == 240` (`256 - 16`, `objective.rs:89`)<br>  - `weighted_cell_rank_hinge_sum == 32 * 240 = 7,680` | **PASS** iff on GPU:<br>1. `R == 0` iff `target_cell ∈ admitted[0..K]` across all $K \in 1..=16$;<br>2. `R == target_cell_rank - K > 0` whenever `target_cell ∉ admitted[0..K]`;<br>3. Ties in `address_distance` break strictly by ascending `cell_id`;<br>4. The 256-cell boundary yields exact `target_cell_rank == 256` and `cell_rank_hinge == 240` at $K = 16$. |
| **`C-R2`** `single_count_per_occupied_cell_invariance` | Sheet line 91; `objective.rs:93-98`; `discrete.cu:291-334` | • **Part 2A (crowding an already-occupied preceding rival cell with $m \in \{1, 2, 4\}$ tokens):** Using Fixture 1A-i (`Spec { vocabulary: 16, blocks: 4, context: 3, kept_cells: K }` for $K \in \{1, 2, 3\}`, `occupied_cells = {0, 2, 15, 32, 255}`, `target_token = 2` in `target_cell = 15`, `target_cell_rank = 3`), vary the token multiplicity $m \in \{1, 2, 4\}$ in preceding rival `cell 2` (`[r0, r0, r0, r2]`, $d = 4 < (4, 15)$) by reassigning $m - 1$ spare tokens (`tokens 4..(3+m)`) from `cell 255` into `cell 2` while `tokens (3+m)..16` remain in `cell 255`. Execute GPU `evaluate` at $K = 1, 2, 3$.<br>• **Part 2B (moving one token into a newly occupied preceding cell):** Starting from Fixture 1A-ii ($q = [\text{root } 3; 4]$, `occupied_cells = {0, 1, 15, 16, 255}` all at $d = 16$, `target_cell = 15`, `target_cell_rank = 3`), move a single spare token (`token 4`, leaving `tokens 5..16` in `cell 255`) into previously empty `cell 2` (`[r0, r0, r0, r2]`, where $d(q, \text{cell } 2) = 8 + 8 = 16$ and `cell_id = 2 < 15`) and re-run GPU `evaluate` at $K \in \{1, 2, 3, 4\}$. | • **Part 2A (crowding rival `cell 2` with $m \in \{1, 2, 4\}$ tokens):**<br>  - `occupied_cells == [5, 5, 5]`<br>  - `target_cell_rank == [3, 3, 3]` (`objective.rs:95`)<br>  - `cell_rank_hinge` at $K = 1$: **`[2, 2, 2]`**; at $K = 2$: **`[1, 1, 1]`**; at $K = 3$: **`[0, 0, 0]`**<br>  - `admitted_token_count` at $K = 3$ (admitted cells `{0, 2, 15}`): **`[3, 4, 6]`** (grows by $+1, +3$ while `target_cell_rank` and `R` remain strictly invariant)<br>• **Part 2B (single token moved into newly occupied preceding `cell 2`):**<br>  - `occupied_cells` increases by `+1` (`5 -> 6`, `{0, 1, 2, 15, 16, 255}`)<br>  - `target_cell_rank` increases by **at most `+1` (`3 -> 4`)** (`objective.rs:96`) | **PASS** iff:<br>1. Adding $m \in \{1, 2, 4\}$ tokens to an already-occupied rival cell produces zero change (`0` delta) in GPU `target_cell_rank` and `cell_rank_hinge`;<br>2. Moving a single token into a newly occupied cell changes `target_cell_rank` by $\Delta r \in \{-1, 0, +1\}$ (and $\Delta R$ by at most $1$). |
| **`C-R3`** `virtual_vs_applied_candidate_identity_on_rank_and_occupancy` | Sheet line 92; `objective.rs:100-104`; `discrete.cu:376-423, 449-504` | Construct 5 disposable GPU snapshots (`Spec { vocabulary: 16, blocks: 4, context: 3, kept_cells: 4 }`) covering all 5 mutation classes in `objective.rs:100`:<br>1. `create_cell`: `Parameter::VectorBlock(flat_index)` where `flat_index = token * 4 + block` (`block ∈ {0, 3}`) moves a shared-cell token into a previously empty cell ($|\text{occ}| \to |\text{occ}| + 1$);<br>2. `empty_cell`: `Parameter::VectorBlock(token * 4 + block)` (`block ∈ {0, 3}`) moves the sole token of an occupied cell into an already-occupied cell ($|\text{occ}| \to |\text{occ}| - 1$);<br>3. `move_target_cell`: `Parameter::VectorBlock(target_token * 4 + block)` (`block ∈ {0, 3}`), directly mutating `target_cell` and `ad`;<br>4. `interior_block_only`: `Parameter::VectorBlock(token * 4 + block)` with interior `block ∈ {1, 2}` at `blocks = 4`, mutating interior blocks while `token_cell` endpoints `0` and `3` are untouched;<br>5. `identity_candidate`: `candidate = 0` (`move_action(g, 0) = g[IDENTITY]`) on both `Parameter::VectorBlock(flat_index)` and `Parameter::Route(flat_index)` (`flat_index = block * stages + stage`).<br>For each fixture and each policy (`NearestAddressMarginV1` and `OccupiedCellRankHingeV1`), run `model.propose(&batch, &targets, param)` on GPU to obtain `proposal.candidate_losses()?[0..16]`, and compare against GPU `evaluate` after applying each candidate on an isolated disposable fixture clone. | • Across all `5` mutation classes $\times$ `2` policies $\times$ `16` candidate ordinals (`160` candidate evaluations):<br>  - `proposal.candidate_losses()?[c] == post_apply_eval.objective_loss_numerator_sum` (`loss_ta_numerator_sum` under `NearestAddressMarginV1`, `loss_tr_numerator_sum` under `OccupiedCellRankHingeV1`)<br>  - `proposal.baseline_loss() == pre_apply_eval.objective_loss_numerator_sum`<br>  - In class `4` (`interior_block_only`, `block ∈ {1, 2}` at `B = 4`), across all 16 candidates: `occupied_cells`, `target_cell`, `target_cell_rank`, `cell_rank_hinge`, and `address_hinge` are **100% invariant (`delta == 0`)**, while only `token_distance` and `token_hinge` change.<br>  - In class `5` (`identity_candidate`, `c = 0`): `candidate_losses()[0] == baseline_loss()`. | **PASS** iff virtual counterfactual loss in `discrete_counterfactual` equals post-apply `discrete_evaluate` objective loss (`8*T + 32*A` or `8*T + 32*R`) to the integer across all 16 candidates and all 5 mutation classes (`create_cell`, `empty_cell`, `move_target_cell`, `interior_block_only`, `identity_candidate`). |
| **`C-R4`** `absent_rival_and_occupancy_under_k_bounds` | Sheet line 93; `objective.rs:106-112`; `discrete.cu:380-399, 449-504` | • **Fixture 4A (`occupied_cells <= K`):** At $K = 16$ (`Spec { vocabulary: 16, blocks: 4, context: 1, kept_cells: 16 }`, valid under `Spec::validate` since `kept_cells == vocabulary == 16`), test (i) `M = 1` occupied cell (all `16` tokens `0..16` placed in `cell 0` with `block 0 = r0, block 3 = r0` and distinct interior blocks `1, 2`) and (ii) `M = 16` occupied cells (`16` tokens placed one per cell in `cells 0..16`). Run GPU `evaluate` over targets in every occupied cell.<br>• **Fixture 4B (`tr == 0` row skipping & all-ineligible status `8` refusal):**<br>  - *4B-i (mixed batch):* 2-row batch where row `0` has `token_rivals == 0` (only the target token resides in the admitted cell set) and row `1` has `token_rivals >= 1`. Run `model.propose` under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1` and compare against a 1-row batch containing only row `1`.<br>  - *4B-ii (all-ineligible batch):* 1-row batch where `token_rivals == 0`. Call `model.propose` under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1`. | • **Fixture 4A (`M = 1` and `M = 16` at $K = 16, V = 16$):**<br>  - For `M = 1`: `occupied_cells == 1`, `target_cell_rank == 1`, `cell_rank_hinge == 0`, `target_admitted == 1` (`objective.rs:108`).<br>  - For `M = 16`: `occupied_cells == 16`, `target_cell_rank ∈ 1..=16`, `cell_rank_hinge == 0`, `target_admitted == 1` on every row.<br>• **Fixture 4B:**<br>  - *4B-i:* Row `0` (`tr == 0`) contributes `0` to `candidate_losses()[0..16]` under both policies (`candidate_losses(row0 + row1) == candidate_losses(row1)`).<br>  - *4B-ii:* Both policies return `Err(Error::InvalidModel("discrete CUDA counterfactual refused with status 8"))` (`expected_counterfactual_refusal_status: 8`, `objective.rs:109`). | **PASS** iff:<br>1. Whenever `occupied_cells <= K`, `target_cell_rank <= occupied_cells <= K`, `cell_rank_hinge == 0`, and `target_admitted == 1` on every row;<br>2. Rows with `tr == 0` are skipped identically under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1`, and a batch with zero `tr > 0` rows refuses with CUDA status `8` under both policies. |
| **`C-R5`** `bounded_integer_arithmetic_dual_loss_and_distribution_schema` | Sheet line 94; `objective.rs:114-120, 130-427`; `discrete.cu:505-569` | • **Fixture 5A (Dual-loss, exact-ratio, and histogram/quantile verification on GPU):** Run `Device::evaluate` + `Device::aggregate_evaluations` (`discrete_evaluate` + `discrete_evaluation_aggregate`) under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1` on a multi-row GPU fixture spanning $T \in [0, 640]$, $A \in [0, 72]$, $R \in [0, 240]$ at $B = 4, \text{width} = 32, K = 16$.<br>• **Fixture 5B (CUDA status `12` bounds & overflow refusal):** Invoke `discrete_evaluation_aggregate` with malformed row words testing each guard in `discrete.cu:513-569`: (i) `token_hinge = 641` ($> 640$), (ii) `address_hinge = 73` ($> 72$), (iii) `cell_rank_hinge = 256` ($> 255$), (iv) `target_cell_rank = 0` or `257`, (v) `occupied_cells = 0` or `257`, (vi) `admitted_cells = 0` or `17`, (vii) mismatched `loss_ta != 8*T + width*A` or `loss_tr != 8*T + width*R`, and (viii) cross-row `occupied_cells` mismatch. | • **Fixture 5A:**<br>  - `schema == "uor-r4.discrete-objective-evaluation/1"`<br>  - `ROW_WORDS == 14`, `AGGREGATE_WORDS == 1020`, `HISTOGRAM_BINS == [641, 73, 256]`<br>  - `weighted_token_hinge_sum == 8 * token_hinge_sum`<br>  - `weighted_address_hinge_sum == 32 * address_hinge_sum`<br>  - `weighted_cell_rank_hinge_sum == 32 * cell_rank_hinge_sum`<br>  - `loss_ta_numerator_sum == weighted_token_hinge_sum + weighted_address_hinge_sum == total_loss_numerator_sum`<br>  - `loss_tr_numerator_sum == weighted_token_hinge_sum + weighted_cell_rank_hinge_sum`<br>  - `objective_loss_numerator_sum == loss_ta_numerator_sum` (`NearestAddressMarginV1`) or `loss_tr_numerator_sum` (`OccupiedCellRankHingeV1`)<br>  - `address_loss_share_ta == { numerator: weighted_address_hinge_sum, denominator: loss_ta_numerator_sum }`<br>  - `rank_loss_share_tr == { numerator: weighted_cell_rank_hinge_sum, denominator: loss_tr_numerator_sum }`<br>  - `mean_admitted_token_count == { numerator: admitted_token_count_sum, denominator: scored_rows }`<br>  - For each of `token_hinge`, `address_hinge`, `cell_rank_hinge`: $\sum \text{bins} == \text{scored\_rows}$, `zero_rows == bins[0]`, and `min, p25, p50, p75, p90, p99, max` equal exact nearest-rank $\lceil p \cdot N / 100 \rceil$ bin indices.<br>• **Fixture 5B:** Every out-of-bounds/inconsistent input returns `Err(Error::InvalidModel("discrete CUDA evaluation aggregation refused with status 12"))` (`expected_aggregate_overflow_status: 12`, `objective.rs:119`). | **PASS** iff:<br>1. All dual-loss sums, exact ratios, occupancy fields, and `[641, 73, 256]` histogram/quantile fields match exact `i32`/`u64` integer identities on GPU under both policies;<br>2. All 8 out-of-bound / inconsistent row tests are refused by `discrete_evaluation_aggregate` with CUDA status `12`. |
| **`C-R6`** `k16_baseline_guard_and_untrained_ceiling_check` | Sheet line 95; `objective.rs:122-127`; `completion_baseline.rs:70, 567-611`; `redirect_admission.rs:374-452` | 1. **Stored baseline guard (`admitted_cells ∈ 1..=16`):** Verify `completion_baseline.rs:70` admits `admitted_cells = 16` on the sealed 36-point `K ∈ {8..16}` Stage-A sweep (`report.json` SHA-256 `cc81ca6f76f3a38ac61ffe1d4665a1442d1e42961f4451691327b91b438f472c`) and refuses `admitted_cells = 0` or `admitted_cells = 17`.<br>2. **Zero-update `G-Pre` GPU corpus check:** Run Section 2 below (`init-corpus` + `eval-corpus` at `accepted_updates == 0` for `seed ∈ {1, 2}`, `K ∈ {8, 16}`). | • `admitted_cells_range == [1, 16]` (`objective.rs:124`)<br>• `g_pre_checkpoint0_k16_target_admitted_bar == 4,894` (`objective.rs:125`, `redirect_admission.rs:449`)<br>• At `seed = 1, K = 8, cursor = 0`: `target_admitted_rows == 5,680`, `top1_correct == 17`, `top8_correct == 173`, `token_strict_win_eligible_rows == 105`, `occupied_cells == 16`<br>• At `seed ∈ {1, 2}, K = 16, cursor = 0`: `target_admitted_rows >= 4,894` on both seeds (`G-Pre PASS`) | **PASS** iff:<br>1. `completion_baseline.rs` accepts `1..=16` and rejects `0` and `>= 17`;<br>2. `seed = 1, K = 8` checkpoint `0` reproduces the sealed pilot checkpoint-0 counts to the digit;<br>3. Both `seed = 1` and `seed = 2` at `K = 16` checkpoint `0` achieve `target_admitted_rows >= 4,894` with `0` `.apply` updates. |

---

## 2. `G-Pre` — Untrained `K=16` Checkpoint-0 Zero-Update Ceiling Evaluation (`bar >= 4,894`)

### 2.1 Purpose & Governing Requirements

Per **Fact 4**, **Fact 5**, Amendment 5, and the owner's explicit confirmation ([#2515 comment 6198442894](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198442894)):
1. **Capacity was trained out (`5,680` at `K=8` checkpoint `0` $\to$ `2,180` at cursor `1,640` $\to$ `1,920` at cursor `16,404`), not absent.** Before any `K=16` training lease is executed, `G-Pre` measures the untrained checkpoint-0 `K=16` admission ceiling (`0` updates applied) across **`seed = 1` and `seed = 2` at `K = 8` and `K = 16`** on the 400-document `ft/dev` split (`14,287` eligible target rows).
2. **`G-Pre` release prerequisite bar (`redirect_admission.rs:447-451`):** Both `seed = 1` and `seed = 2` at `K = 16` checkpoint `0` must satisfy:
   $$\text{target\_admitted\_rows}(K=16, t=0) \ge 4,894 \quad (\text{out of } 14,287 \text{ eligible rows}).$$
   - **Analytic checkpoint-0 identity in `discrete_init` (`discrete.cu:353-355`):** At initialization, `discrete_init` computes `U block = i / 8; U state = seed ^ block ^ 0x9e3779b9u; state ^= state << 13; state ^= state >> 17; state ^= state << 5; table[i] = (int)g[ROOT + (state & 15) * 8 + (i & 7)];` (XOR with `0x9e3779b9u` followed by 32-bit xorshift, no multiplication). Because `blocks = 4`, for every token $v \in 0..4096$, `block 0` uses index `block_0 = 4*v` and `block 3` uses index `block_3 = 4*v + 3 = (4*v) ^ 3`, so `state_3 = state_0 ^ 3`. By $\mathbb{F}_2$-linearity of `xorshift32`, `(xorshift32(state_0 ^ 3) & 15) == (xorshift32(state_0) & 15) ^ (xorshift32(3) & 15)`, which pairs each first-block root $a \in 0..15$ deterministically with a unique last-block root $b(a) \in 0..15$, populating **at most `16` occupied endpoint cells across all `4,096` tokens at checkpoint `0`** (`occupied_cells == 16`, as already observed in the sealed `seed = 1, K = 8` checkpoint-0 receipt where `occupied_cells = 16` and `5,680 / 14,287` rows fell in the top `8` of `16` occupied cells). Consequently, at `K = 16` checkpoint `0`, all `<= 16` occupied cells are admitted (`admitted_cells == occupied_cells`), yielding `target_cell_rank <= 16`, `cell_rank_hinge_sum == 0`, and **`target_admitted_rows == 14,287 >= 4,894`**.
3. **Re-measuring the `K=16` checkpoint-0 strict-win baseline `u` (`Fact 4`):** Record `token_strict_win_eligible_rows` and `admitted_strict_win_rows` at checkpoint `0` for each `(seed, arm)` (`k16-A-seed1`, `k16-R-seed1`, `k16-A-seed2`, `k16-R-seed2`). This sets each arm's checkpoint-0 baseline rate $u = \text{token\_strict\_win\_eligible\_rows}(t=0) / 14,287$ and the one-sided 99% binomial guard threshold $\lceil 14,287 u + 2.326348 \sqrt{14,287 u (1 - u)} \rceil$ required by `G-Guard`.

### 2.2 Verbatim `DriverConfig` Contract & `G-Pre` Commands (`init-corpus` + `eval-corpus`, Zero `.apply`)

#### Source Contract Citations (`crates/uor-r4-discrete`)
- **`DriverConfig` (`crates/uor-r4-discrete/src/training.rs:36-84`):** Annotated with `#[serde(deny_unknown_fields)]`; requires:
  - `schema`: `"uor-r4.discrete-corpus-driver/1"` (`DRIVER_SCHEMA`, `training.rs:14, 39, 58`)
  - `spec`: `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 | 16 }` (`crates/uor-r4-discrete/src/lib.rs:38-59`, `training.rs:40, 57`)
  - `read_spec`: `{"kind": "route"}` (`ReadSpec`, `#[serde(tag = "kind", rename_all = "snake_case")]` at `crates/uor-r4-discrete/src/lib.rs:67-95`, `training.rs:41, 57`)
  - `objective_policy`: `"nearest_address_margin_v1"` or `"occupied_cell_rank_hinge_v1"` (`ObjectivePolicy`, `crates/uor-r4-discrete/src/objective.rs:13-33`, `training.rs:42-46`)
  - `seed`: `1` or `2` (`u32`, `training.rs:47`, `seed != 0` enforced in `Device::new_with_objective` at `device.rs:212`)
  - `batch_rows`: `64` (`u32`, validated range `1..=64` at `training.rs:48, 59`)
  - `train`: `CorpusSource` (`crates/uor-r4-discrete/src/corpus.rs:40-47`, `training.rs:49, 60, 68-69, 76`, `split: "train"`)
  - `dev`: `CorpusSource` (`crates/uor-r4-discrete/src/corpus.rs:40-47`, `training.rs:50, 61, 70-71, 76`, `split: "dev"`)
  - `tokenizer`: `FilePin` (`crates/uor-r4-discrete/src/corpus.rs:15-19`, `training.rs:51, 72`)
  - `bos`: `BosIdentity { token: "<|bos|>", id: 0 }` (`crates/uor-r4-discrete/src/corpus.rs:21-25`, `training.rs:52, 79-81`)
  - `schedule`: `"canonical_vector_then_read_v1"` (`SchedulePolicy::CanonicalVectorThenReadV1`, `training.rs:31-35, 53`)
- **`init-corpus CONFIG NEW_REPORT_DIR` (`crates/uor-r4-discrete/src/main.rs:9-28, 71-80`):** Calls `PreparedDriver::load(config)` (`training.rs:157-162`), `driver.new_model(0)` (`training.rs:786-794`), and `driver.checkpoint(&model, Cursor::initial())` (`training.rs:795-813`), and writes the serialized `DriverCheckpoint` (`CHECKPOINT_SCHEMA = "uor-r4.discrete-driver-checkpoint/1"`, `training.rs:15, 139-146`) to **`NEW_REPORT_DIR/report.json`**.
- **`eval-corpus CONFIG DRIVER_CHECKPOINT NEW_REPORT_DIR` (`crates/uor-r4-discrete/src/main.rs:9-28, 81-93`):** Reads `DRIVER_CHECKPOINT` (i.e., `INIT_REPORT_DIR/report.json`), enforces `checkpoint.cursor.accepted_updates == 0` (`main.rs:87-89`, returning `Err(Error::TrainingHeld)` if non-zero), calls `driver.restore(checkpoint, 0)` (`training.rs:814-825`) and `driver.evaluate(&model)` (`training.rs:913-964`), and writes the evaluation report (`schema = "uor-r4.discrete-corpus-driver/1"`, `objective_evaluation_schema = "uor-r4.discrete-objective-evaluation/1"`) to **`NEW_REPORT_DIR/report.json`**.

#### Pinned `#2523` `DriverConfig` Structure (`docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed1.json`)

```json
{
  "schema": "uor-r4.discrete-corpus-driver/1",
  "spec": {
    "vocabulary": 4096,
    "blocks": 4,
    "context": 192,
    "kept_cells": 16
  },
  "read_spec": {
    "kind": "route"
  },
  "objective_policy": "occupied_cell_rank_hinge_v1",
  "seed": 1,
  "batch_rows": 64,
  "train": {
    "split": "train",
    "tokens": {
      "path": "local/driver/data/mw-balp/train/tokens.u16",
      "sha256": "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
    },
    "mask": {
      "path": "local/driver/data/mw-balp/train/response_mask.u8",
      "sha256": "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
    },
    "expected_payload_tokens": 7512797,
    "expected_eligible_targets": 2425925
  },
  "dev": {
    "split": "dev",
    "tokens": {
      "path": "local/driver/data/ft/dev/tokens.u16",
      "sha256": "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
    },
    "mask": {
      "path": "local/driver/data/ft/dev/response_mask.u8",
      "sha256": "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
    },
    "expected_payload_tokens": 42856,
    "expected_eligible_targets": 14287
  },
  "tokenizer": {
    "path": "local/driver/data/tokenizer.json",
    "sha256": "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
  },
  "bos": {
    "token": "<|bos|>",
    "id": 0
  },
  "schedule": "canonical_vector_then_read_v1"
}
```

#### Verbatim `G-Pre` Execution Script (Reusing `#2523` Configs + Derived `K=8` Controls)

```bash
set -euo pipefail
cd /workspace/uor-r4

OUT_DIR="${1:?usage: run-g-pre.sh OUT_DIR}"
mkdir -p "${OUT_DIR}/configs"

PACKET_CFG_DIR="docs/labs/admission-redirect-packet-2026-10-15/configs"

# 1. Copy the four SHA-pinned K=16 configs from #2523 verbatim
for arm_seed in k16-A-seed1 k16-A-seed2 k16-R-seed1 k16-R-seed2; do
  cp "${PACKET_CFG_DIR}/${arm_seed}.json" "${OUT_DIR}/configs/${arm_seed}.json"
done

# 2. Derive the two K=8 checkpoint-0 control configs by changing only .spec.kept_cells = 8
jq '.spec.kept_cells = 8' "${PACKET_CFG_DIR}/k16-A-seed1.json" > "${OUT_DIR}/configs/k8-A-seed1.json"
jq '.spec.kept_cells = 8' "${PACKET_CFG_DIR}/k16-A-seed2.json" > "${OUT_DIR}/configs/k8-A-seed2.json"

BIN="./target/release/discrete-stack"

# 3. Record build/config/lineage identities without creating a CUDA context (main.rs:53-65)
for run in k16-A-seed1 k16-A-seed2 k16-R-seed1 k16-R-seed2; do
  "${BIN}" completion-build-identity "${OUT_DIR}/configs/${run}.json" \
    > "${OUT_DIR}/build-identity-${run}.json"
done

# 4. Run zero-update init-corpus (writes INIT_DIR/report.json) and eval-corpus (writes EVAL_DIR/report.json)
for run in k8-A-seed1 k16-A-seed1 k16-R-seed1 k8-A-seed2 k16-A-seed2 k16-R-seed2; do
  "${BIN}" init-corpus "${OUT_DIR}/configs/${run}.json" "${OUT_DIR}/g-pre-${run}-init"
  "${BIN}" eval-corpus "${OUT_DIR}/configs/${run}.json" \
    "${OUT_DIR}/g-pre-${run}-init/report.json" \
    "${OUT_DIR}/g-pre-${run}-eval"
done
```

### 2.3 `G-Pre` Pass/Fail Verification Table

| Run ID | Seed | $K$ | Policy | Checkpoint / Eval Files | Expected `accepted_updates` / `updates_applied_by_evaluation` | Expected `occupied_cells` | Expected `target_admitted_rows` (`G-Pre` Bar) | Expected `top1` / `top8` / `strict_wins` |
|---|---|---|---|---|---|---|---|---|
| `k8-A-seed1` | `1` | `8` | `nearest_address_margin_v1` | `g-pre-k8-A-seed1-{init,eval}/report.json` | `0` / `0` | `16` | **`== 5,680`** (exact match to sealed pilot checkpoint-0 evaluation) | `top1 == 17`, `top8 == 173`, `token_strict_win_eligible_rows == 105`, `admitted_strict_win_rows == 0`, `address_hinge_sum == 261,824` |
| `k16-A-seed1` | `1` | `16` | `nearest_address_margin_v1` | `g-pre-k16-A-seed1-{init,eval}/report.json` | `0` / `0` | `16` | **`>= 4,894`** (analytically `14,287` since `occupied_cells = 16 <= K = 16`) | Record `u_seed1_k16 = token_strict_win_eligible_rows / 14,287`; `cell_rank_hinge_sum == 0` |
| `k16-R-seed1` | `1` | `16` | `occupied_cell_rank_hinge_v1` | `g-pre-k16-R-seed1-{init,eval}/report.json` | `0` / `0` | `16` | **`>= 4,894`** (`== target_admitted_rows(k16-A-seed1)`) | Identical to `k16-A-seed1` on `top1`, `top8`, `strict_wins`, `target_admitted_rows`, `T`, `A`, `R`, `loss_ta_numerator_sum`, `loss_tr_numerator_sum`; `objective_loss_numerator_sum == loss_tr_numerator_sum == weighted_token_hinge_sum` |
| `k8-A-seed2` | `2` | `8` | `nearest_address_margin_v1` | `g-pre-k8-A-seed2-{init,eval}/report.json` | `0` / `0` | `16` | Record `target_admitted_rows(seed2, K=8, t=0)` | Record `seed 2` `K=8` checkpoint-0 reference |
| `k16-A-seed2` | `2` | `16` | `nearest_address_margin_v1` | `g-pre-k16-A-seed2-{init,eval}/report.json` | `0` / `0` | `16` | **`>= 4,894`** (analytically `14,287` since `occupied_cells = 16 <= K = 16`) | Record `u_seed2_k16 = token_strict_win_eligible_rows / 14,287`; `cell_rank_hinge_sum == 0` |
| `k16-R-seed2` | `2` | `16` | `occupied_cell_rank_hinge_v1` | `g-pre-k16-R-seed2-{init,eval}/report.json` | `0` / `0` | `16` | **`>= 4,894`** (`== target_admitted_rows(k16-A-seed2)`) | Identical to `k16-A-seed2` on all row/ranking/dual-loss counts; `objective_loss_numerator_sum == loss_tr_numerator_sum` |

- **`G-Pre` Verdict Rule (`redirect_admission.rs:374-452`):**
  - **PASS:** `g-pre-k8-A-seed1-eval/report.json` matches the sealed pilot checkpoint-0 counts (`17 / 173 / 105 / 0 / 5,680 / 261,824 / 16`), all four `K=16` evaluations (`k16-{A,R}-seed{1,2}`) satisfy `target_admitted_rows >= 4,894`, and `updates_applied_by_evaluation == 0`, `accepted_updates_in_model == 0`, `evaluation_mutates_model == false` on all 6 evaluations.
  - **STOP (`G-Pre FAIL`):** If either `seed = 1` or `seed = 2` at `K = 16` checkpoint `0` has `target_admitted_rows < 4,894`, stop immediately before any `K = 16` training and report as an instrument/release blocker (`redirect_admission.rs:450`).

---

## 3. Mandatory Step-0 CUDA Build/Clippy Gate & 100-Batch `K=16` Synchronized Timing Benchmark

### 3.0 Mandatory Step-0 Pod-Side CUDA Build & Clippy Gate (`#2524` + `#2522` Comment `6202519677` + `redirect_admission.rs:19-22, 345-356`)

Before **any** timing or qualification execution on the pod, the executor must run `--features cuda` build and `clippy --all-targets` across both `crates/uor-r4-discrete/Cargo.toml` and the standalone timing instrument manifest [`docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`](../k16-timing-2026-10-15/instrument/Cargo.toml):

1. **Commands required on the pod (`UOR_NVCC_ARCH=compute_120`):**
   - `cargo build --release --locked --features cuda -p uor-r4-discrete`
   - `cargo build --release --locked --features cuda --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`
   - `cargo clippy --locked --features cuda --all-targets -p uor-r4-discrete`
   - `cargo clippy --locked --features cuda --all-targets --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`
2. **Evidence & stop rule (`redirect_admission.rs:19-22, 345-356`):**
   - Retain the complete compiler/linter transcripts (`${OUT_DIR}/cuda-build.log`, `${OUT_DIR}/cuda-clippy.log`) including exact commands, manifest paths, `nvcc --version`, `rustc --version`, and `git rev-parse HEAD`, along with the integer exit statuses `cuda_build_exit` and `cuda_clippy_exit`.
   - If `cuda_build_exit != 0` or `cuda_clippy_exit != 0`, **stop and report immediately** — do not run `G-Pre` or the timing benchmark. Laptop no-default-feature host tests do not discharge this gate.

### 3.1 Why a Dedicated `K=16` Dual-Occupancy Benchmark Is Required (`Fact 6`)

1. The existing `discrete-stack measure-cost NEW_REPORT_DIR` subcommand (`crates/uor-r4-discrete/src/measurement.rs:12-17`) is hardcoded to `SHAPE.kept_cells = 8` and times 3 repeats of a single synthetic batch at checkpoint-0 occupancy (`16` occupied cells).
2. In `discrete_counterfactual` (`crates/uor-r4-discrete/src/discrete.cu:376-399`) and `writer` (`discrete.cu:304-345`), each candidate ordinal (`candidate ∈ 0..16`, grid `(1, 16, 1)` or `blockIdx.y ∈ 0..16`) evaluates every baseline-eligible row (`tr > 0` at `discrete.cu:383`), runs `read_vector` (`discrete.cu:384, 391`) and `writer` (`discrete.cu:386, 392`), scans all `256` cells over `occupied` to admit up to `K = 16` cells (`discrete.cu:306-315`), scans all `4,096` vocabulary tokens (`discrete.cu:317-327`), and counts occupied cells preceding `target_cell` (`discrete.cu:329-334`).
3. Per **Fact 6** (`m1-discrete-rung0-admission-redirect-2026-10-15.md` lines 49, 102–112, 167), throughput and `K=16` wall-time must be sized from **100 synchronized batches at `K = 16` (`batch_rows = 64`, `context = 192`, `6,400` target rows, `1,228,800` physical padded context tokens)** across both **Regime 0 (`occupied_cells = 16`, checkpoint-0 init)** and **Regime Q4 (`occupied_cells = 189` from `checkpoint-16404.json`)** for both objective policies (`NearestAddressMarginV1` and `OccupiedCellRankHingeV1`), with **zero `.apply` updates**.

### 3.2 `#2524` Timing Instrument Contract, Candidate Ordinal Evaluation, and Coordinate Indexing (`0` `.apply` Calls)

1. **Merged `#2524` timing instrument (`docs/labs/k16-timing-2026-10-15/instrument/src/main.rs:28-120, 463-660`):**
   - Standalone binary `k16-timing` (`docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`) accepts `check --pin FILE`, `plan --pin FILE`, and `run --pin FILE` (`main.rs:669-729`) and writes `${out_dir}/k16-timing-record.json` (`schema = "k16-timing-record/v1"`, `main.rs:366, 733`).
   - Across each of the 4 cells (`source ∈ {checkpoint0, sealed_seed1_q4}` $\times$ `policy ∈ {nearest_address_margin_v1, occupied_cell_rank_hinge_v1}`), `run_cell` (`main.rs:463-660`) executes `1` excluded warmup batch followed by `100` timed batches (`ROWS = 64`, `PADDED_CONTEXT_TOKENS_PER_ROW = 192`).
   - Each timed batch synchronizes before and after each phase (`Device::synchronize()`, `main.rs:513-550`) and times the complete unit:
     $$\text{RATE\_BASIS} = \texttt{"full timed unit: carry\_windows + propose(route) + observation + propose(vector\_block) + observation"},$$
     reporting `carry_ns`, `route_propose_ns`, `route_observation_ns`, `vector_propose_ns`, `vector_observation_ns`, `unit_ns`, `rates.rows_per_s`, `rates.padded_context_tokens_per_s`, `carry_only`, and `per_family_ns_per_batch_floor` (`main.rs:610-647`), while asserting `before_digest == after_digest` and `epoch_before == epoch_after` (`main.rs:596-608`).
2. **Exact parameter indexing & 16-ordinal GPU counterfactual evaluation (`crates/uor-r4-discrete/src/training.rs:229-248`, `discrete.cu:23-30, 49-52, 291-300, 376-408`):**
   - In `PreparedDriver::coordinate(index)` (`training.rs:229-248`), the canonical `16,404`-coordinate cycle (`SchedulePolicy::CanonicalVectorThenReadV1`) maps `index ∈ 0..16384` to `Parameter::VectorBlock(index)` (`kind == 1` in `device.rs:251`, where `token = index / blocks`, `block = index % blocks`) and `index ∈ 16384..16404` to `Parameter::Route(index - 16384)` (`kind == 0` in `device.rs:252`, where `block = flat_index / stages`, `stage = flat_index % stages` at `discrete.cu:49` with `blocks = 4, stages = 5`).
   - In `Device::propose` (`device.rs:249-280`) and `discrete_counterfactual` (`discrete.cu:376-399`), the device evaluates **all `16` candidate ordinals (`candidate ∈ 0..16`, where `candidate == 0` is `CURRENT` via `move_action(g, 0) = g[IDENTITY]` and `candidate ∈ 1..16` are the `15` non-identity generators `g[GENERATOR + candidate - 1]`, `discrete.cu:23`)** for both families:
     - For `Parameter::Route(flat_index)` (`kind == 0`), `route_vector` composes `frame = compose(g, move_action(g, candidate), frame)` at `parameter == index` (`discrete.cu:52`), and `discrete_counterfactual` calls `read_vector` (`discrete.cu:384`) and `writer` (`discrete.cu:386`) using the baseline `occupied` cell mask on every baseline-eligible row (`tr > 0`).
     - For `Parameter::VectorBlock(flat_index)` (`kind == 1`), `table_block` / `input_block` applies `action(g, move_action(g, candidate), base, output)` to the selected `(token, block)` (`discrete.cu:27-34`), `occupied_cells_for_candidate` (`discrete.cu:291-300, 390`) recomputes the 256-cell `candidate_occupied` bitmap across the vocabulary, and `discrete_counterfactual` calls `read_vector` (`discrete.cu:391`) and `writer` (`discrete.cu:392`) on every baseline-eligible row (`tr > 0`).
   - When referencing the canonical schedule's `80` `VectorBlock` + `20` `Route` coordinate sample (`step_index ∈ 0..80` for `Parameter::VectorBlock(0..80)` and `step_index ∈ 16384..16404` for `Parameter::Route(0..20)`), it is labeled as a **two-slice family-stratified `80/20` coordinate sample** (never "100 consecutive canonical coordinates", since `step_index` jumps from `79` to `16384`).
3. **`sealed_seed1_q4` `CompletionCheckpoint` wrapper & cross-version evaluation-only rebind/import dependency (`crates/uor-r4-discrete/src/completion.rs:364`, `crates/uor-r4-discrete/src/device.rs:705-750`, `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs:425-459`):**
   - **Wrapper layout:** The sealed Q4 file `checkpoint-16404.json` (`4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`) is a `CompletionCheckpoint` (`schema = "uor-r4.completion-checkpoint/1"`, top-level keys `["schema", "admission_sha256", "pilot_parent", "checkpoint", "baseline_snapshot_metadata", "baseline_evaluation", "vector", "route"]`). The inner `Snapshot` lives at **`.checkpoint.model`** (with `.checkpoint.model_sha256 == 29ba5517a6b1b6d3c7ce54d593221560b446c5048a0b0db882ad53b5ec75bbbf` and `189` occupied cells), **not** `.model`.
   - **Cross-version `source_sha256` / `kernel_sha256` loader check:** The inner snapshot `.checkpoint.model` carries the historical Stage-A pilot `source_sha256 = "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a"` and `kernel_sha256 = "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b"`, whereas `Device::from_snapshot` (`crates/uor-r4-discrete/src/device.rs:722-735`, called by `k16-timing::driver::load_fixture` at `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs:458`) requires `snapshot.source_sha256 == hex_digest(&source_sha256())` and `snapshot.kernel_sha256 == kernel_sha256()` for the current compiled `D-R` binary.
   - **Reviewed cross-version evaluation-only rebind/import dependency (`NOT_EXECUTABLE` until merged):**
     - **Hard prohibition:** Never run an unreviewed `jq` rewrite of `source_sha256` or `kernel_sha256`, never silently relabel historical Stage-A weights as a native current-source checkpoint, and never weaken `Device::from_snapshot`'s native checkpoint loader checks.
     - **Required dependency:** Executing the `sealed_seed1_q4` timing leg (`T-K16A-OCC189`, `T-K16R-OCC189`) requires a **reviewed cross-version evaluation-only rebind/import path** (in `k16-timing` or an explicitly reviewed evaluation-only rebind helper) that:
       1. Authenticates the outer `CompletionCheckpoint` wrapper SHA-256 `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` and inner `.checkpoint.model_sha256 == 29ba5517a6b1b6d3c7ce54d593221560b446c5048a0b0db882ad53b5ec75bbbf`;
       2. Authenticates the historical pilot `source_sha256 = "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a"` and `kernel_sha256 = "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b"`, plus invariant `geometry_sha256 = "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327"`, `operator_kappa`, `composition_kappa`, and exact `table` (`131,072` `i8` coordinates, `189` occupied cells), `routes` (`20` `u8`), and `gates` (`20` `u8`);
       3. Preserves original provenance (`original_sha256`, `original_model_sha256`, `original_source_sha256`, `original_kernel_sha256`, `original_k = 8`, `evaluation_only_rebind = true`, `training_resume = false`) while loading the `189`-cell state into an evaluation-only `K=16, epoch=0` `Device` for zero-apply timing.
     - **Status on current `main`:** `checkpoint0` (`T-K16A-OCC16`, `T-K16R-OCC16`) and `k16-timing check` / `plan` are executable as written; the `sealed_seed1_q4` GPU timing leg (`T-K16A-OCC189`, `T-K16R-OCC189` via `k16-timing run`) is **gated / `NOT_EXECUTABLE` pending that reviewed cross-version evaluation-only rebind/import dependency**.

| Timing Cell ID | `#2524` `cell.source` | Occupancy Regime | Model State (`epoch = 0`, `accepted_updates = 0`, Zero `.apply`) | `#2524` `cell.policy` (`ObjectivePolicy`) | Timed Batches (`ROWS = 64`, `C = 192`, `1` Excluded Warmup) |
|---|---|---|---|---|---|
| **`T-K16A-OCC16`** | `"checkpoint0"` | Regime 0 (`occupied_cells = 16`, checkpoint-0 init) | Fresh `seed = 1` checkpoint-0 `Device::new_with_objective(K16, Route, NearestAddressMarginV1, 1, 0)` | `"nearest_address_margin_v1"` (`8*T + 32*A`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` padded context tokens) |
| **`T-K16R-OCC16`** | `"checkpoint0"` | Regime 0 (`occupied_cells = 16`, checkpoint-0 init) | Fresh `seed = 1` checkpoint-0 `Device::new_with_objective(K16, Route, OccupiedCellRankHingeV1, 1, 0)` | `"occupied_cell_rank_hinge_v1"` (`8*T + 32*R`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` padded context tokens) |
| **`T-K16A-OCC189`** | `"sealed_seed1_q4"` | Regime Q4 (`occupied_cells = 189`, Q4 sizing anchor per `Fact 6`) | Evaluation-only `K=16, epoch=0` rebind of `.checkpoint.model` (`29ba5517...`) inside sealed `checkpoint-16404.json` (`4477eb5c...`; **`NOT_EXECUTABLE` pending reviewed cross-version evaluation-only rebind/import helper**) | `"nearest_address_margin_v1"` (`8*T + 32*A`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` padded context tokens) |
| **`T-K16R-OCC189`** | `"sealed_seed1_q4"` | Regime Q4 (`occupied_cells = 189`, Q4 sizing anchor per `Fact 6`) | Evaluation-only `K=16, epoch=0` rebind of `.checkpoint.model` (`29ba5517...`) inside sealed `checkpoint-16404.json` (`4477eb5c...`; **`NOT_EXECUTABLE` pending reviewed cross-version evaluation-only rebind/import helper**) | `"occupied_cell_rank_hinge_v1"` (`8*T + 32*R`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` padded context tokens) |

### 3.3 Exact Integer Rate Formulas for `D-Packet` (`Zero Float`, `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs:284-309, 610-647`)

For each of the four timing cells in `k16-timing-record.json`, the instrument records integer nanoseconds (`u128`) and floor rates:

$$\begin{aligned}
\text{timed\_batches} &= 100, \quad \text{rows} = 64, \quad \text{padded\_context\_tokens\_per\_row} = 192 \\
\text{total\_rows} &= 100 \times 64 = 6,400, \quad \text{total\_padded\_context\_tokens} = 100 \times 64 \times 192 = 1,228,800 \\
\text{unit\_ns} &= \text{carry\_ns\_total} + \text{route\_propose\_ns} + \text{route\_observation\_ns} + \text{vector\_propose\_ns} + \text{vector\_observation\_ns} \\
\text{rows\_per\_s} &= \left\lfloor \frac{6,400 \times 1,000,000,000}{\text{unit\_ns}} \right\rfloor \\
\text{padded\_context\_tokens\_per\_s} &= \left\lfloor \frac{1,228,800 \times 1,000,000,000}{\text{unit\_ns}} \right\rfloor \\
\text{per\_family\_ns\_per\_batch\_floor} &= \left\{ \left\lfloor \frac{\text{carry\_ns\_total}}{100} \right\rfloor,\ \left\lfloor \frac{\text{route\_propose\_ns}}{100} \right\rfloor,\ \left\lfloor \frac{\text{route\_observation\_ns}}{100} \right\rfloor,\ \left\lfloor \frac{\text{vector\_propose\_ns}}{100} \right\rfloor,\ \left\lfloor \frac{\text{vector\_observation\_ns}}{100} \right\rfloor \right\} \\
\text{projected\_pass\_ns}(16,404\text{ batches}) &= \left\lceil \frac{16,404 \times \text{carry\_ns\_total} + 16,384 \times (\text{vector\_propose\_ns} + \text{vector\_observation\_ns}) + 20 \times (\text{route\_propose\_ns} + \text{route\_observation\_ns})}{100} \right\rceil
\end{aligned}$$

- **`D-Packet` Slot Rule (`Fact 6`):**
  - Populate the `D-Packet` `tokens/s` slot (`docs/labs/admission-redirect-packet-2026-10-15/README.md:99-111`) with the measured `k16-timing-record.json` rates across all 4 cells (`checkpoint0` and `sealed_seed1_q4` $\times$ `K16-A` and `K16-R`) and bound the `65,616`-batch (`4`-run) contrast wall-time from the slower of Regime 0 and Regime Q4 plus `20 × eval-corpus` wall time.

---

## 4. Pod Discipline — `uor-pod` Lifecycle, Single-GPU Locking, and `cloud-store put` MD5 Round-Trip Before Teardown

### 4.1 Hard Pod & Budget Rules (`docs/labs/compute.md`, `AGENTS.md`)

1. **Authorization & review gate:** Do **not** run `uor-pod up` or `uor-pod lease` until the qualification executors and runbook have passed exact-head review and merged to `origin/main` within the owner-authorized qualification scope ([#2029 comment 6201365250](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250), [#2029 comment 6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519)).
2. **Fleet & spend caps:** $\le 4$ live pods fleet-wide, $\le \$8.00/\text{h}$ fleet-wide (`1× RTX 5090` is $\$0.89/\text{h}$), reuse an existing idle 5090 pod before creating a new one (`uor-pod up` refuses if a free matching GPU already exists).
3. **One job per GPU:** Every GPU command must execute through `scripts/pod/uor-pod run POD --lab <lab> --session <session> --gpu 0 -- <cmd>`, which acquires `flock -n /root/gpu0.lock` on the pod and tees output to `/workspace/uor-r4/jobs/<lab>/<UTC>-<session>-gpu0.log` with a trailing `# exit=<rc> <UTC>` marker.
4. **Lease & heartbeat cadence:** Renew the harness heartbeat every $\le 5$ minutes (`UOR_LAB=<lab> uor-harness heartbeat -m m1`) and renew the pod lease every $\le 30$ minutes (`scripts/pod/uor-pod renew POD --lab <lab> --session <session> --hours 1`). Never cancel a healthy run because a self-set timer expired.
5. **Archive with MD5 round-trip BEFORE `release` / `down`:** Never release or terminate a pod until the sealed output directory has been copied to laptop staging (`~/uor-r4-local/<run-dir>`), verified via `shasum -a 256 -c SHA256SUMS`, and archived via `~/.local/share/uor-r4/bin/cloud-store put <lab> ~/uor-r4-local/<run-dir>` with a verified MD5 round-trip against `icloud:UOR-R4/results/<lab>/<run-dir>.tar`.

### 4.2 Verbatim Step-by-Step Pod Sequence

#### Step 4.2.1 — Pre-Flight Fleet Check & Lease Acquisition (Laptop)

```bash
export UOR_LAB=<lab>
export SESSION="dr-qual-20261015"
export REVIEWED_SHA="<exact-40-char-reviewed-main-sha>"

# 1. Record harness presence
/Users/casey.allard/uor-r4/local/harness-target/debug/uor-harness heartbeat -m m1

# 2. Inspect fleet & active leases (must be <= 4 pods, <= $8.00/h)
scripts/pod/uor-pod status

# 3a. If an idle RTX 5090 pod is already RUNNING with a free GPU 0, lease it:
scripts/pod/uor-pod lease <POD_ID> \
  --lab "${UOR_LAB}" \
  --session "${SESSION}" \
  --gpus 0 \
  --purpose "M1 D-R step-0 build/clippy + C-R1..C-R6 + G-Pre + 100-batch K16 timing (NOT_TRAINING, 0 apply)" \
  --hours 1 \
  --card "https://github.com/UOR-Foundation/uor-r4/issues/2029"

# 3b. Otherwise, if no free RTX 5090 exists, provision 1x RTX 5090 ($0.89/h):
scripts/pod/uor-pod up \
  --lab "${UOR_LAB}" \
  --session "${SESSION}" \
  --count 1 \
  --gpu 5090 \
  --ref "${REVIEWED_SHA}" \
  --purpose "M1 D-R step-0 build/clippy + C-R1..C-R6 + G-Pre + 100-batch K16 timing (NOT_TRAINING, 0 apply)" \
  --hours 1 \
  --card "https://github.com/UOR-Foundation/uor-r4/issues/2029"
```

#### Step 4.2.2 — Pod Bootstrap, Corpus/Fixture Staging, and Mandatory Step-0 CUDA Build & Clippy Gate (`uor-pod run`, GPU 0)

```bash
scripts/pod/uor-pod run "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --gpu 0 -- "
  set -euo pipefail
  cd /workspace/uor-r4
  git fetch origin
  git checkout --detach ${REVIEWED_SHA}
  test \"\$(git rev-parse HEAD)\" = \"${REVIEWED_SHA}\"

  export UOR_NVCC_ARCH=compute_120
  OUT_DIR=\"/workspace/uor-r4/evals/20261015T000000Z-dr-qualification-${REVIEWED_SHA:0:9}\"
  mkdir -p \"\${OUT_DIR}\"

  # Verify frozen DriverConfig data pins under local/driver/data/ (completion.rs:285-330)
  # and committed #2523 config file SHA-256 digests (§0)
  sha256sum -c <<'PINS'
d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89  local/driver/data/tokenizer.json
3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8  local/driver/data/mw-balp/train/tokens.u16
b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3  local/driver/data/mw-balp/train/response_mask.u8
0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  local/driver/data/ft/dev/tokens.u16
a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  local/driver/data/ft/dev/response_mask.u8
eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4  docs/labs/admission-redirect-packet-2026-10-15/configs/k16-A-seed1.json
bed46548bdb15e883386044442999584696e6cd9acddaf8c2cc5a8be6b578ee4  docs/labs/admission-redirect-packet-2026-10-15/configs/k16-A-seed2.json
25ef452f78cb612bdedd8135461836ec459d1f7c899e2e5940e830c3dffc61b7  docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed1.json
fe9f172b428ee131fac4a3865402f55ef80c3749ef32f1d98a830718ad361156  docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed2.json
PINS

  # STEP-0 GATE: Pod-side cargo build --features cuda + cargo clippy --features cuda --all-targets
  # (redirect_admission.rs:19-22, 345-356; #2522 comments 6202519677 & 6203506028)
  # Disable -e around each command so non-zero exit statuses and full transcripts are always
  # written to cuda-build.log / cuda-clippy.log and cuda_build_exit.txt / cuda_clippy_exit.txt
  # before halting on failure, and never overwrite the first command's failure with the second's status.
  set +e
  {
    echo \"# head=\$(git rev-parse HEAD)\"
    echo \"# UOR_NVCC_ARCH=\${UOR_NVCC_ARCH}\"
    echo \"+ nvcc --version\"
    nvcc --version
    echo \"+ rustc --version\"
    rustc --version
    echo \"+ cargo build --release --locked --features cuda --manifest-path crates/uor-r4-discrete/Cargo.toml\"
    cargo build --release --locked --features cuda \
      --manifest-path crates/uor-r4-discrete/Cargo.toml
    BUILD_DISCRETE_EXIT=\$?
    echo \"# exit(crates/uor-r4-discrete/Cargo.toml)=\${BUILD_DISCRETE_EXIT}\"
    echo \"+ cargo build --release --locked --features cuda --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml\"
    cargo build --release --locked --features cuda \
      --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml
    BUILD_TIMING_EXIT=\$?
    echo \"# exit(docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml)=\${BUILD_TIMING_EXIT}\"
  } > \"\${OUT_DIR}/cuda-build.log\" 2>&1
  if [ \"\${BUILD_DISCRETE_EXIT}\" -ne 0 ]; then
    CUDA_BUILD_EXIT=\"\${BUILD_DISCRETE_EXIT}\"
  else
    CUDA_BUILD_EXIT=\"\${BUILD_TIMING_EXIT}\"
  fi
  printf '%s\n' \"\${CUDA_BUILD_EXIT}\" > \"\${OUT_DIR}/cuda_build_exit.txt\"

  {
    echo \"# head=\$(git rev-parse HEAD)\"
    echo \"# UOR_NVCC_ARCH=\${UOR_NVCC_ARCH}\"
    echo \"+ cargo clippy --locked --features cuda --all-targets --manifest-path crates/uor-r4-discrete/Cargo.toml\"
    cargo clippy --locked --features cuda --all-targets \
      --manifest-path crates/uor-r4-discrete/Cargo.toml
    CLIPPY_DISCRETE_EXIT=\$?
    echo \"# exit(crates/uor-r4-discrete/Cargo.toml)=\${CLIPPY_DISCRETE_EXIT}\"
    echo \"+ cargo clippy --locked --features cuda --all-targets --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml\"
    cargo clippy --locked --features cuda --all-targets \
      --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml
    CLIPPY_TIMING_EXIT=\$?
    echo \"# exit(docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml)=\${CLIPPY_TIMING_EXIT}\"
  } > \"\${OUT_DIR}/cuda-clippy.log\" 2>&1
  if [ \"\${CLIPPY_DISCRETE_EXIT}\" -ne 0 ]; then
    CUDA_CLIPPY_EXIT=\"\${CLIPPY_DISCRETE_EXIT}\"
  else
    CUDA_CLIPPY_EXIT=\"\${CLIPPY_TIMING_EXIT}\"
  fi
  printf '%s\n' \"\${CUDA_CLIPPY_EXIT}\" > \"\${OUT_DIR}/cuda_clippy_exit.txt\"
  set -e

  if [ \"\${CUDA_BUILD_EXIT}\" -ne 0 ] || [ \"\${CUDA_CLIPPY_EXIT}\" -ne 0 ]; then
    echo \"STEP-0 GATE FAILED: cuda_build_exit=\${CUDA_BUILD_EXIT} cuda_clippy_exit=\${CUDA_CLIPPY_EXIT}\" >&2
    exit 1
  fi
"
```

#### Step 4.2.3 — Execute Finite GPU Suites, `C-R1..C-R6`, `G-Pre`, and 100-Batch `K=16` Timing (`uor-pod run`, GPU 0)

```bash
scripts/pod/uor-pod run "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --gpu 0 -- "
  set -euo pipefail
  cd /workspace/uor-r4
  export UOR_NVCC_ARCH=compute_120
  OUT_DIR=\"/workspace/uor-r4/evals/20261015T000000Z-dr-qualification-${REVIEWED_SHA:0:9}\"
  BIN=\"./target/release/discrete-stack\"
  TIMING_BIN=\"./docs/labs/k16-timing-2026-10-15/instrument/target/release/k16-timing\"

  # 0. Verify Step-0 CUDA build & clippy gate succeeded before any GPU execution
  test -s \"\${OUT_DIR}/cuda-build.log\"
  test -s \"\${OUT_DIR}/cuda-clippy.log\"
  test \"\$(cat \"\${OUT_DIR}/cuda_build_exit.txt\")\" = \"0\"
  test \"\$(cat \"\${OUT_DIR}/cuda_clippy_exit.txt\")\" = \"0\"

  # 1. Existing finite GPU suites (each takes NEW_REPORT_DIR and writes NEW_REPORT_DIR/report.json; main.rs:66-69)
  \"\${BIN}\" qualify          \"\${OUT_DIR}/qualify\"
  \"\${BIN}\" qualify-flock    \"\${OUT_DIR}/qualify-flock\"
  \"\${BIN}\" qualify-relation \"\${OUT_DIR}/qualify-relation\"
  \"\${BIN}\" qualify-driver   \"\${OUT_DIR}/qualify-driver\"

  # 2. C-R1..C-R6 GPU fixture receipts (redirect_admission.rs:18, 357-367)
  # NOTE: Requires the reviewed GPU C-R1..C-R6 fixture runner producing \${OUT_DIR}/c-r{1..6}/report.json.
  # Host objective::tests are static contract checks only and must NEVER be substituted for GPU receipts.
  for idx in 1 2 3 4 5 6; do
    test -s \"\${OUT_DIR}/c-r\${idx}/report.json\"
  done

  # 3. G-Pre zero-update checkpoint-0 ceiling evaluations (Section 2.2: seed 1 & seed 2, K=8 & K=16)
  mkdir -p \"\${OUT_DIR}/configs\"
  PACKET_CFG_DIR=\"docs/labs/admission-redirect-packet-2026-10-15/configs\"
  for arm_seed in k16-A-seed1 k16-A-seed2 k16-R-seed1 k16-R-seed2; do
    cp \"\${PACKET_CFG_DIR}/\${arm_seed}.json\" \"\${OUT_DIR}/configs/\${arm_seed}.json\"
    \"\${BIN}\" completion-build-identity \"\${OUT_DIR}/configs/\${arm_seed}.json\" \
      > \"\${OUT_DIR}/build-identity-\${arm_seed}.json\"
  done
  jq '.spec.kept_cells = 8' \"\${PACKET_CFG_DIR}/k16-A-seed1.json\" > \"\${OUT_DIR}/configs/k8-A-seed1.json\"
  jq '.spec.kept_cells = 8' \"\${PACKET_CFG_DIR}/k16-A-seed2.json\" > \"\${OUT_DIR}/configs/k8-A-seed2.json\"

  for run in k8-A-seed1 k16-A-seed1 k16-R-seed1 k8-A-seed2 k16-A-seed2 k16-R-seed2; do
    \"\${BIN}\" init-corpus \"\${OUT_DIR}/configs/\${run}.json\" \"\${OUT_DIR}/g-pre-\${run}-init\"
    \"\${BIN}\" eval-corpus \"\${OUT_DIR}/configs/\${run}.json\" \
      \"\${OUT_DIR}/g-pre-\${run}-init/report.json\" \
      \"\${OUT_DIR}/g-pre-\${run}-eval\"
  done

  # 4. 100-batch K=16 synchronized timing benchmark (#2524 k16-timing instrument, 0 apply)
  # Authenticate sealed CompletionCheckpoint wrapper (4477eb5c...) and inner .checkpoint.model (29ba5517...)
  SEALED_Q4=\"/workspace/uor-r4/local/pilot-seed1/checkpoints/checkpoint-16404.json\"
  test \"\$(sha256sum \"\${SEALED_Q4}\" | awk '{print \$1}')\" = \"4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf\"
  test \"\$(jq -r '.checkpoint.model_sha256' \"\${SEALED_Q4}\")\" = \"29ba5517a6b1b6d3c7ce54d593221560b446c5048a0b0db882ad53b5ec75bbbf\"
  test \"\$(jq -r '.checkpoint.model.source_sha256' \"\${SEALED_Q4}\")\" = \"e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a\"
  test \"\$(jq -r '.checkpoint.model.kernel_sha256' \"\${SEALED_Q4}\")\" = \"931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b\"

  # CROSS-VERSION REBIND GATE (§3.2 item 3):
  # .checkpoint.model carries historical Stage-A pilot source_sha256/kernel_sha256, whereas
  # Device::from_snapshot (device.rs:722-735, called by k16-timing::driver::load_fixture at
  # instrument/src/main.rs:458) requires current compiled D-R source_sha256/kernel_sha256.
  # Do NOT rewrite source_sha256/kernel_sha256 via jq or weaken Device::from_snapshot.
  # The sealed_seed1_q4 timing leg is NOT_EXECUTABLE until the reviewed cross-version
  # evaluation-only rebind/import helper (preserving original provenance and exact
  # table/routes/gates/geometry_sha256/operator_kappa/composition_kappa) has landed and been
  # invoked to produce \${OUT_DIR}/seed1-q4-k16-rebind-snapshot.json.
  REBIND_SNAP=\"\${OUT_DIR}/seed1-q4-k16-rebind-snapshot.json\"
  test -s \"\${REBIND_SNAP}\"
  REBIND_SHA=\"\$(sha256sum \"\${REBIND_SNAP}\" | awk '{print \$1}')\"

  jq --arg snap \"\${REBIND_SNAP}\" \
     --arg snap_sha \"\${REBIND_SHA}\" \
     --arg orig_sha \"4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf\" \
     --arg out \"\${OUT_DIR}\" \
     '.fixture.snapshot_path = \$snap
      | .fixture.snapshot_sha256 = \$snap_sha
      | .fixture.original_sha256 = \$orig_sha
      | .out_dir = \$out' \
     docs/labs/k16-timing-2026-10-15/pin.example.json > \"\${OUT_DIR}/k16-timing-pin.json\"

  \"\${TIMING_BIN}\" check --pin \"\${OUT_DIR}/k16-timing-pin.json\" > \"\${OUT_DIR}/k16-timing-plan.json\"
  \"\${TIMING_BIN}\" run   --pin \"\${OUT_DIR}/k16-timing-pin.json\"
  test -s \"\${OUT_DIR}/k16-timing-record.json\"

  # 5. Seal output directory with SHA256SUMS
  (
    cd \"\${OUT_DIR}\"
    find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
    sha256sum -c SHA256SUMS
  )
"
```

*(If total wall time approaches 25 minutes, run `scripts/pod/uor-pod renew "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --hours 1` and `UOR_LAB="${UOR_LAB}" uor-harness heartbeat -m m1` before the 30-minute mark.)*

#### Step 4.2.4 — Laptop Retrieval, `SHA256SUMS` Verification, and `cloud-store put` MD5 Round-Trip (Before Pod Teardown)

```bash
RUN_NAME="20261015T000000Z-dr-qualification-${REVIEWED_SHA:0:9}"
LOCAL_STAGE="${HOME}/uor-r4-local/${RUN_NAME}"
mkdir -p "${HOME}/uor-r4-local"

# 1. Copy sealed result directory from pod to laptop gitignored staging
SSH_INFO="$(scripts/pod/uor-pod ssh "${POD_ID}")"
# Copy via scp using the host/port reported by uor-pod ssh:
scp -P <PORT> -i ~/.ssh/id_ed25519 -r \
  "root@<HOST>:/workspace/uor-r4/evals/${RUN_NAME}" \
  "${LOCAL_STAGE}"

# 2. Verify every file digest on laptop against pod-generated SHA256SUMS
(
  cd "${LOCAL_STAGE}"
  shasum -a 256 -c SHA256SUMS
)

# 3. Archive to shared iCloud store via cloud-store put (runs automatic MD5 download round-trip check)
~/.local/share/uor-r4/bin/cloud-store put "${UOR_LAB}" "${LOCAL_STAGE}"
# Expected output before proceeding:
#   stored 20261015T000000Z-dr-qualification-<sha> (<bytes> bytes, md5 <32-hex-md5>) at icloud:UOR-R4/results/<lab>/20261015T000000Z-dr-qualification-<sha>.tar
# Verify index TSV entry exists:
~/.local/share/uor-r4/bin/cloud-store list "${UOR_LAB}" | grep "${RUN_NAME}"
```

#### Step 4.2.5 — Pod Lease Release & Termination (Strictly After Step 4.2.4 Succeeds)

```bash
# 1. Release GPU lease in state.json
scripts/pod/uor-pod release "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}"

# 2. Terminate pod if no other lab holds an active lease on it
scripts/pod/uor-pod down "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}"

# 3. Confirm 0 leaked leases / pods
scripts/pod/uor-pod status
```

---

## 5. Explicit `NOT_TRAINING` Scope & Post-Run Zero-Update Verification Checklist

### 5.1 Static Source Guarantees (`crates/uor-r4-discrete`)

1. **`crates/uor-r4-discrete/src/lib.rs:177-179, 200-278` (`training_admission` & `apply_call_sites_remain_bounded_to_qualified_paths`):**
   - `training_admission()` (`lib.rs:177-179`) unconditionally returns `Err(Error::TrainingHeld)`.
   - In `src/main.rs`, `src/qualification.rs`, `src/flock_qualification.rs`, `src/relation_qualification.rs`, `src/measurement.rs`, `src/objective.rs`, `src/completion_baseline.rs`, and `src/redirect_admission.rs`: **zero** `.apply(` calls (`lib.rs:233-256`).
   - In `src/driver_qualification.rs`: bounded to $\le 11$ disposable 2-row synthetic fixture `.apply(` calls (`lib.rs:235-240, 266`), which never touch corpus files.
   - In `src/training.rs`: the single corpus `.apply(` call (`training.rs:880`) inside `PreparedDriver::transition` (`training.rs:859-897`) requires a `&CorpusUpdateAdmission` capability (`training.rs:658-661`), whose fields are private and whose public constructor `request_corpus_update_admission` returns `Err(Error::TrainingHeld)` (`training.rs:662-664`). A `CorpusUpdateAdmission` is issued only after `controller::ReleaseManifest` (`src/controller.rs:17-24`, for `pilot` / `pilot-resume` at `src/main.rs:43-47`) or `controller::completion::CompletionAdmission` + `redirect_admission` validation (`src/completion.rs:18-24`, `src/redirect_admission.rs:332-565`, for `completion` / `completion-resume` at `src/main.rs:38-42`) succeeds. None of `pilot`, `pilot-resume`, `completion`, or `completion-resume` is invoked by this runbook.
   - In `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs`: **zero** `.apply(` calls (`EXCLUSION` at `main.rs:52` and pre/post snapshot digest + epoch equality check at `main.rs:596-608`).
2. **`eval-corpus` hard-refuses any updated checkpoint (`crates/uor-r4-discrete/src/main.rs:87-89`):**
   ```rust
   if checkpoint.cursor.accepted_updates != 0 {
       return Err(Error::TrainingHeld);
   }
   ```

### 5.2 Mandatory 8-Point Post-Run `NOT_TRAINING` Audit Checklist

Before publishing the qualification receipt PR and `D-Packet`, the executor must verify all 8 invariants below across every file in `${LOCAL_STAGE}`:

| # | Audit Invariant | Exact Verification Command / Field Check | Required Result |
|---|---|---|---|
| 1 | **Step-0 CUDA build & clippy gate passed** | `cat "${LOCAL_STAGE}/cuda_build_exit.txt" "${LOCAL_STAGE}/cuda_clippy_exit.txt"` and non-empty `cuda-build.log`, `cuda-clippy.log` | Both exit codes `== 0` (`redirect_admission.rs:345-356`) |
| 2 | **Zero corpus updates accepted in `G-Pre` checkpoints** | `jq '.cursor.accepted_updates, .cursor.completed_batches, .model.epoch' "${LOCAL_STAGE}"/g-pre-*-init/report.json` | Every value `== 0` |
| 3 | **Zero updates applied by `G-Pre` evaluations** | `jq '.updates_applied_by_evaluation, .accepted_updates_in_model, .evaluation_mutates_model, .model_epoch' "${LOCAL_STAGE}"/g-pre-*-eval/report.json` | `0`, `0`, `false`, `0` on all 6 `G-Pre` evaluations (`redirect_admission.rs:422-425`) |
| 4 | **`G-Pre` ceiling bar met on all `K=16` runs** | `jq '.aggregate.target_admitted_rows' "${LOCAL_STAGE}"/g-pre-k16-*-eval/report.json` | Every value `>= 4894` (`redirect_admission.rs:447-451`) |
| 5 | **Pre- vs post-benchmark snapshot digest & epoch identity** | `jq -r '.status, (.records[] \| select(.record_kind=="cell") \| "\(.snapshot_digest_before == .snapshot_digest_after) \(.epoch_before) \(.epoch_after) \(.applied)")' "${LOCAL_STAGE}/k16-timing-record.json"` | `"complete"` and `"true 0 0 false"` on all 4 timing cells |
| 6 | **Zero training release / admission manifests executed** | `find "${LOCAL_STAGE}" -name '*release-plan*.json' -o -name '*owner-admission*.json'` | Empty (`0` files; `pilot` and `completion` are never invoked) |
| 7 | **Hold status preserved in `qualify-driver` report** | `jq -r '.training, .corpus_updates_executed' "${LOCAL_STAGE}/qualify-driver/report.json"` | `"HELD"` and `0` |
| 8 | **MD5 archive round-trip verified before pod release** | `~/.local/share/uor-r4/bin/cloud-store list "${UOR_LAB}" \| grep "${RUN_NAME}"` | Present in `icloud:UOR-R4/index/${RUN_NAME}.tsv` with matching byte size and MD5 digest |

---

## 6. Verification Executed for This Runbook Record (Host-Only, Zero Compute)

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target cargo test --locked --offline -p uor-r4-discrete --no-default-features`: **40 passed (`35` lib + `5` `prime-zeta-qualify`), 0 failed, 1 ignored** (confirms `training::driver_source_sha256() == ed4655a22dbec24316cfe5df16a43407860651ce6aa7a690ee8fcd5bedaf05d8`, `discrete.cu` raw file SHA-256 `f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`, `objective.rs` `C-R1..C-R6` host contract scaffolds, `redirect_admission.rs` K16 qualification/G-Pre/step-0 metadata validators, and `lib.rs` `.apply(` call-site census).
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target cargo test --locked --offline --manifest-path docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`: **15 passed, 0 failed**.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness check-formats`, `uor-harness shard-check`, `python3 scripts/check_claim_wording.py`: all pass.
