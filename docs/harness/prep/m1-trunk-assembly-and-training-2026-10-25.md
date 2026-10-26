# Pre-Registration Sheet: M1 Stage 3 Trunk Assembly and Training (`m1-trunk-assembly-and-training`, `#2844`)

- **Date**: 2026-10-25
- **Lab**: `antigravity` (`antigravity-89`, PR `#3258`)
- **Milestone**: M2 (`#2030`), Effective Milestone M1 (`#2029`) because [`m1-address-as-read`](../../roadmap/items/m1-address-as-read.md) (`#2833`) depends on `#2844`
- **Roadmap Item**: [`docs/roadmap/items/m1-trunk-assembly-and-training.md`](../../roadmap/items/m1-trunk-assembly-and-training.md) (`#2844`)
- **Order**: [`docs/loop/tasks/m1-trunk-assembly-and-training/antigravity-89/ORDER.md`](../../loop/tasks/m1-trunk-assembly-and-training/antigravity-89/ORDER.md)
- **Prerequisite Results**:
  - [`m1-bvg-k-stage-t-training`](../../labs/m1-bvg-k-stage-t-training/README.md) (`#2836`, PR `#3239`)
  - [`m1-normalization-mechanism`](../../labs/m1-normalization-mechanism/README.md) (`#2842`, PR `#3231`)
  - [`m1-position-mechanism`](../../labs/m1-position-mechanism/README.md) (`#2843`, PR `#3256`)

---

## 1. Question & Whole-Project Chain

Stage 2 (`m1-bvg-k-stage-t-training`, `#2836`) established that `BVG-K` (`K = 16`) block learning on the `A8-Outer` (`V = 4,096`, `B = 4`, `C = 192`) discrete model expands DEV target cell admission to `5,009 / 14,287` (`35.06%`, `P_succ = 1,216 / 3,847`) on `T16_fz` and lifts learned relation moves (`425` moves) and DEV `top-1` `2.79x` (`133 -> 371`) on `T16_rel`. Crucially, `#2836` diagnosed that inside those admitted cells $A_W(x)$, the read code $c_{\text{read}}(x)$ and query stream $q_{\text{tok}}(x)$ already carry enough information for `2,140 / 14,287` (`T_restr(A_W)`) and `3,084 / 14,287` (`G_restr(A_W)`) exact hits, whereas the raw `OuterL1V1` writer applied directly to the unrotated read vector $q^{\text{read}}(x)$ reaches only `455 / 14,287` (`T16_fz`) and `371 / 14,287` (`T16_rel`) because there is no non-linear geometric feed-forward gate or causal recurrence between the read stage and the single-vector `OuterL1V1` writer.

Now that both Stage 3 prerequisites are `done` (`#2842` `OrbitRootSnapNormV1` and `#2843` `PositionHolonomyV1`), this sheet pre-registers the full Stage 3 discrete trunk (`TrunkAssemblyV1` in `crates/uor-r4-discrete/src/trunk.rs`) to answer:

> **Primary Question**: When the causal read (`Relation { kept: 8, sweeps: 1 }` and `FrozenZeroV1`), prefix-holonomy position mechanism (`PositionHolonomyV1`), orbit-root snap normalization (`OrbitRootSnapNormV1`), causal quaternion/Clifford holonomy recurrence (`DiscreteQuaternionRecurrenceV1`), and non-linear geometric feed-forward gate (`DiscreteHamiltonGateV1`) are assembled into a single-vector trunk whose sole output per row is one 32-int vector $q^{\text{trunk}}_L(x) \in \mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$ scored by the unmodified `OuterL1V1` writer, does the assembled trunk beat Stage 2's best by more than the seed-to-seed noise and the exact integer $2\text{-SE}$ McNemar floor across both seeds (`Seed 1`, `Seed 2`) on both `ft/dev` (`14,287` rows) and the sealed fresh evaluation panel `fresh_v2` (`14,287` rows)?

---

## 2. Exact Trunk Architecture (`TrunkAssemblyV1`)

