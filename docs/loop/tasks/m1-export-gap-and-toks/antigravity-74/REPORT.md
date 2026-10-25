# REPORT: m1-export-gap-and-toks, antigravity-74

Lab: antigravity. Item: [m1-export-gap-and-toks](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839). Order: [ORDER.md](ORDER.md).
References #2839.

## What did not work

- **Umbrella-lock refusal on `uor-harness loop claim m1-export-gap-and-toks` (`instrument` — cross-item scope overlap with idle draft PR `#3169`):**
  - `uor-harness loop claim m1-export-gap-and-toks --lab-topic m1-export-gap-fit` refused (`exit 2`) because `docs/mechanisms/lut-serving.md` in `m1-export-gap-and-toks.md`'s `scope:` overlaps `docs/mechanisms/lut-serving.md` in the 35-glob `scope:` of the umbrella item `infra-loop-harness` (`PR #3169`, `codex-1`).
  - Verified via `gh pr view 3169 --json files,isDraft,updatedAt` that PR `#3169` (`codex-1`, idle draft since `2026-10-25T01:07:47Z`) modifies only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/**` (`0` overlapping files with `m1-export-gap-and-toks`), and claimed under the umbrella-lock exception (`run-antigravity.txt` Part 3 Step 2).
- **Direct Stage 4 GPU evaluation & laptop throughput run is blocked on Step 4 (`m1-bvg-k-stage-t-training`, `#2836`) producing the trained discrete checkpoint (`instrument` — upstream checkpoint dependency, not a mechanism negative):**
  - `PLAN.md` Step 6 (`m1-export-gap-and-toks`, `#2839`) requires both the discrete D11 exporter (`#2838`, `done`) and a trained discrete model from Step 4 (`m1-bvg-k-stage-t-training`, `#2836`). Because `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`) ended in an instrument `STOP_FOR_DIAGNOSIS` (`consecutive_rejections` at batch `1,600` on `Relation` and batch `5,248` on `FrozenZero`) and `#2836` has not yet run while `crates/uor-r4-discrete/**` is locked by `claude-103` (PR `#3203`), this unit completes Step 3 (`How to start`: non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` and D68 registration of the Stage 4 bar, two-seed noise band, and `status: held` until `#2836` completes).

## What changed

- **Size (`uor-harness loop pr-check --local` / `git diff --stat origin/main...HEAD`):** 6 files changed (`tests 0 file(s) +0/-0, code 0 file(s) +0/-0, docs 5 file(s) +114/-16, generated 1 file(s) +3/-3` across `ORDER.md`, `REPORT.md`, `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`, `docs/labs/m1-export-gap-and-toks/README.md`, `docs/roadmap/items/m1-export-gap-and-toks.md`, and `docs/roadmap/README.md`).
- **Pre-registration sheet (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`):**
  - Updated for D67/D68 (`CHARTER.md` §5.1, §6) and `PLAN.md` Step 4 -> Step 6 dependency (`m1-bvg-k-stage-t-training`, `#2836`), added `- **Run:** m1-export-gap-and-toks-stage4`, and stamped `TEST FITNESS: FIT` after fresh-context non-author review (`8a915285-22c1-4fcf-b336-9f292d2a5b7d`, [`#2839#issuecomment-6398709883`](https://github.com/UOR-Foundation/uor-r4/issues/2839#issuecomment-6398709883)).
- **Stage 4 lab record (`docs/labs/m1-export-gap-and-toks/README.md`):**
  - Updated with the `#2831` (`m1-a8-full-dose-rerun`) Q1 outcome (`FrozenZero` Q1 `DEV top1_correct = 871 / 14,287`, `b_3(P_succ) = 1,106 / 3,847`), the `TEST FITNESS: FIT` verdict, and the D68 registration of `B1..B4` and the K16 two-seed noise band.
- **D68 status transition (`docs/roadmap/items/m1-export-gap-and-toks.md`, `docs/roadmap/README.md`):**
  - Following fresh-context D68 decision review (`8a915285-22c1-4fcf-b336-9f292d2a5b7d`, `DECISION REVIEW: APPROVE`, logged at [`#2839#issuecomment-6398709883`](https://github.com/UOR-Foundation/uor-r4/issues/2839#issuecomment-6398709883)), set `status: held` (`status_only_edit` per `crates/uor-r4-harness/src/loopprcheck.rs:488`) pending `m1-bvg-k-stage-t-training` (`#2836`) and re-rendered `docs/roadmap/README.md`.
- **Suborders issued:** `local/antigravity/antigravity-74/orders/1.md` (read-only fresh-context non-author `TEST FITNESS` and D68 decision review by subagent `8a915285-22c1-4fcf-b336-9f292d2a5b7d`) and `local/antigravity/antigravity-74/orders/2.md` (fresh-context D66 exact-head review of PR `#3220`).

## Measurements

- **Registered Stage 4 export-gap bars and K16 two-seed empirical noise bands (`docs/labs/k16-outcome-call-2026-10-19/README.md`):**
  - Frozen DEV panel: `14,287` scored target positions (`3,847` `P_succ` rows; `ft/dev/tokens.u16` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `ft/dev/response_mask.u8` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`).
  - Table-native lossless discrete export (`uor-r4.lut-discrete/1`): required export gap is exact `0` (`\Delta_{\text{top1}} = 0`, `\Delta_{\text{top8}} = 0`, `\Delta_{\text{adm}} = 0`, `\Delta_{\text{loss}} = 0`, `14,287 / 14,287` row agreement).
  - Offline float-trained candidate (`D37`, `CHARTER.md` §3.7): bounded by the K16 two-seed noise band `\Delta_{\text{noise,top1}} = |463 - 65| = 398` rows (`2.79%`), `\Delta_{\text{noise,top8}} = |1,295 - 547| = 748` rows (`5.24%`), and `\Delta_{\text{noise,adm}} = \max(|3,167 - 2,237|, |12,466 - 12,089|, 87) = 930` rows (`6.51%`).
- **Pre-registration completeness check (`uor-harness check`):** `0` errors on `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` (`8 / 8` required blocks present, `TEST FITNESS: FIT`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap render`: `PASS` (`92` items rendered)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: `PASS` (`92 items OK`)
- `uor-harness loop report --check --order docs/loop/tasks/m1-export-gap-and-toks/antigravity-74/ORDER.md`: `PASS`
- `uor-harness loop pr-check --local`: `PASS` (`0` errors, `0` warnings)
- `uor-harness loop gates --full`: `PASS` (`11 / 11` gates `GREEN`)

## Skills used

- `scope-guard`: Verified zero file overlap with idle draft PR `#3169` (`codex-1`) before claiming under the `infra-loop-harness` umbrella exception, and restricted the item edit on `docs/roadmap/items/m1-export-gap-and-toks.md` to a `status_only_edit` (`crates/uor-r4-harness/src/loopprcheck.rs:488`).
- `solution-integrity`: Completed the non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5) and D68 whole-project decision review (`CHARTER.md` §5.1) before registering the Stage 4 bar/noise band and setting `status: held`.
- `gates`: Verified `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` (`11 / 11` `GREEN`).

## Still open

- **Training the Step 4 discrete model (`m1-bvg-k-stage-t-training`, `#2836`):** Once `#2836` completes on two seeds (`seed = 1`, `seed = 2`), `#2839` (`m1-export-gap-and-toks`) runs immediately via a D67 pre-run review (`CHARTER.md` §5.1) under the FIT-cleared sheet `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` to export the trained checkpoints to `UORLUT01`, verify the zero-multiply static audits (`B1`) and `14,287`-row DEV export gap (`B2`) plus planted-fault control (`B4`), and measure separate prompt-reading (`192`-token prefill) and reply-writing (`64`-token decode) `tokens/s` on the client laptop (`B3`).

## Plain summary

Stage 4 of Milestone M1 (`m1-export-gap-and-toks`, `#2839`, Step 6 in `PLAN.md`) tests two things when a trained model is exported into the project's whole-number serving file format (`UORLUT01`, governed by rule D11, which forbids floating-point numbers, multiplications, and divisions in the served model): first, the **export gap** (how many predictions change on the `14,287`-row development test panel `ft/dev` after export), and second, **tokens per second** (`tok/s`) on a client laptop, measured separately for reading a 192-token prompt (context prefill) and writing a 64-token reply (autoregressive decode). Earlier work (`antigravity-42`) drafted the one-page pre-registration sheet `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`, but left it at `TEST FITNESS: PENDING` before decisions D67 and D68 gave each lab authority to complete its own fresh-context fitness and bar/status reviews. In this unit (`antigravity-74`), we updated the pre-registration sheet and companion lab record for D67/D68 and the recent A8 rerun (`#2831`), ran a fresh-context non-author review that cleared the sheet as `TEST FITNESS: FIT`, and passed a fresh-context whole-project decision review (D68) registering the Stage 4 decisive criteria (`B1..B4`, requiring exact zero gap across all `14,287` rows for lossless discrete exports and bounding offline float exports by the empirical two-seed K16 noise band of `398` top-1 rows) and setting `#2839` to `status: held` until Step 4 (`m1-bvg-k-stage-t-training`, `#2836`) produces the trained model checkpoints.
