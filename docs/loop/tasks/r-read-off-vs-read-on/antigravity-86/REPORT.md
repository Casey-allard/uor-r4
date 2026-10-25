# REPORT: r-read-off-vs-read-on, antigravity-86

Lab: antigravity. Item: [r-read-off-vs-read-on](../../../../roadmap/items/r-read-off-vs-read-on.md) (#2885). ORDER: [ORDER.md](./ORDER.md).

## What did not work

- **Uncalibrated `Route` read-on (`R-On`, `routes = 71cdce57…`) hurts cp0 cell admission (`READ_HURTS_ADMISSION_OR_RANKING`, classification: `mechanism`):** On `A8` cp0 (`address_structured_v1`, `6600bdb1…`), `a8-r-on` drops full DEV cell admission from `5,818 / 14,287` (`a8-r-off`) to `748 / 14,287` (`b=349, c=5419, b-c=-5070 < -152`), `P_succ` from `1,643 / 3,847` to `408 / 3,847` (`b-c=-1235 < -82`), `top8` from `450` to `74` (`b-c=-376 < -46`), and `top1` from `22` to `0` (`b-c=-22`, $|b-c|\le 24$, `READ_INERT_AT_CHECKPOINT`), while beating group-deranged `a8-r-perm` on admission (`748` vs `574`, `b-c=+174 > 47`). On `P8` cp0 (`PrimeZeta`, `83c1380a…`), `p8-r-on` drops full admission from `415` to `263` (`b-c=-152 < -52`) and `P_succ` from `118` to `84` (`b-c=-34 < -28`), and is indistinguishable from `p8-r-perm` (`263` vs `261`, `b-c=+2 <= 42`). Replacing the last-position query vector $x_{\text{length}-1}$ with an uncalibrated tree-routed leaf discards `A8`'s predecessor-to-target cell alignment. Under D22 (`CHARTER.md` §4.1), this closes uncalibrated cp0 `Route` (`routes = 71cdce57…`), never the `route-read` mechanism.
- **`Flock` read-on (`ActiveLeafV1`, `a8-f-on`) narrows `A8` cp0 cell shortlist admission (`READ_HURTS_ADMISSION_OR_RANKING` on `target_admitted`, classification: `mechanism`):** Adding level-0 leaf consensus (`u = query[b] + m_0`) before cell shortlist selection shifts `1,896` boundary queries out of their predecessor-placed target cells while admitting `322` new rows (`4,244` vs `5,818`, `b-c=-1574 < -95` full; `1,351` vs `1,643`, `b-c=-292 < -48` `P_succ`).
- **Attempt 1/2 `read-off-battery.sh` shell checks (`classification: instrument`):** Attempt 1 (`r-read-off-vs-read-on-20261025`) completed all 10 GPU arm evaluations cleanly, then exited `8` at line 69 of `read-off-battery.sh` because it queried `.aggregate.top1_hits` instead of `.aggregate.top1_correct`. Per L-002, Attempt 1 was left untouched and Attempt 2 ran into a fresh exclusive root `r-read-off-vs-read-on-20261025-r2` (`FILES-r2.sha256` `41dab02c…`), matching Attempt 1 bit-for-bit across all 10 arms (`10 / 10`) and exiting `0`. D66 review (`eb55cdde`) then caught that `read-off-battery.sh` line 27 passed unqualified `route_read_off_and_permuted_controls_run_and_refuse` with `--exact`, filtering out `336` tests (`0 passed`); fixed line 27 to `device::unit_snap_device_tests::route_read_off_and_permuted_controls_run_and_refuse` + `grep -q '1 passed; 0 failed'` and ran the test on RTX 5090 `vast-57715285` (`1 passed; 0 failed in 0.94s`, `g-qual-device-test-r3.stdout` `722cc3b4…`).
- **Unrelated prep-sheet CI lint on `main` (`classification: instrument`):** Fixed Section 6 heading and standalone `TEST FITNESS: FIT` line in `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md` (`f9246910c`).

## What changed

- Registered Option A and the 3-branch paired McNemar bar in `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md` (`TEST FITNESS: FIT`, D68, D67).
- Ran the 10-arm GPU battery on `vast-57708111`, saved [`docs/labs/r-read-off-vs-read-on/evidence/summary.json`](../../../labs/r-read-off-vs-read-on/evidence/summary.json), updated [`docs/labs/r-read-off-vs-read-on/README.md`](../../../labs/r-read-off-vs-read-on/README.md) and [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md), and marked `r-read-off-vs-read-on` `status: done`.

## Measurements

- **Hardware & store:** `1 x RTX 5090` (`vast-57708111`, driver `595.84`, `~$0.23`), `discrete-stack` `00688b0d…`, archived in `caseyallard/uor-r4-store:results/antigravity/r-read-off-vs-read-on-2026-10-25/` (tarball `cbd29504000c465a3837c735caa51b25566c175c3af14e49c06e5baa69f65e6e`).
- **Throughput:** `Route` arms `1,948–2,041 payload tok/s` (`649–680 rows/s`, `21–22 s`); `Flock` arms `1,429–1,587 payload tok/s` (`476–529 rows/s`, `27–30 s`).
- **D31 gates:** `G-Qual: PASS`, `G-Repro: PASS` (`4244/1351/133` and `427/127/38`), `G-CrossMode: PASS` (`r-off == f-off` on `14,287/14,287` rows on `a8` and `p8`).
- **`Flock` (`ActiveLeafV1` vs `ZeroGatesV1`, `classification: mechanism`):** **`READ_ADDS_SIGNAL`** on within-cell ranking (`A8 top1` `133` vs `22`, `b-c=+111 > 24`, `6.05x`; `A8 P_succ top1` `62` vs `0`, `b-c=+62 > 24`; `A8 top8` `+63` full / `+36` `P_succ`; `P8 top8` `95` vs `30`, `b-c=+65 > 24`, `3.17x` full and `30` vs `1`, `b-c=+29 > 24`, `30.0x` `P_succ`).

## Checks run at the head

- `cargo test --no-fail-fast --locked -p uor-r4-discrete --test read_off_ablation_controls` -> `7 passed`.
- `uor-harness loop gates --full`, `uor-harness loop report --check`, `uor-harness loop pr-check --local` -> `PASS`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`.

## Still open

- None for `#2885` (`status: done`). Follow-on work decouples cell admission from `Flock` token ranking (`#2860`) and runs batched `BVG-K` Stage T training (`#2836`).

## Plain summary

Completed the 10-arm zero-update GPU read-off vs read-on comparison (`#2885`) on `A8` and `P8` checkpoint-0 models on `1 x RTX 5090`: `Flock` (`ActiveLeafV1`) adds `6.05x` top-1 signal (`133` vs `22` full, `62` vs `0` `P_succ`) while narrowing `A8` cell admission (`4,244` vs `5,818`), whereas uncalibrated `Route` replaces the last-position query anchor and hurts cell admission (`748` vs `5,818`).

Closes #2885