Every operator in `TrunkAssemblyV1` is pure integer geometry on the 64-element signed-permutation / quaternion-Clifford group $\mathrm{Cl}(0,6)$ and the 16-root $E_8$ orbit $\mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$ (`CHARTER.md` §3.1–§3.5, D23, D24, D25, D32):

1. **Stage 1 — Causal Read + Position + Normalization (GPU)**:
   - On CUDA (`Device::forward` + `Device::evaluate`), each context window $x = (w_0, \dots, w_{L-1})$ ($1 \le L \le 192$) produces its 32-int read vector $q^{\text{read}}(x) \in \mathcal{O}_{16}^4$ (packed 16-bit orbit code $c_{\text{read}}(x) = (r_0, r_1, r_2, r_3) \in \{0..15\}^4$) and its 8-cell outer-L1 admitted shortlist $A_W(x) \subset \{0..255\}$, defining the admitted candidate token set $V_{A_W(x)} = \{y \in \{0..4095\} : \mathrm{cell}(y) \in A_W(x)\}$.
2. **Stage 2 — Causal Quaternion/Clifford Prefix-Holonomy Recurrence (`DiscreteQuaternionRecurrenceV1`)**:
   - Using the 64-entry whole-number step table (`PositionStepTable`, `blocks = 4`, candidates in `0..16`), each block $b \in \{0, 1, 2, 3\}$ accumulates causal prefix holonomy $F_{p,b} = u_{b, r_{p,b}} \circ F_{p-1,b} \in \{0..63\}$ across $p \in [0, L)$, forms the exact multiscale relative holonomy transport $H_{\text{prev},b} = F_{L-1,b} \circ F_{\max(0, L-2),b}^{-1}$, $H_{\text{mid},b} = F_{L-1,b} \circ F_{\lfloor(L-1)/2\rfloor,b}^{-1}$, and full-window $F_{L-1,b}$, rotates the predecessor, mid-window, and window-start roots via `apply_frame_to_root`, sums them with the query root $\rho(r_{L-1,b})$, and projects back onto $\mathcal{O}_{16}$ via scale-matched `OrbitRootSnapNormV1` (`multiplicity = 4`, `NormFrameTable`), yielding the normalized 32-int recurrence vector $h_{\text{rec}}(x) \in \mathcal{O}_{16}^4$ and its 16-bit orbit code $c_{\text{rec}}(x) \in \{0..65535\}$.
3. **Stage 3 — Non-Linear Geometric Feed-Forward Gate (`DiscreteHamiltonGateV1`) + Unmodified Single-Vector `OuterL1V1` Writer**:
   - Because $\mathrm{Cl}(0,6)$ acts transitively on $\mathcal{O}_{16}$, the build-time table `geometry::BFS[r * 16 + t]` maps any read block root $r_b \in \{0..15\}$ to any target block root $t_b \in \{0..15\}$ by an exact signed-axis rotation $g_b \in \{0..63\}$.
   - Fitted by exact integer sufficient-statistic counting over the `1,049,856` model-matched TRAIN rows (`16,404` GPU batches $\times$ `64` rows, strictly disjoint from `fresh_v2`), `DiscreteHamiltonGateV1` selects the target roots $(t_0, t_1, t_2, t_3)$ of $y^*_L(x)$, rotates $q^{\text{read}}(x)$ by $g_L(x) = (\mathrm{BFS}[r_b \cdot 16 + t_b])_{b=0}^3 \in \{0..63\}^4$ (`transform_block_by_frame`), and projects via `OrbitRootSnapNormV1` (`multiplicity = 1`), outputting **one single 32-int trunk vector** $q^{\text{trunk}}_L(x) \in \mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$.
   - That single 32-int vector $q^{\text{trunk}}_L(x)$ is then scored by the **unmodified `OuterL1V1` writer** (`writer_score_admitted_fast` over $V_{A_W(x)}$ and `writer_score_full_outer_l1` recomputing both cell admission and token ranking from $q^{\text{trunk}}_L(x)$ alone):
     $$\hat{y}_L(x) = \arg\min_{y \in V_{A_W(x)}} \left( \|q^{\text{trunk}}_L(x) - v_y\|_1,\ y \right).$$

