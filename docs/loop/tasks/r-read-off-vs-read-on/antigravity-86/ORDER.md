# ORDER: r-read-off-vs-read-on, antigravity-86

Lab: antigravity. Item: [r-read-off-vs-read-on](../../../../roadmap/items/r-read-off-vs-read-on.md) (#2885). Base: `db2770a2d`.

## Goal

The item's done-when reads:

> Paired read-on versus read-off results on the same retained checkpoints are recorded as notebook rows, with the classification instrument versus mechanism stated. The bar for a "read adds something" verdict is not yet registered; registering it, with the score it is read on, is part of How to start. For scale only: on the 14,287 development rows a context-free successor table scores 6,300 top-1, and the trained read scored 149.

In this unit (`antigravity-86`):
1. Register Option A (zero-update checkpoint-0 paired battery on `A8` cp0 and `P8` cp0, preserving `eval-corpus` at `main.rs:108-110`) and the 3-branch paired McNemar bar $F = \max(24, 2\sqrt{b+c})$ in `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md` via fresh-context D68 decision review (`CHARTER.md` §5.1, `D65`, `D68`, `D69`), non-author `TEST FITNESS: FIT` (`CHARTER.md` §4.5), and D67 pre-run review (`CHARTER.md` §5.10, `D67`).
2. Lease 1x RTX 5090 pod via `scripts/pod/uor-pod` inside global caps (`<= 4` pods, `<= $8.00/h`), execute `docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh` (`{a8, p8} x {r-on, r-off, r-perm, f-on, f-off}`) plus row-by-row paired McNemar discordances $(b, c)$ on the `14,287` full DEV and `3,847` `P_succ` rows, upload sealed artifacts to `caseyallard/uor-r4-store`, and bring the pod down.
3. Record the paired `Route` and `Flock` read-on vs read-off vs permuted results in `docs/labs/r-read-off-vs-read-on/README.md` and `docs/research/notebook/m1-discrete-attention.md`, set `status: done` on `docs/roadmap/items/r-read-off-vs-read-on.md` (`Closes #2885`), and re-render `docs/roadmap/README.md`.

## Cancel probe (run first)

Ran on base `db2770a2d`:
1. `uor-harness prep list | grep r-read-off-vs-read-on` -> `PENDING` (**classification: `instrument`**, sheet awaits D68 registration, non-author `FIT`, and D67 pre-run review).
2. `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target/antigravity ~/.cargo/bin/cargo test --no-fail-fast --locked -p uor-r4-discrete --test read_off_ablation_controls` -> `7 passed; 0 failed` (`C1`–`C7`, **classification: `instrument`**).
3. `scripts/pod/uor-pod status` -> `0/4 pods, $0/h of $8.00/h; 5090: 4 offers from $0.908/h` (**classification: `instrument`**).

## Plan (more than 3 files expected)

1. Update `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`, run fresh-context `TEST FITNESS: FIT` + D68 + D67 review, and post on `#2885`.
2. Lease 1x RTX 5090 via `scripts/pod/uor-pod`, run `read-off-battery.sh`, upload sealed artifact tarball to `caseyallard/uor-r4-store`, save summary receipt in `docs/labs/r-read-off-vs-read-on/evidence/summary.json`, and bring pod down.
3. Update `docs/labs/r-read-off-vs-read-on/README.md`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/r-read-off-vs-read-on.md` (`status: done`), `docs/roadmap/README.md`, and `REPORT.md`.

## Files in scope

- `docs/harness/prep/r-read-off-vs-read-on*`
- `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md`
- `docs/labs/r-read-off-vs-read-on/**`
- `docs/loop/tasks/r-read-off-vs-read-on/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/r-read-off-vs-read-on.md`
- `docs/roadmap/README.md`

## Done when

1. `uor-harness prep list` reports `FIT` for `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`.
2. The 10-arm GPU battery passes `G-Qual`, `G-Repro`, and `G-CrossMode` on a leased RTX 5090 pod, and paired `Route` and `Flock` rows are recorded in `docs/research/notebook/m1-discrete-attention.md`.
3. `uor-harness loop gates --full` and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any `G-Qual`, `G-Repro`, or `G-CrossMode` check fails on the GPU (halt immediately as a D31 instrument fault).
- Touching any file outside the declared scope is required.
