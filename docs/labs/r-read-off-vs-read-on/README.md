# `r-read-off-vs-read-on` — Route & Flock Read-Off vs Read-On Comparison (`antigravity-45`..`antigravity-46`)

- **Roadmap item:** [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../roadmap/items/r-read-off-vs-read-on.md) ([`#2885`](https://github.com/UOR-Foundation/uor-r4/issues/2885))
- **Pre-registration sheet:** [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md) (`DRAFT`, `TEST FITNESS: PENDING`)
- **Host finite controls:** [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs) (`7 / 7` unit tests pass on the host)
- **Kernel & driver switch (`antigravity-46`):** [`crates/uor-r4-discrete/src/lib.rs`](../../../crates/uor-r4-discrete/src/lib.rs), [`discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu), [`device.rs`](../../../crates/uor-r4-discrete/src/device.rs), [`training.rs`](../../../crates/uor-r4-discrete/src/training.rs), [`attention_preflight.rs`](../../../crates/uor-r4-discrete/src/attention_preflight.rs), and [`d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs)
- **Staged battery & configs (`antigravity-46`):** [`docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`](evidence/read-off-battery.sh) and the 10 zero-update arm configs (`{a8,p8}-{r-on,r-off,r-perm,f-on,f-off}.json`)
- **Status:** Steps 1–4 of [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../roadmap/items/r-read-off-vs-read-on.md) are complete on the host. No GPU compute has been run (`needs_gpu: true` Step 5 execution awaits non-author `TEST FITNESS: FIT`, PM/owner registration on `#2885`, and Casey's approval in chat per CHARTER §4.5 / §5.1).

---

## 1. Retained-Checkpoint Inventory & `eval-corpus` Refusal Audit (Step 1)

Per Step 1 of [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../roadmap/items/r-read-off-vs-read-on.md), we audited [`crates/uor-r4-discrete/src/main.rs:102-114`](../../../crates/uor-r4-discrete/src/main.rs#L102-L114) and all retained checkpoint records linked from [`docs/labs/attention-state-2026-10-18/README.md`](../attention-state-2026-10-18/README.md), [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../k16-outcome-call-2026-10-19/README.md), [`docs/labs/address-cells-preflight-2026-10-19/README.md`](../address-cells-preflight-2026-10-19/README.md), [`docs/labs/a8-dose-2026-10-19/README.md`](../a8-dose-2026-10-19/README.md), and [`docs/labs/m1-a8-erosion-diagnosis/README.md`](../m1-a8-erosion-diagnosis/README.md).

### 1.1 `eval-corpus` Refusal Check (`crates/uor-r4-discrete/src/main.rs:102-114`)

In [`crates/uor-r4-discrete/src/main.rs:108-110`](../../../crates/uor-r4-discrete/src/main.rs#L108-L110), `eval-corpus` enforces:

```rust
if checkpoint.cursor.accepted_updates != 0 {
    return Err(Error::TrainingHeld);
}
```

Consequently, `eval-corpus` as currently written accepts **only** checkpoints with `checkpoint.cursor.accepted_updates == 0`, and refuses every post-update checkpoint with `Error::TrainingHeld`.

### 1.2 Inventory of Retained Checkpoints

| Checkpoint / Lineage | Store Location | `ReadSpec` | Writer $K$ | Codebook Table | `accepted_updates` | `eval-corpus` Today | Recorded DEV Metrics (`14,287` full / `3,847` `P_succ`) |
|---|---|---|---:|---|---:|---|---|
| **`A8` cp0** (`init-corpus`) | HF `caseyallard/uor-r4-store:vast-archive/vast-56660582/` & `vast-56670997/` | `Relation { kept: 8, sweeps: 1 }` (`ActiveLeafV1`, edge trits `0` = Flock-active at cp0) | `8` | `address_structured_v1` (`6600bdb1…`, table `4122cbab…`) | **`0`** | **Accepted** | Full admitted `4,244`, `P_succ` admitted `1,351`, full `top1 = 133`, `P_succ top1 = 62` (`routes = 71cdce57…`, `gates = ceb6ae99…`) |
| **`P8` cp0** (`init-corpus`) | HF `vast-archive/vast-56660582/` | `Relation { kept: 8, sweeps: 1 }` (`ActiveLeafV1`, `model_sha256 = 134fd663…`) | `8` | `PrimeZeta` (`83c1380a…`, table `e9e03bcd…`) | **`0`** | **Accepted** | Full admitted `427`, `P_succ` admitted `127`, full `top1 = 38` |
| **`S8` cp0** (`init-corpus`) | HF `vast-archive/vast-56660582/` | `Relation { kept: 8, sweeps: 1 }` (`ActiveLeafV1`) | `8` | `address_shuffled_v1_seed_20261019` (`a185bdca…`, table `d2fb8680…`) | **`0`** | **Accepted** | Full admitted `1,733`, `P_succ` admitted `476`, full `top1 = 99` |
| **`A16` / `P16` cp0** | HF `vast-archive/vast-56660582/` | `Relation { kept: 8, sweeps: 1 }` (`ActiveLeafV1`) | `16` | `A8` (`6600bdb1…`) / `PrimeZeta` (`83c1380a…`) | **`0`** | **Accepted** | `A16`: `6,023` full, `2,178` `P_succ`, `top1 = 133`; `P16`: `872` full, `246` `P_succ`, `top1 = 37` |
| **`Route` cp0 on `A8` / `P8`** | Reproducible via `init-corpus` from pinned `6600bdb1…` / `83c1380a…` | `Route` (`routes = 71cdce57…`) | `8` | `A8` (`6600bdb1…`) / `PrimeZeta` (`83c1380a…`) | **`0`** | **Accepted** | Shares the exact checkpoint-0 table and `routes` hash `71cdce57…` with `A8`/`P8` |
| **Rung-0 Q4** (`16,404` batches) | Archived (`archive/labs/rung0-admission-prep-2026-10-15/README.md`, `alias-census-REPORT.md`) | `Route` | `8` | `SeededDeviceV1` (16 init vectors $\to$ 264 at Q4) | **`391`** (incl. `16/20` route moves) | **Refused (`TrainingHeld`)** | Full admitted `1,920`, `top1 = 149`, `top8 = 747` |
| **K16-A s1 / s2** (`16,404` batches) | Sealed custody `codex-cf1-k16-four-role-data1-training-root-part-000-2324z` (`1ca211b4…`) | `Route` | `16` | `SeededDeviceV1` (16 $\to$ 279 / 274 vectors) | **`407` / `410`** | **Refused (`TrainingHeld`)** | `A s1`: admitted `2,237`, `top1 = 65`, `top8 = 547`; `A s2`: admitted `3,167`, `top1 = 463`, `top8 = 1,295` |
| **K16-R s1 / s2** (`16,404` batches) | Sealed custody `codex-cf1-k16-four-role-data1-training-root-part-000-2324z` (`1ca211b4…`) | `Route` | `16` | `SeededDeviceV1` (collapsed to 20 / 19 cells) | **`130` / `129`** | **Refused (`TrainingHeld`)** | `R s1`: admitted `12,089`, `top1 = 101`; `R s2`: admitted `12,466`, `top1 = 230` (`G-Occ` collapse) |
| **A8 Dose (`vast-56670997`)** | HF `vast-archive/vast-56670997/` (40 checkpoints across 64-batch intervals) | `Relation { kept: 8, sweeps: 1 }` (`LearnV1` vs `FrozenZeroV1`) | `8` | `A8` (`6600bdb1…`) | **`396`** (`Relation` Q1 at batch `4,096`) / **`809`** (`FrozenZero` Q4 at batch `16,404`) | **Refused (`TrainingHeld`)** | `Relation` Q1: `P_succ` admitted `551`, full `1,511`; `FrozenZero` Q1: `P_succ` admitted `857`, full `2,519` |

### 1.3 Open PM / Owner Question on `#2885` (No Guessing)

Per Step 1 (*"If every retained checkpoint has accepted updates, say so on the issue and ask the PM whether a zero-update checkpoint or a new driver path is the intended input. Do not guess"*):

1. **Zero-update checkpoints (`accepted_updates == 0`) exist and are scoreable by `eval-corpus` today:** The retained `A8` (`6600bdb1…`) and `P8` (`83c1380a…`) checkpoint-0 models have `accepted_updates == 0`, full 256-cell / 4,096-vector injectivity (`0` tie mass), and identical initial `routes` (`71cdce57…`). On `A8` cp0, `Relation { kept: 8, sweeps: 1 }` with `ActiveLeafV1` and all relation edge trits at `0` is bit-identical to `Flock { kept: 8 }` read-on (`1,351 / 3,847` `P_succ` admitted, `top1 = 133`, `P_succ top1 = 62`), and switching `read_initialization` to `ZeroGatesV1` gives the exact `Flock` read-off arm on the same checkpoint.
2. **Every retained *trained* checkpoint (`Rung-0 Q4`, `K16-A/R s1/s2`, `A8 Relation Q1` / `FrozenZero Q4`) has `accepted_updates > 0` and is refused by `eval-corpus` (`main.rs:108-110`):** To compare read-on vs read-off on the trained checkpoints where `Route` scored `149` top-1 (`Rung-0 Q4`) or `463` top-1 (`K16-A s2`) and `FrozenZero` took `508` combine-gate updates (`vast-56670997`), we ask the PM/owner on `#2885` to choose between:
   - **Option A (Zero-update checkpoint-0 paired battery only, `eval-corpus` unchanged):** Run the paired `Read-On` vs `Read-Off` vs `Read-Permuted` comparison on `A8` cp0 and `P8` cp0 (`accepted_updates == 0`) for both `Route` and `Flock`.
   - **Option B (Both checkpoint-0 and retained post-update checkpoints via a read-only evaluation command):** Keep `eval-corpus`'s `accepted_updates == 0` guard intact, and authorize a read-only checkpoint evaluation entry point (which verifies `evaluation_mutates_model == false`, performs zero proposals/updates, and scores stored checkpoints from `vast-56670997` and `K16-A s1/s2`) so both zero-update and post-update retained checkpoints are compared on the same `14,287` DEV rows.

---

## 2. Design of the `Route` and `Flock` Read-Off Arms (Step 2)

### 2.1 `(a)` Exact Read-Off Arm for Each Read

1. **`Route` read-off (`RouteBypassV1`, device spec word `s[13] == 1` in `MODE_ROUTE`):**
   - In `route_vector` ([`crates/uor-r4-discrete/src/discrete.cu:41-77`](../../../crates/uor-r4-discrete/src/discrete.cu#L41-L77)), each block $b \in \{0, \dots, s[1]-1\}$ currently loads the query block at the last valid position `pos = length - 1`:
     ```c
     int query[8];
     input_block(input, ids, g, row, length - 1, block, s, kind, index, candidate, query);
     ```
     and then walks `s[5]` stages of the ternary `BinderPlan`, applying `routes[block * s[5] + stage]` and composing `BFS[root -> goal]` until a leaf `atom` is reached, writing `action(g, total, carrier, output + block * 8)`.
   - Under `RouteBypassV1` (`s[13] == 1`), after the row-plan header bounds check (`length == 0 || length > s[2] || nodes > s[6] || frontier == 0 || frontier > 16`), `route_vector` skips the ternary-tree port walk and `BFS` transport loop and writes `query` directly to `output + block * 8`:
     ```c
     if (s[13] == 1u) {
         for (U block = 0; block < s[1]; ++block) {
             input_block(input, ids, g, row, length - 1, block, s, kind, index, candidate, output + block * 8);
         }
         return true;
     }
     ```
   - This returns no routed context vector: the writer receives the unrouted last-position query vector $q = x_{\text{length}-1} \in (\mathbb{Z}^8)^B$, isolating what the `Route` tree walk and `BFS` transport add above the last token's own codebook carrier.

2. **`Flock` read-off (`ReadInitialization::ZeroGatesV1`, all gates $g_{b,\ell} = 0$):**
   - In `flock_vector` ([`crates/uor-r4-discrete/src/discrete.cu:127-270`](../../../crates/uor-r4-discrete/src/discrete.cu#L127-L270)), for each block $b \in \{0, \dots, s[1]-1\}$, the accumulator `u[0..8]` is initialized to the last-position query block `input_block(..., pos = length - 1, b, ..., u)` (`discrete.cu:147`) and `terms` is initialized to `1` (`discrete.cu:223`).
   - At each tree level $\ell \in \{0, \dots, s[5]-1\}$, if `gate == 0`, lines `229` and `260` skip adding the level consensus root $m_\ell$ to `u` and leave `terms` unchanged.
   - When **all** level gates are `0` (`ReadInitialization::ZeroGatesV1`, and in `Relation` mode all edge gates are also `0`), the level loop finishes with `u` equal to the initial query block `query[b]` and `terms == 1`.
   - Line `265` then computes `U out = snap(u, 1u, g)` (under default `s[12] == 0`, since `terms == 1`) and writes `g[ROOT + out * 8 + d]` to `output[b * 8 + d]`.
   - **Exact identity proof:** Because every valid codebook block is one of the 16 orbit roots $r_0 \in \{0, \dots, 15\}$, for $u = r_0$ the candidate $r = r_0$ in `snap(u, 1, g)` achieves $L_1(r_0 - 1 \cdot r_0) = 0$ and $\operatorname{Ham}(r_0, r_0) = 0$, whereas every other orbit root $r \ne r_0$ has $L_1(r_0 - r) > 0$. Therefore `snap(u, 1, g) == root_id(u)` holds for all 16 orbit roots, and `output[b]` is **bit-for-bit identical** to the last-position query block `query[b]`.
   - **Cross-read equivalence corollary:** On any valid codebook table, `Route` with `RouteBypassV1` (`s[13] == 1`) and `Flock` (or `Relation`) with `ZeroGatesV1` (`gates == 0`) produce **bit-for-bit identical** read outputs `output[0..B*8]` and **bit-for-bit identical** writer predictions, admitted cells, rival counts, and hinge losses on every row. `Flock` read-off therefore requires **zero new CUDA kernel arithmetic**.

### 2.2 `(b)` Why Neither Arm Uses Softmax, Squashing, Matmul, or Floats

- **No softmax or squashing (CHARTER §3.2, D23):** `Route` selects one port per tree stage by hard lexicographic integer minimum `(distance, hamming, port)` (`discrete.cu:56-62`). `Flock` selects up to $K$ neighbours per tree level by hard integer insertion sort `(rel_score desc, distance asc, hamming asc, recency desc, node asc)` (`discrete.cu:239-250`) and snaps their unweighted integer sum to the nearest orbit root via `snap` (`discrete.cu:104-112`). `RouteBypassV1` and `ZeroGatesV1` pass the integer query block directly without any weighting or normalization function.
- **No dense matrix multiplication or GEMM (CHARTER §3.3, D25, D30):** All 64 group actions in $G$ (`action`, `discrete.cu:20-22`) permute and sign-flip 8 integer coordinates via the 512-entry `AXIS` lookup table, and transport composes actions via the 4,096-entry `COMPOSE` and 256-entry `BFS` lookup tables. The writer (`discrete.cu:277-310`) computes integer $L_1$ distances over $\mathbb{Z}^8$ blocks.
- **Integer-only (CHARTER §3.1, D32):** Every value in `route_vector`, `flock_vector`, `RouteBypassV1`, `ZeroGatesV1`, and `writer` is `int` (`i32`), `U` (`u32`), or `W` (`u64`). Zero `f32` or `f64` types or instructions appear anywhere.

### 2.3 `(c)` Six Finite Controls (`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`)

In [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs), we implement an exact integer host reference of `route_vector`, `flock_vector`, `snap`, and `writer` against the compiled `OUT_DIR/geometry.rs` tables (`ROOTS`, `AXES`, `COMPOSE`, `BFS`, `BRANCH`, `GENERATORS`, `IDENTITY`) and verify 6 finite controls (mirroring the 11 controls in `qualification.rs` and 6 controls in `flock_qualification.rs`):

| Control | Test Function | What It Verifies |
|---|---|---|
| **C1** | `route_read_on_preserved_bit_for_bit` | With `route_bypass == 0` (`s[13] == 0`), `route_vector` output and writer `(prediction, admitted_cells)` match the unmodified baseline `route_vector` bit-for-bit across multi-stage ternary plans and all 64 group actions in $G$. |
| **C2** | `route_read_off_equals_query_last_position_bypass` | With `route_bypass == 1` (`s[13] == 1`), `route_vector` returns `input_block(..., pos = length - 1, ...)` for every block regardless of `routes` or earlier context tokens, while still honoring virtual `VectorBlock` moves (`kind == 1`) on the last position's token. |
| **C3** | `flock_zero_gates_is_exact_read_off_and_matches_route_bypass` | Exhaustively verifies `snap(root(r), 1) == r` for all 16 orbit roots $r \in \{0, \dots, 15\}$, and proves that `flock_vector` with all gates `0` (`ZeroGatesV1`, in both `MODE_FLOCK` and `MODE_RELATION`) emits `terms == 1` and bit-for-bit identical `output` vectors and writer `(prediction, admitted_cells)` as `Route` with `RouteBypassV1`. |
| **C4** | `flock_active_leaf_differs_from_zero_gates_when_context_shifts_consensus` | Verifies on a multi-token causal prefix that `Flock` read-on (`ActiveLeafV1`, $g_0 = +1$) transports and snaps earlier context leaves, shifting both the read output root and the writer's top-1 prediction relative to `ZeroGatesV1` read-off. |
| **C5** | `mode_and_proposal_refusal_guards` | Verifies that enabling `route_bypass` (`s[13] != 0`) when `s[8] != MODE_ROUTE` or setting `s[13] > 1` is refused (`status = 11`), and proposing a `Route` move (`kind == 0`) while `s[13] == 1` is refused (`status = 11`) so counterfactual scoring cannot silently evaluate no-op route proposals. |
| **C6** | `geometric_permuted_read_control_breaks_route_alignment_without_leaving_group` | Constructs a deterministic geometric shuffled-read control (`RoutePermutedActionsV1`) by applying an odd-stride affine derangement $a \mapsto (5a + 1) \bmod 64$ inside the 64-element signed-axis group $G$, verifying that every action remains a valid group element in `0..64` with zero fixed points while altering routed leaf selection and writer predictions relative to `Route` read-on. |

### 2.4 `(d)` Instrument-vs-Mechanism Classification (CHARTER §4.2, D31)

Before any result from the paired GPU evaluation is interpreted as a finding about `Route` or `Flock`, the run is classified under D31:

1. **Instrument faults (halt and fix the harness; never counted as a mechanism negative):**
   - **Cross-mode read-off discrepancy:** On any shared checkpoint table (`A8` cp0 or `P8` cp0), if `Route` read-off (`RouteBypassV1`) and `Flock` read-off (`ZeroGatesV1`) disagree on **even 1 of the 14,287 DEV rows** (`top1_correct`, `top8_correct`, `target_admitted_rows`, `admitted_cell_count_sum`, `token_rival_count_sum`, or `address_rival_count_sum`), the run stops immediately as an instrument fault. By mathematical construction (Section 2.1), both read-off arms feed the exact same last-position query vector $x_{\text{length}-1}$ into the same writer.
   - **Read-on reproduction discrepancy:** If `Flock` read-on (`ActiveLeafV1` on `A8` cp0) fails to reproduce `target_admitted_rows = 4,244 / 14,287`, `P_succ = 1,351 / 3,847`, `top1_correct = 133`, and `P_succ top1 = 62` (`P8` cp0: `427 / 127 / 38`), or if any retained post-update checkpoint fails to reproduce its sealed evaluation receipt (`Rung-0 Q4`: `1,920 / 149 / 747`; `K16-A s1/s2`: `2,237 / 65 / 547` and `3,167 / 463 / 1,295`), the run stops as an instrument fault.
   - **Pin or determinism failure:** Any mismatch in `DEV` corpus SHA-256 (`0919e90f…` / `a838bb1a…`), codebook table SHA-256, or bit-for-bit repeatability across two consecutive `eval-corpus` runs.

    - **Mechanism classification (when all instrument gates pass):**
       - Paired row-by-row contrasts on the `14,287` DEV rows and the `3,847` `P_succ` copy-successor subset between `Read-On` and `Read-Off` (and between `Read-On` and `Read-Permuted` `RoutePermutedActionsV1`), judged against the paired McNemar discordance floor $F = \max(24, 2\sqrt{b + c})$:
         - **Positive (`READ_ADDS_SIGNAL`):** `Read-On` exceeds both `Read-Off` and `Read-Permuted` on `top1_correct` (or `target_admitted_rows` at fixed occupancy) by more than $F = \max(24, 2\sqrt{b + c})$ without triggering `G-Occ`.
         - **Null / Inactive (`READ_INERT_AT_CHECKPOINT`):** $|b - c| \le F$ between `Read-On` and `Read-Off` — the read adds no statistically distinguishable signal above the unrouted last-position token carrier at that checkpoint (closing that checkpoint/schedule configuration under D22, and routing effort to admission/codebook/writer or an interleaved read schedule).
         - **Destructive interference (`READ_HURTS_ADMISSION_OR_RANKING`):** `Read-Off` exceeds `Read-On` by more than $F$ (`c - b > F`), showing that uncalibrated tree transport / neighbour consensus moves queries away from their predecessor-placed target cells at that checkpoint.

---

## 3. Switch Implementation & Staged Battery Script (Step 4, `antigravity-46`)

Per Step 4 of [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../roadmap/items/r-read-off-vs-read-on.md):

1. **`ReadInitialization` variants & kernel switch (`s[13]`):**
   - [`crates/uor-r4-discrete/src/lib.rs`](../../../crates/uor-r4-discrete/src/lib.rs) adds `ReadInitialization::RouteBypassV1` (`route_bypass_word() == 1`, `ReadSpec::Route` only) and `ReadInitialization::RoutePermutedActionsV1` (`permuted_route_action(a) = (5 * a + 1) % 64`, `ReadSpec::Route` only). `is_zero_gates()` remains true only for `ZeroGatesV1`, keeping all existing `Snapshot` and `DriverConfig` serializations and training-release guards (`controller.rs:162`) unchanged.
   - [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`route_vector` and `discrete_validate`) validates `s[13] <= 1` and `(s[13] == 0 || s[8] == 0)`, refuses `Parameter::Route` (`kind == 0`) proposals when `s[13] == 1`, and when `s[13] == 1` writes `input_block(..., length - 1, block, ...)` directly into `output + block * 8`.
   - [`crates/uor-r4-discrete/src/device.rs`](../../../crates/uor-r4-discrete/src/device.rs) passes `read_initialization.route_bypass_word()` as the 14th device spec word `s[13]`, applies and readback-verifies `permuted_route_action` on `routes` when `read_initialization == RoutePermutedActionsV1`, refuses `Parameter::Route` in `Device::parameter` under `RouteBypassV1`, and adds the `#[ignore = "requires a CUDA GPU"]` device test `route_read_off_and_permuted_controls_run_and_refuse`.
   - [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) refuses `RouteBypassV1` and `RoutePermutedActionsV1` so research control arms are never exported as `UORLUT01` artifacts.
   - [`crates/uor-r4-discrete/src/attention_preflight.rs`](../../../crates/uor-r4-discrete/src/attention_preflight.rs) (`validate_dev_b3_config`) admits the paired `Route` (`ZeroGatesV1`, `RouteBypassV1`, `RoutePermutedActionsV1`) and `Flock`/`Relation` (`ActiveLeafV1`, `ZeroGatesV1`) arms in `attention-dev-b3` while keeping `capture_dev` strictly gated by `attention_learning::validate_config`.
2. **Staged 10-arm zero-update configs & battery script (`docs/labs/r-read-off-vs-read-on/evidence/`):**
   - Ten `DriverConfig` files (`a8-r-on.json`, `a8-r-off.json`, `a8-r-perm.json`, `a8-f-on.json`, `a8-f-off.json`, `p8-r-on.json`, `p8-r-off.json`, `p8-r-perm.json`, `p8-f-on.json`, `p8-f-off.json`) are committed and verified by control `C7` (`c7_committed_ten_arm_battery_configs_are_valid_and_distinct` in [`read_off_ablation_controls.rs`](../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs)).
   - [`docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`](evidence/read-off-battery.sh) runs `G-Qual` (`route_read_off_and_permuted_controls_run_and_refuse`), `init-corpus`, `eval-corpus`, and `attention-dev-b3` across all 10 arms, checks `G-Repro` (`4,244 / 1,351 / 133` on `a8-f-on` and `427 / 127 / 38` on `p8-f-on`) and `G-CrossMode` (`r-off == f-off` across all `14,287` rows on both `a8` and `p8`), and writes `FILES.sha256`.