---

## 3. Base Artifact, Data Split, Registered Arms, Readout Levels, and Component Ablations

### 3.1 Four Primary Arms Across Two Seeds (`Seed 1`, `Seed 2`)

| Arm ID | Base Stage T Checkpoint (`#2836`) | Policy & Block Size | Seed |Recurrence / Norm Offset |
|---|---|---|---:|---:|
| **`Trunk-FZ-S1`** | `t16-frozen-zero-seed1/final-checkpoint.json` (`epoch = 420`, `854` moves) | `FrozenZeroV1`, `K = 16` | `1` | `seed_offset = 0` |
| **`Trunk-FZ-S2`** | `t16-frozen-zero-seed2/final-checkpoint.json` (`epoch = 420`, `854` moves) | `FrozenZeroV1`, `K = 16` | `2` | `seed_offset = 1` |
| **`Trunk-Rel-S1`** | `t16-relation-seed1/final-checkpoint.json` (`epoch = 678`, `1,505` moves) | `LearnV1`, `K = 16` | `1` | `seed_offset = 0` |
| **`Trunk-Rel-S2`** | `t16-relation-seed2/final-checkpoint.json` (`epoch = 678`, `1,505` moves) | `LearnV1`, `K = 16` | `2` | `seed_offset = 1` |

### 3.2 Five Transparent Single-Vector Readout Levels (Evaluated on Both `ft/dev` and `fresh_v2`)

1. **`Level 0` (`writer_raw` / `C_gate_id`)**: Identity Hamilton gate $g_0(x) = (\mathrm{IDENTITY})^4$, $q^{\text{trunk}}_0(x) = q^{\text{read}}(x)$ (`455 / 14,287` on `T16_fz`, `371 / 14,287` on `T16_rel` on `ft/dev`).
2. **`Level 1_global` (`hamilton_global_single_vec`)**: $y^*_{1,\text{global}}(c_{\text{read}})$ selected globally from $C_{\text{read}}(c_{\text{read}}, y)$ without knowing $A_W(x)$, then single vector $q^{\text{trunk}}_{1,\text{global}}(x)$ scored over $V_{A_W(x)}$ by `OuterL1V1`.
3. **`Level 1` (`hamilton_read_only`)**: $y^*_1(x) \in V_{A_W(x)}$ selected from $C_{\text{read}}(c_{\text{read}}(x), y)$, rotated into single vector $q^{\text{trunk}}_1(x)$ and scored by `OuterL1V1`.
4. **`Level 2` (`hamilton_read_recurrence`)**: $y^*_2(x) \in V_{A_W(x)}$ selected from $S_2(x, y) = 16 C_{\text{read}}(c_{\text{read}}, y) + 8 C_{\text{read,rec}}((c_{\text{read}}, c_{\text{rec}}), y) + C_{\text{rec}}(c_{\text{rec}}, y)$, rotated into single vector $q^{\text{trunk}}_2(x)$ and scored by `OuterL1V1`.
5. **`Level 3` (`hamilton_full_trunk`)**: $y^*_3(x) \in V_{A_W(x)}$ selected from $S_3(x, y) = 64 C_{\text{query}}(q_{\text{tok}}, y) + 32 C_{\text{query,read}}((q_{\text{tok}}, c_{\text{read}}), y) + 16 C_{\text{query,rec}}((q_{\text{tok}}, c_{\text{rec}}), y) + 4 C_{\text{read}}(c_{\text{read}}, y) + C_{\text{rec}}(c_{\text{rec}}, y)$, rotated into single vector $q^{\text{trunk}}_3(x)$ and scored by `OuterL1V1`.

### 3.3 Four Component Ablations

