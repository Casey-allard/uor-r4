# Lab Record: `m4-flock-coupling-prereg` — Flock Coupling (`FlockCoupled`) Design, Literal Re-Derivation, and Pre-Registration Staging

- **Date:** 2026-10-22
- **Lab:** `antigravity` (`antigravity-25`)
- **Issue / Milestone:** [`#2860`](https://github.com/UOR-Foundation/uor-r4/issues/2860) (`m4-flock-coupling-prereg`), M4 (`#2032`)
- **Design note & PM questions:** [`#2860` comment `6352562539`](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539)
- **Pre-registration sheet:** [`docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`](../../harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING` until the PM/owner registers the 5 open items on `#2860`)
- **Host control suite:** [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) (`7/7` tests PASS, `0` D23/D25/D32 ratchet sites)

---

## 1. Summary and Negative / Blocker Findings First

1. **Both October discrete attention runs ended in `Q1_STOP_FOR_DIAGNOSIS`, leaving no promoted Relation KEEP checkpoint:**
   - `docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md` stopped at batch `4,096` (`P_succ` admission `2,034 -> 225` Relation, `2,034 -> 205` control).
   - `docs/labs/a8-dose-2026-10-19/README.md` stopped the Relation arm at batch `4,096` (`P_succ` admission `1,351 -> 551`), while the `FrozenZero` control completed `16,404` batches (`1,351 -> 857`).
   - Consequently, binding the base checkpoint for C1 requires a PM decision between **(a)** waiting for the post-admission-guard A8 rerun (`#2832`) or **(b)** running C1 on the retained A8 `FrozenZero` / preflight checkpoint (`1,351 / 3,847` $P_{\text{succ}}$ admitted).
2. **Five registration items require PM/owner rulings (`#2860` comment `6352562539`):**
   - Whether $k_{\max} \in \{2, 3\}$ applied by $|K|$ repeated additions in `FlockCoupled` is admissible under D11;
   - Whether C1 remains wanted as the M4 starling/murmuration decisive run (`#2032`) or a sub-rung under `m1-discrete-attention` (`#2029`);
   - Which base checkpoint SHA-256 to bind;
   - Confirmation that the offline GPU float forward in the D29-scanned module is the D37 item-3 export-gap instrument; and
   - Approval of the proposed `C0` sample size (`1,024` TRAIN rows), proxy-agreement threshold ($\ge 0.70$), and DEV keep bar ($\Delta_{\text{top1}} \ge \max(24, \text{two-seed range of } A_0)$).
3. **Why $K = +1$ fails to flip Fixture T′ while $K = +2$ succeeds (`X7` exact writer losses):**
   - On Fixture T′ (`tokens = [[A, A], [A, B]]`, query `0 = [A, A]`, context `1 = [A, B]`, target `1 = [A, B]`), the query payload in block 1 is $u = A = r_0 = [2,2,0,0,0,0,0,0]$ and the transported consensus is $m_0 = B = r_{14} = [0,0,2,0,2,0,0,0]$.
   - At the trit ceiling $K = +1$ ($\text{terms} = 2$), $\operatorname{snap}_2(a + b)$ ties between $r_0 = A$ and $r_{14} = B$ at $(d_{L1} = 8, d_H = 4)$ and breaks the tie by lowest root index (`0 < 14`), returning `A` with writer loss **`448`**—identical to $K = 0$ (`448`) and $K = -1$ (`448`).
   - At $K = +2$ ($\text{terms} = 3$), the residual $[2,2,4,0,4,0,0,0]$ has unique minimum $D(r_{14}) = 8 < D(r_0) = 16$, flipping the output to `[A, B]` and dropping the writer loss from **`448` to `64`** (`tm = 8, am = 0`). Across all $2k_{\max}+1 = 7$ active ordinals `[CURRENT=0, -1, +1, -2, +2, -3, +3]`, the exact writer losses are **`[448, 448, 448, 256, 64, 256, 64]`** (with lanes `7..15` at `u64::MAX`), and `discrete_choose` selects ordinal `4` ($K = +2$, loss `64`).

---

## 2. Re-Verification Against Current `main` (`2987514aa`)

| Source / Component | Verified Location & Identity at `2987514aa` |
|---|---|
| Geometry generator | [`crates/uor-r4-discrete/build.rs`](../../../crates/uor-r4-discrete/build.rs) (`OPERATOR_KAPPA = blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b`, `COMPOSITION_KAPPA = blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0`) |
| `snap` device helper | [`crates/uor-r4-discrete/src/discrete.cu:102–110`](../../../crates/uor-r4-discrete/src/discrete.cu#L102-L110) |
| `flock_vector` device kernel | [`crates/uor-r4-discrete/src/discrete.cu:125–267`](../../../crates/uor-r4-discrete/src/discrete.cu#L125-L267) |
| `writer` (`NearestAddressMarginV1`) | [`crates/uor-r4-discrete/src/discrete.cu:307–345`](../../../crates/uor-r4-discrete/src/discrete.cu#L307-L345) |
| Existing 6 CUDA Flock controls | [`crates/uor-r4-discrete/src/flock_qualification.rs:109–356`](../../../crates/uor-r4-discrete/src/flock_qualification.rs#L109-L356) |
| New host control suite (`C0`, `X1`–`X12`, `F1`–`F7`) | [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) (`7/7` PASS) |

---

## 3. Verification Commands and Output

```sh
CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --test flock_coupling_controls --locked
uor-harness check
```

All `7` tests in `flock_coupling_controls` pass (`geometry_roots_and_bfs_match_rederived_prereg_literals`, `c0_cached_tuple_replay_equals_live_forward_and_refuses_absent_roots`, `x1_to_x5_and_x10_to_x12_integer_export_kernel_controls_match_literals`, `x6_and_x11_refusals_and_relation_trit_bounds_hold`, `x7_widened_counterfactual_lanes_match_derived_writer_losses_on_t_prime_and_c8`, `x8_empty_upper_level_and_x9_two_levels_at_once_match_derived_literals`, and `f1_to_f7_offline_proxy_and_export_gap_controls_match_literals`), and a deliberate break on `X7` (`448 -> 447`) was verified to fail before restoration.

---

## 4. Staged Research Notebook Row for `docs/research/notebook/m1-discrete-attention.md`

Because `docs/research/notebook/m1-discrete-attention.md` is outside the declared `scope:` of [`docs/roadmap/items/m4-flock-coupling-prereg.md`](../../roadmap/items/m4-flock-coupling-prereg.md), the six-column notebook row is staged here verbatim for backfill:

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-22 | Re-derived all C1 Flock-coupling (`FlockCoupled`) finite controls (`C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`) against current `build.rs` geometry on the laptop, derived the exact `X7` counterfactual writer losses (`[448, 448, 448, 256, 64, 256, 64]`) and `X8`/`X9` two-stage literals, and staged the complete pre-registration sheet (`$0`, no GPU). | 7 of 7 host control suites pass (`C0`, `X1`–`X12`, `F1`–`F7`); 0 model updates; 5 registration questions posted on `#2860` | All finite control derivations and `prep.sheet_shape` blocks pass; GPU run awaits PM registration and owner hold release | positive (pre-registration and finite control verification only; `TEST FITNESS: PENDING` on PM decisions) | [m4-flock-coupling-prereg](../../labs/m4-flock-coupling-prereg/README.md) |

## Non-author FIT review (deepseek-47, 2026-10-23)

Non-author `TEST FITNESS` review of the canonical sheet [`m4-flock-coupling-prereg-DRAFT-2026-10-22.md`](../../harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md): **NOT_FIT**, recorded in that sheet's `## Verdict` (status line + rendered `prep fit` block). No compute is cleared.

- The PM declined the scope widening at `#2028` comment `6362816839` and named the `-2026-10-22` sheet the canonical one; it supersedes the 10-18 draft (retained byte-identical) and matches the item scope glob, so no widening was needed. The earlier 10-18 sheet is untouched.
- **Verified:** the finite-control suite passes on current main (`cargo test -p uor-r4-discrete --test flock_coupling_controls` = **7 passed**); the re-derived control literals match the build-time geometry; the sheet is machine-complete with zero bar and zero run.
- **Remaining prerequisites (all PM/owner):** `k_max > 1` under D11; C1 line identity/decisive-run status; base-checkpoint binding after the two October Q1 stops; D37 item-3 confirmation for the float forward; and the registration numbers.
- The consolidated decision card with one recommendation per question is on #2860.
