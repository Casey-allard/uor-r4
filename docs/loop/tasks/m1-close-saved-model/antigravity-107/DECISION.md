# DECISION CARD (D68): Scope Expansion for `m1-close-saved-model` (`antigravity-107`)

- **Date:** 2026-10-26
- **Lab:** `antigravity`
- **Unit:** `antigravity-107` (PR `#3308`)
- **Item:** [`m1-close-saved-model`](../../../../roadmap/items/m1-close-saved-model.md) (`#2837` / Milestone M1 `#2029`)
- **Authority:** `CHARTER.md` §5.1 (`D68`, `D69`)

## 1. Whole-Project Context & Why This Decision Is Needed

Under `CHARTER.md` §2 ("1. **Model:** the geometric model, exported as one D11 integer/table artifact; built in `crates/uor-r4-discrete`, served by `crates/uor-r4-integer`") and `CHARTER.md` §4.8 (`D39`), closing Milestone M1 (`m1-close-saved-model`, `#2837` / `#2029`, Step 8 of `PLAN.md`) requires a saved discrete model evaluated through the clean-room `UORLUT01` integer reader (`crates/uor-r4-integer::discrete_d11::IntegerDiscreteModel`) to clear all five M1 closing bars (a)–(e).

In `antigravity-105` (PR `#3301`) and `antigravity-106` (PR `#3304`, `bb38c1e93bf8cb1d0e22028fe47113a57214079e`), we established and proved on `crates/uor-r4-discrete::trunk`:
1. The 4-table Stage 2 `UORLUT01` (`uor-r4.lut-discrete/1`) reader is structurally bounded by its pre-rotation 8-cell shortlist `A_W(q_read) = 5,009 < T2 = 6,300` and single-code ceiling `T(c_read) = 4,085 < 6,300`.
2. The Path A post-rotation joint-pointer trunk (`trunk-joint-pointer-v1-d11`, `export_trunk_d11` / `import_trunk_d11`, 19-table `UORLUT01` `uor-r4.lut-discrete-trunk/1`) clears:
   - **Bar (b):** `0` export gap between `export_trunk_d11` / `import_trunk_d11` and `evaluate_joint_pointer_full_writer`, plus planted-fault `B4` rejection of corrupted magic, corrupted tail token, and trailing bytes.
   - **Bar (c):** `34,325`–`34,337 context tok/s` GPU on `ft/dev`.
   - **Bar (d):** Two-seed `ft/dev` (`14,287` rows) `top-1` = **`7,386 / 14,287` (`51.70%`, `+1,086` over `T2 = 6,300`)** on `T16_fz` Seed 1 and **`7,374 / 14,287` (`51.61%`, `+1,074` over `T2 = 6,300`)** on `T16_fz` Seed 2 using `roots_final` (`snapshot.table`), and **`7,702` / `7,701`** using `roots0`, with `P_succ = 2,090 / 2,086 > 1,245` (`T2_psucc`), and stored `#2844` `fresh_v2` (`14,287` rows) = **`7,703` / `7,722` (`+938` / `+957` over `T2_fresh = 6,765`)**.
3. The sole remaining blocker for **Bar (a)** (`d11_reader_accepted = true` and `d11_export_includes_full_scoring_path = true` in the clean-room `crates/uor-r4-integer` serving crate) and laptop `d11-scorer toks` serving throughput (`prefill_median_tok_s` and `decode_median_tok_s` under `CHARTER.md` §3.6) is that `IntegerDiscreteModel::from_bytes` in `crates/uor-r4-integer/src/discrete_d11.rs:702` hardcodes `header.schema == "uor-r4.lut-discrete/1"` and `header.tables.len() == 4`.

