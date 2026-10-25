# REPORT: m4-flock-coupling-prereg, antigravity-85

- Lab: `antigravity`
- Item: [`m4-flock-coupling-prereg`](../../../../roadmap/items/m4-flock-coupling-prereg.md) (#2860)
- PR: #3240
- Base: `71fc6741519318fbc7c2cd35741d4238e8b768ce`

Closes #2860

## What did not work

- At `71fc6741519318fbc7c2cd35741d4238e8b768ce`, `uor-harness prep list | grep m4-flock-coupling-prereg` reported `NOT_FIT` (`instrument` finding) because `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` still had the five PM/owner registration questions from `#2860` comment `6352562539` and `deepseek-47` (`2026-10-23`) open in §7 and §8 (`PROPOSED, NOT REGISTERED`). Under `CHARTER.md` §5.1 (`D65`, `D68`, `D69`), an unanswered ask becomes the lab's own whole-project decision review, logged for the owner.
- Passing `--no-fail-fast` after `--` (`cargo test --locked -p uor-r4-discrete --test flock_coupling_controls -- --no-fail-fast`) was rejected by the libtest harness (`Unrecognized option: 'no-fail-fast'`); passing `--no-fail-fast` to `cargo test` (`cargo test --no-fail-fast --locked -p uor-r4-discrete --test flock_coupling_controls`) ran all `7 / 7` tests cleanly.

## What changed

- **Size:** 8 files changed (+165 / -62 lines; 0 Rust code lines, 0 test lines, 227 documentation/sheet/notebook/task lines; within 2x of the ~110-line estimate in `ORDER.md`).
- **`docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`:**
  1. **Decision 1 (`k_max > 1` under D11, §7.1):** Approved $k_{\max} \in \{2, 3\}$ applied by $|K_{b,\ell}|$ repeated additions (`add`/`sub` instructions only, zero float/multiply/divide instructions, `CHARTER.md` §3.6 D11 R1–R5), with $k_{\max} = 1$ (`A0`, `A2-prime`, `Z`) retained inside the 7-arm table as the restricted trit control and fallback.
  2. **Decision 2 (line identity & status, §7.2):** Confirmed C1 remains wanted (`flock-read` is `ACTIVE` at `docs/mechanisms/README.md:90`; `docs/mechanisms/flock-read.md:80` names C1 as step 1 of "What comes next") and registered it under M4 (`#2032`, `#2860`) with its result row in `docs/research/notebook/m1-discrete-attention.md`.
  3. **Decision 3 (base checkpoint binding, §7.3 & §8 `Base artifact`):** Bound the admission-preserving `#2831` `FrozenZero` restored passing checkpoint `continuation-05248.json` (`70935d37234a868ceeeff66a132fb4584e27a67710f06ec7b87db39358fbc36b`, `epoch = 274`, `TRAIN anchor admitted = 1,249 / 3,847 >= 1,187`; `Q1` `q1.json` `1e436e66a243cf7f9367c14372ef44ff1b9459fce98bf172100f28c5323f5ef7` with `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top1 = 871 / 14,287` in `a8-full-dose-rerun-20261025.tar.gz` `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`) and codebook `address_structured_v1` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`).
  4. **Decision 4 (D37 item-3 float export-gap forward, §7.4):** Confirmed the §5.2 offline GPU float forward in the D29-scanned module evaluates the float-minus-integer export gap on the unchanged integer writer and `14,287`-row DEV panel (`CHARTER.md` §3.7), is never served, and disqualifies (`EXPORT_LIMITED`) any candidate whose export gap exceeds $F_{\text{keep}}$.
  5. **Decision 5 (registration numbers & outcome decision table, §7.5 & §8):** Registered the `1,024`-row stratified TRAIN `C0` sample (`0` disagreements), $\ge 0.70$ (`717 / 1,024`) proxy-agreement threshold, DEV keep bar $F_{\text{keep}} = \max(24, |A_0(\text{seed }1) - A_0(\text{seed }2)|)$ over `A0`, `Z`, and `R` across 2 seeds, full 64-hex `TRAIN`/`DEV`/`fresh_v2` digests, expected effect size (`40–120` rows) vs $F_{\text{keep}}$, and the 4-branch outcome decision table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG` keeping `fresh_v2` sealed, `FAIL_EXPORT_GAP_DISQUALIFY`, `FAIL_TRANSFER_CLOSE_CONFIG`).
  6. **Non-author `TEST FITNESS: FIT` & D68 decision review (`1429e09e-7ea9-41e5-a697-cbb8ea55d611`):** Appended the fresh-context `TEST FITNESS: FIT` verdict at the end of `## Verdict` while retaining `deepseek-47` verbatim above it.
