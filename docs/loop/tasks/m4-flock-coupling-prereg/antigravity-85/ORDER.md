# ORDER: m4-flock-coupling-prereg, antigravity-85

Lab: antigravity. Item: [m4-flock-coupling-prereg](../../../../roadmap/items/m4-flock-coupling-prereg.md) (#2860). Base: `71fc67415`.

## Goal

Register `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` for `ReadSpec::FlockCoupled { kept, k_max }` (`#2860`), obtain a fresh-context D68 decision review and non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5, §5.1, D68), update `docs/mechanisms/flock-read.md` and `docs/labs/m4-flock-coupling-prereg/README.md`, add the notebook row in `docs/research/notebook/m1-discrete-attention.md`, and close `#2860` (`status: done`):

1. Resolve all five open decisions from `deepseek-47` (`2026-10-23`, §7 of the sheet) under D68 (`CHARTER.md` §5.1):
   - **D1 (`k_max > 1` under D11):** Confirm $k_{\max} \in \{2, 3\}$ applied by $|K_{b,\ell}|$ repeated additions (`add`/`sub` only, zero multiply/divide, D11 R1–R5) is admissible under D11, with $k_{\max} = 1$ (`A0`, `A2-prime`, `Z`) as restricted trit control.
   - **D2 (line identity & status):** Confirm C1 remains wanted (`flock-read` `ACTIVE` at `docs/mechanisms/README.md:90`; `docs/mechanisms/flock-read.md:80`) and registers under M4 (`#2032`, `#2860`) with its result row in `docs/research/notebook/m1-discrete-attention.md`.
   - **D3 (base checkpoint binding):** Bind the `#2831` `FrozenZero` restored passing checkpoint `continuation-05248.json` (`70935d37…`, `epoch = 274`, `TRAIN anchor admitted = 1,249 / 3,847`, `Q1` `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV top1 = 871 / 14,287`) and `address_structured_v1` codebook (`6600bdb1306d…`).
   - **D4 (D37 item-3 float export-gap forward):** Confirm the §5.2 offline GPU float forward on the unchanged integer writer and `14,287`-row DEV panel is the D37 item-3 export-gap instrument (never served, never promotes, disqualifies any candidate whose float-minus-integer gap exceeds `A0`'s two-seed range).
   - **D5 (registration numbers & outcome table):** Register the `1,024`-row stratified TRAIN `C0` sample (`0` disagreements), $\ge 0.70$ (`717 / 1,024`) proxy-agreement threshold, DEV keep bar ($\Delta_{\text{top1}} \ge \max(24, |A_0(\text{seed }1) - A_0(\text{seed }2)|)$ over `A0`, `Z`, `R` on both seeds on `P_succ` and full DEV with non-decreasing admission), fresh-V2 (`0b7e754c…`) replication ($b - c \ge \lceil 2\sqrt{b+c}\rceil$), and 4-branch outcome table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG`, `FAIL_EXPORT_GAP_DISQUALIFY`, `FAIL_TRANSFER_CLOSE_CONFIG`).
2. Append the fresh-context `TEST FITNESS: FIT` verdict at the end of `## Verdict` in the sheet.
3. Update `docs/mechanisms/flock-read.md` and `docs/labs/m4-flock-coupling-prereg/README.md`, add the notebook row in `docs/research/notebook/m1-discrete-attention.md`, set `m4-flock-coupling-prereg.md` to `status: done`, and re-render `docs/roadmap/README.md`.

## Cancel probe (run first)

At `71fc6741519318fbc7c2cd35741d4238e8b768ce`:
- `uor-harness prep list | grep m4-flock-coupling-prereg` reports `NOT_FIT`.
- `cargo test --no-fail-fast --locked -p uor-r4-discrete --test flock_coupling_controls` passes `7 / 7` tests.
- **Cancel probe result (`instrument`):** Proceed with D68 registration and `TEST FITNESS: FIT` review.

## Files in scope

- `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`
- `docs/mechanisms/flock-read.md`
- `docs/labs/m4-flock-coupling-prereg/**`
- `docs/roadmap/items/m4-flock-coupling-prereg.md`
- `docs/roadmap/README.md`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/loop/tasks/m4-flock-coupling-prereg/antigravity-85/**`

## Done when

- `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` passes D68 decision review and non-author `TEST FITNESS: FIT` review, `uor-harness prep list` reports `FIT`, and `uor-harness loop gates --full` is green (~110 lines across 7 files).

## Stop and ask if

- Any edit would touch files outside `Files in scope` — stop and decide under D68.