However, `docs/roadmap/items/m1-close-saved-model.md` currently declares:
```yaml
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-close-saved-model*, docs/labs/m1-close-saved-model/**, docs/loop/tasks/m1-close-saved-model/**]
```
Without adding `crates/uor-r4-integer/**` (and `docs/labs/m1-d11-artifact-scorer/instrument/**`, already approved in `antigravity-105` D68 card `#issuecomment-6423438792`) to the scope of `m1-close-saved-model`, no unit on `m1-close-saved-model` can extend `IntegerDiscreteModel::from_bytes` and `IntegerDiscreteModel::evaluate_prefix` in `crates/uor-r4-integer/src/discrete_d11.rs` to read the 19-table `uor-r4.lut-discrete-trunk/1` container.

## 2. Decision

Under `CHARTER.md` §5.1 (`D68`), expand the scope of `m1-close-saved-model` (`docs/roadmap/items/m1-close-saved-model.md` and `docs/loop/tasks/m1-close-saved-model/antigravity-107/ORDER.md`) to include:
- `crates/uor-r4-integer/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/roadmap/items/m1-close-saved-model.md`

No active open PR by any other lab overlaps `crates/uor-r4-integer/src/discrete_d11.rs` (`kimi/d11-float-trunk` PR `#3306` is scoped to `crates/uor-r4-Amber/**` / `crates/uor-r4-training/**` / `docs/labs/d11-exporter-float-trunk/**`, and `claude/loop-pr13-Verify` PR `#3307` is scoped to `crates/uor-r4-harness/**`).

## 3. Compliance with `CHARTER.md` §3 (Technical Constraints)

- **Zero changes to `CHARTER.md`, `§3`, or GPU caps.**
- **`#![forbid(unsafe_code)]`, integer-only (`D11 R1–R5`):** The 19-table reader and evaluator in `crates/uor-r4-integer/src/discrete_d11.rs` use zero floating-point types or literals (`D32`), zero squashing or exponential maps (`D23`), zero dense matrix multiplication (`D25`), and pure bit-shift hierarchy scoring `(c_qr << 6) + (c_qc << 5) + (c_rr << 3) + (c_r << 1) + c_q + (ptr_match << 8)` (`0` general integer multiplies, `0` divides).
- **Rejected alternative:** Leaving `crates/uor-r4-integer` at 4 tables (`uor-r4.lut-discrete/1`), which permanently blocks Path A on Bar (a) (`d11_reader_accepted = false`) despite clearing Bars (b), (c), and (d).
- **Cost to reverse:** One revert commit restoring `crates/uor-r4-integer/src/discrete_d11.rs` and `docs/roadmap/items/m1-close-saved-model.md`.

## 4. Fresh-Context D68 Decision Review Record (`2d1f0194-35c2-489c-9525-454ef83e2284`)

- **Verdict:** `D68 DECISION REVIEW: APPROVE`
- **Reviewer:** Fresh-context non-author `research` subagent (`2d1f0194-35c2-489c-9525-454ef83e2284`, 2026-10-26T22:24:53Z)
- **Summary:**
  1. **Whole-Project Chain:** Verified against `CHARTER.md` §2.1, §4.8 (`D39`), and `PLAN.md` Step 8 (`m1-close-saved-model`, `#2837`). Expanding scope to include `crates/uor-r4-integer/**` and `docs/roadmap/items/m1-close-saved-model.md` directly addresses the `header.tables.len() != 4` blocker in `crates/uor-r4-integer/src/discrete_d11.rs:702` so the clean-room `UORLUT01` integer reader can accept and evaluate the 19-table `uor-r4.lut-discrete-trunk/1` artifact for Bar (a) and `CHARTER.md` §3.6 serving `tokens/s`.
  2. **Charter §3 & Owner Boundaries:** Touches zero owner-only paths (`docs/harness/owner-paths.json`) and preserves `#![forbid(unsafe_code)]`, `D11 R1–R5` integer-only serving, `D32` zero floats, `D23` zero squashing/softmax, `D25` zero dense matmul, and bit-shift-only hierarchy scoring.
  3. **No Overlap:** Verified no active PR by another lab overlaps `crates/uor-r4-integer/src/discrete_d11.rs`.
  4. **Reversibility:** Rejected alternative and one-commit cost to reverse are explicitly documented.