- **`docs/mechanisms/flock-read.md` and `docs/labs/m4-flock-coupling-prereg/README.md`:** Updated the C1 coupling sheet status to `REGISTERED; TEST FITNESS: FIT`.
- **`docs/research/notebook/m1-discrete-attention.md`:** Added the `2026-10-25` result row for `m4-flock-coupling-prereg` (`7 / 7` host control suites + `TEST FITNESS: FIT` 7-arm C1 pre-registration; GPU run `NOT_RUN`).
- **`docs/roadmap/items/m4-flock-coupling-prereg.md` and `docs/roadmap/README.md`:** Set `status: done` and re-rendered `docs/roadmap/README.md` (`92 items OK`).

## Measurements

- **Finite host controls (`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`, host CPU, 2026-10-25):**
  - Command: `~/.cargo/bin/cargo test --no-fail-fast --locked -p uor-r4-discrete --test flock_coupling_controls`
  - Result: `7 passed; 0 failed` (`C0` cached-tuple replay, `X1`–`X12` integer export kernel with exact `X7` counterfactual losses `[448, 448, 448, 256, 64, 256, 64]` on T′, `F1`–`F7` piecewise-linear proxy + D29 15-token banned scan + D32 integer export boundary).
- **Pre-registration fitness check (`uor-harness`, 2026-10-25):**
  - Command: `uor-harness prep list | grep m4-flock-coupling-prereg`
  - Result: `FIT docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`.
- **GPU training status:** `NOT_RUN` in this unit (`needs_gpu: false` pre-registration item).

## Checks run at the head

- `uor-harness prep list` — `FIT` on `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md`
- `~/.cargo/bin/cargo test --no-fail-fast --locked -p uor-r4-discrete --test flock_coupling_controls` — `7 passed; 0 failed`
- `~/.cargo/bin/cargo run --quiet -p xtask -- roadmap render && ~/.cargo/bin/cargo run --quiet -p xtask -- roadmap check` — `92 items OK`
- `~/.cargo/bin/cargo run --quiet -p xtask -- links` — `0 broken links`
- `~/.cargo/bin/cargo run --quiet -p xtask -- docs-catalog --check` — `OK`
- `uor-harness check` — `0 error`
- `uor-harness loop report --check --order docs/loop/tasks/m4-flock-coupling-prereg/antigravity-85/ORDER.md` — `PASS`
- `uor-harness loop pr-check --local` — `PASS`
- `uor-harness loop gates --full` — `11/11 GREEN`

## Skills used

- `scope-guard`: Kept edits strictly within `m4-flock-coupling-prereg.md`'s declared scope (`docs/harness/prep/m4-flock-coupling-prereg*`, `docs/mechanisms/flock-read.md`, `docs/labs/m4-flock-coupling-prereg/**`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m4-flock-coupling-prereg.md`, `docs/roadmap/README.md`, and `docs/loop/tasks/m4-flock-coupling-prereg/antigravity-85/**`).
- `solution-integrity`: Bound the C1 sheet to the admission-preserving `#2831` `FrozenZero` checkpoint (`continuation-05248.json` `70935d37…`) rather than the admission-eroded October checkpoints, and ran a fresh-context non-author `TEST FITNESS` and D68 decision reviewer (`1429e09e-7ea9-41e5-a697-cbb8ea55d611`).

## Still open

- **Registered 7-arm C1 GPU run (`A0`, `A1`, `A2`, `A2-int`, `A2-prime`, `Z`, `R`):** Executes on a GPU pod after its own D67 pre-run review inside global GPU caps, starting with device finite controls (`X1`–`X12`, `F1`–`F7`) and the `1,024`-row `C0` / proxy-agreement ($\ge 0.70$) pre-run instrument gates.

## Plain summary

The model's Flock read replaces transformer attention by rotating a few nearest earlier tokens into the current token's frame, snapping them to a consensus root vector, and adding that consensus to the current token's vector using a three-valued gate (`-1`, `0`, or `+1`). On some inputs (such as Fixture T′ in control `X7`), a single `+1` addition ties between the old root and the consensus root and fails to flip the output, whereas adding the consensus root twice (`+2`) or three times (`+3`) breaks the tie cleanly using only repeated integer addition (no multiplication, division, or softmax). This PR resolves all five open registration items on the C1 flock-coupling pre-registration sheet `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` under a fresh-context D68 whole-project decision review, binds it to the admission-preserving `#2831` `FrozenZero` checkpoint (`continuation-05248.json`), records its non-author `TEST FITNESS: FIT` review, logs the notebook row in `docs/research/notebook/m1-discrete-attention.md`, and closes `#2860`.