- **`C_rec_null`** (`RecurrenceMode::IdentityNullControlV1`): Sets $c_{\text{rec}}(x) = 0$ on every row, testing whether the causal quaternion recurrence adds genuine predictive signal in `Level 2` and `Level 3`.
- **`C_rec_shuf`** (`RecurrenceMode::ShuffledOrderControlV1`): Shuffles prefix context positions $0..L-1$ before prefix holonomy, testing whether causal token order inside `DiscreteQuaternionRecurrenceV1` matters.
- **`C_norm_unit`** (`NormalizationMode::UnitSnapControlV1`): Replaces multiplicity-matched `snap_3` normalization in `DiscreteQuaternionRecurrenceV1` with unscaled `snap_1`.
- **`C_gate_id`** (`Level 0`): Replaces `DiscreteHamiltonGateV1` with identity frames $(\mathrm{IDENTITY})^4$.

---

## 4. Step 0 Instrument Checks (`trunk-run step0`)

Before any Stage 3 claim is graded, all five Step 0 checks must pass (`all_controls_pass = true`):

1. **`G-T1` (Transitive Hamilton Gate Closure)**: All $16 \times 16 = 256$ `(from_root, to_root)` pairs in `transitive_bfs_frame` map `from_root` to `to_root` on $\mathcal{O}_{16}$ with `0` mismatches.
2. **`G-T2` (Recurrence Holonomy & Orbit Norm Exactness)**: Across `256` DEV windows, `compute_recurrence_state` matches `position_mechanism::relative_holonomy` bitwise (`0` mismatches) and every trunk output block satisfies `BlockNormInvariants::is_canonical_unit_root()` (`L1 = 4`, `L2^2 = 8`, `L_infty = 2`, `support = 2`).
3. **`G-T3` (Cross-Build Gate R Exactness on DEV)**: Loading `T16_fz` Seed 1 (`455` top-1, `1,484` top-8, `5,009` admitted, `1,216` `P_succ` admitted) and `T16_rel` Seed 1 (`371` top-1, `1,124` top-8, `3,827` admitted, `1,144` `P_succ` admitted) reproduces the sealed `#2836` DEV evaluation bitwise (`0` mismatches).
4. **`G-T4` (Single-Vector `OuterL1V1` Writer Equivalence)**: On all `14,287` DEV rows and `14,287` `fresh_v2` rows, `writer_score_admitted_fast(q_read)` and `writer_score_full_outer_l1(q_read)` match GPU `discrete_evaluate` `top8` on `14,287 / 14,287` rows (`0` mismatches).
5. **`G-T5` (Fresh Panel `fresh_v2` Isolation & Export)**: `cut_fresh_slice(train, 192, 1_049_856, 14_287)` yields exactly `14,287` kept targets from documents starting strictly after TRAIN target `1,049,856` with `0` duplicate `(window, target)` rows against the visited prefix, and `export_and_verify_trunk_artifact` passes D11 binary round-trip verification (`round_trip_verified = true`).

---

## 5. Minimum Test Power, Bars & Decision Table

| Outcome | Condition | Decision & Unlocked Next Work |
|---|---|---|
| **Instrument Fault** | Any of `G-T1`..`G-T5` fails (`all_controls_pass = false`) | Classify as instrument (`CHARTER.md` §4.2), repair before grading, and re-run. |
| **Stage 3 PASS (`STAGE3_PASS_BEATS_STAGE2_BEST_ON_DEV_AND_FRESH`)** | Across both seeds (`Seed 1`, `Seed 2`) on both `ft/dev` (`14,287` rows) and `fresh_v2` (`14,287` rows): (1) `Level 1` beats `Level 0` (`455` / `371` on DEV) by $> \max(\text{seed\_noise}, \lceil 2\sqrt{b+c}\rceil)$; (2) `Level 2` beats `Level 1` (`2,140` / `1,602` on DEV) and `C_rec_null` by $> \text{seed\_noise}$; (3) `Level 3` beats Stage 2's best restricted table (`3,084` on `T16_fz`, `2,364` on `T16_rel` on DEV) and `Level 3 [C_rec_null]` by $> \text{seed\_noise}$ on both `ft/dev` and `fresh_v2`. | Set `#2844` `status: done`, unblocking `#2833` (`m1-address-as-read`), `#2867` (`m2-recurrence-regularization`), `#2865` (`m2-residual-allocation`), and the discrete branch of `#2851` (`m2-pin-train-mask-sheet`). |
| **Stage 3 Diagnostic Hold (`STAGE3_DIAGNOSTIC_HOLD`)** | `G-T1`..`G-T5` pass, but the Stage 3 bar is missed on `ft/dev` or `fresh_v2` | Record the negative configuration row in `docs/research/notebook/m1-discrete-attention.md` and route to `#2847` (`m1-wider-code-carrier`, $B = 8$). |

---

## 6. Pinned Inputs, Checkpoints, Cost, and Stop Rule

| Input / Checkpoint | Source Location | SHA-256 |
|---|---|---|
| Unchanged `driver_source_sha256` | `crates/uor-r4-discrete/src/training.rs` | `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` |
| Compiled `geometry_sha256` | `uor-r4-core` geometry tables | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| `ft/train/tokens.u16` & `response_mask.u8` | `caseyallard/uor-r4-store:inputs/m1-w2-capacity-capture-2026-10-25/` | `2b8a3f926db9be0fb5fc2fd37c6d39f4fb7f3b5f8fb3a2f2e0f6fb02d11b8b5b` (verified via config pins) |
| `T16_fz` Seed 1 / Seed 2 configs | `docs/labs/m1-bvg-k-stage-t-training/configs/` | `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d` / `d169d4ef347f28f4a289d131c3664e56a40a62a29774bf87cf0313ab358592cf` |
| `T16_rel` Seed 1 / Seed 2 configs | `docs/labs/m1-bvg-k-stage-t-training/configs/` | `ab41788a5e72a1f11370f875b090653e0d1b36be091393861b0e147688e9d746` / `a0bfa6c5fe7497d85d2a3d8a68d86157a8dcaa72a5d5b7eca193d904c480f263` |
| Stage T checkpoints archive (`stage-t-20261025.tar.gz`) | `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/` | `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6` |
| `T16_fz` S1 / S2 `stored_model_snapshot_sha256` | `t16-frozen-zero-seed{1,2}/final-checkpoint.json` | `2902d21ff792d17ebbd20dc8218dd4a815e38f0a4f1b49d273026854bb349fc1` / `d6375cf629873d2cf83a440c6bf36544634873e160fb9d6785560be5a4e61569` |
| `T16_rel` S1 / S2 `stored_model_snapshot_sha256` | `t16-relation-seed{1,2}/final-checkpoint.json` | `c98561bf05ce0421593c9872364c45c504843d30f0fa4e22b18471f0e38c5a18` / `848f13d1c168cf077ca00383a5b0f4f55dc18e87fa846d3d0ab73c20171504fb` |

- **Compute & Cost Budget**: `1x NVIDIA GeForce RTX 5090` (`~$0.76/h`) via `scripts/pod/uor-pod`, estimated wall time `~1.2 hours` (`~$0.95`), hard cap `3.0 hours` (`$3.00`), inside the global `<= 4` pods and `<= $8/h` caps.
- **Stop Rule**: Stop immediately if any Step 0 control (`G-T1`..`G-T5`) fails, if any GPU status word is non-zero, or if wall time approaches `3.0 hours`. Archive all artifacts to `caseyallard/uor-r4-store:results/antigravity/m1-trunk-assembly-and-training-2026-10-25/` before releasing and tearing down the pod.

---

## 7. TEST FITNESS

TEST FITNESS: FIT (Reviewed by independent fresh-context reviewer `65458246-8c07-489e-85d5-935d01db8259` on 2026-10-25; recorded in [`DECISION.md`](../../loop/tasks/m1-trunk-assembly-and-training/antigravity-89/DECISION.md) and [#2844 comment `6408200462`](https://github.com/UOR-Foundation/uor-r4/issues/2844#issuecomment-6408200462)).

