# REPORT: m1-normalization-mechanism, antigravity-84

- Lab: `antigravity`
- Item: [`m1-normalization-mechanism`](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842)
- PR: #3238
- Base: `dcc4fe4435e9b2f9cea0e130954e685757fedefa`

Closes #2842

## What did not work

- Placing the new `TEST FITNESS: FIT` section first under `## Verdict` (above the two historical `TEST FITNESS: NOT_FIT` records from `deepseek-37` and `claude-57`) initially caused `uor-harness prep list` to still classify `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` as `NOT_FIT` (`instrument` finding). Inspection of `crates/uor-r4-harness/src/prep.rs:97-120` confirmed that `prep::test_fitness` takes the **last** `TEST FITNESS:` line in a sheet (`// The LAST verdict line wins: sheets append a later review below earlier ones`). Appending the `antigravity-84` `TEST FITNESS: FIT` block at the bottom of `## Verdict` resolved this immediately so `uor-harness prep list` reports `FIT`.

## What changed

- **Size:** 5 files changed (+122 / -49 lines; 0 Rust code lines, 0 test lines, 171 documentation/sheet/notebook/task lines; within the ~120-line estimate in `ORDER.md`, ratio `1.02x`).
- **`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`:**
  1. **Prerequisite 1 (`claude-57`):** Cited the merged discrete normalization entry points and scorers from `claude-64` (`#3113`), `claude-66` (`#3116`), `claude-67` (`#3119`), `claude-68` (`#3121`), and `claude-72` (`#3131`) (`discrete-stack normalization-build-identity`, `normalization-check-plan`, `normalization-owner-release`, `normalization`, `normalization-resume`, `normalization-dev-structure`, `normalization-dev-b3`, `normalization-dev-paired`) across `ScaleMatchedSnapV1` (`N_snap`), `UnitSnapControlV1` (`C_unscaled`), and `UnnormalizedControlV1` (`C_raw`) and seeds `1` and `2`, running `N_snap` under its own dedicated scope `m1-normalization-n_snap-injective-6600bdb1306d-relation-k8-seed{1|2}-batches0..16404`.
  2. **Prerequisite 2 (`claude-57`):** Pinned §4 to a single configuration — the discrete M1 A8 proxy trunk at `dcc4fe4435e9b2f9cea0e130954e685757fedefa` (`DRIVER_SOURCE_SHA256 = cb686691d136260e3a8d0b7b1a820cbeb4934cce273e225e06b8e32783a814cd`, codebook `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) — and deferred the assembled Stage 3 trunk (`m1-trunk-assembly-and-training`, `#2844`, M2 under D79) to its own future pre-registration sheet.
  3. **Prerequisite 3 (`claude-57`):** Quoted the owner's `2026-10-23T02:36:36Z` bar registration (`#2028` comment `6356034555`) and `2026-10-24T03:41:41Z` bar-term ruling (`#2028` comment `6379430093`) in §3, fixing the strict non-inferiority margin against `C_unscaled` ($b - c > -F_I$ where $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$, scored in `uor-r4.normalization-dev-paired/2`) and the fresh-V2 replication threshold ($b - c \ge \lceil 2\sqrt{b + c}\rceil$, no 24-row floor).
  4. **Prerequisite 4 & non-blocking items 5–7 (`claude-57`):** Added the 3-branch outcome decision table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG` keeping fresh V2 sealed, `FAIL_TRANSFER_CLOSE_CONFIG`) in §3 and §7, pinned the full 64-hex `TRAIN`/`DEV`/`fresh_v2` SHA-256 digests in §5, updated §6 commands, and stated expected effect size ($\Delta_{\text{expected}} \ge 50\text{–}150$ rows) vs $F_I$ in §8.
  5. **Non-author `TEST FITNESS: FIT` & D68 decision review (`fd1dd468-ae67-4ea2-938d-8af9c36811df`):** Appended the fresh-context `TEST FITNESS: FIT` verdict under `## Verdict` while retaining `deepseek-37` and `claude-57` verbatim above it.
- **`docs/research/notebook/m1-discrete-attention.md`:** Added the `2026-10-25` result row for `OrbitRootSnapNormV1` (`20 / 20` normalization unit controls + `3 / 3` paired DEV scorer controls + `TEST FITNESS: FIT` pre-registration on the A8 proxy trunk; GPU trunk training `NOT_RUN`).
- **`docs/roadmap/items/m1-normalization-mechanism.md` and `docs/roadmap/README.md`:** Set `status: done` and re-rendered `docs/roadmap/README.md` (`92 items OK: ready 29, held 16, decision 1, done 46`).

## Measurements

- **Finite normalization controls (`crates/uor-r4-discrete`, host CPU, 2026-10-25):**
  - Command: `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib normalization -- --no-fail-fast`
  - Result: `20 passed; 0 failed` (`C-N1`..`C-N5` idempotence, exact L1 norm `32` per block / `128` full vector, deterministic tie-breaking, positive scale preservation, zero-block invariant, 3-mode separation `N_snap` vs `C_unscaled` vs `C_raw`, snapshot persistence, device-host parity).
- **Paired row-level DEV scorer controls (`crates/uor-r4-discrete`, host CPU, 2026-10-25):**
  - Command: `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib dev_paired -- --no-fail-fast`
  - Result: `3 passed; 0 failed` (`b`, `c`, `d = b - c`, strict non-inferiority `b - c > -F_I`).
- **Pre-registration fitness check (`uor-harness`, 2026-10-25):**
  - Command: `uor-harness prep list | grep m1-normalization-mechanism`
  - Result: `FIT docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`.
- **GPU training status:** `NOT_RUN` in this unit (`needs_gpu: false` design + pre-registration item).

## Checks run at the head

- `uor-harness prep list` — `FIT` on `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib normalization -- --no-fail-fast` — `20 passed; 0 failed`
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib dev_paired -- --no-fail-fast` — `3 passed; 0 failed`
- `~/.cargo/bin/cargo run --quiet -p xtask -- roadmap render && ~/.cargo/bin/cargo run --quiet -p xtask -- roadmap check` — `92 items OK`
- `~/.cargo/bin/cargo run --quiet -p xtask -- links` — `0 broken links`
- `~/.cargo/bin/cargo run --quiet -p xtask -- docs-catalog --check` — `OK`
- `uor-harness check` — `0 error`
- `uor-harness loop report --check --order docs/loop/tasks/m1-normalization-mechanism/antigravity-84/ORDER.md` — `PASS`
- `uor-harness loop pr-check --local` — `PASS`
- `uor-harness loop gates --full` — `11/11 GREEN`

## Skills used

- `scope-guard`: Kept edits strictly within `m1-normalization-mechanism.md`'s declared scope (`docs/harness/prep/m1-normalization-mechanism*`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-normalization-mechanism.md`, `docs/roadmap/README.md`, and `docs/loop/tasks/m1-normalization-mechanism/antigravity-84/**`).
- `solution-integrity`: Verified how `prep::test_fitness` (`crates/uor-r4-harness/src/prep.rs:97-120`) resolves multiple `TEST FITNESS:` verdicts in a single sheet before committing, and ran a fresh-context non-author `TEST FITNESS` and D68 decision reviewer (`fd1dd468-ae67-4ea2-938d-8af9c36811df`) on the sheet and its sources.

## Still open

- **Registered 3-arm GPU trunk run (`N_snap` vs `C_unscaled` and `C_raw`):** With `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` now `FIT`, the next step before dispatching the 3-arm GPU run (`6 x 16,404` batches across 2 seeds) is the one-line `NORMALIZATION_SHEET` SHA-256 update in `crates/uor-r4-discrete/src/driver_source_pin.rs` and a D67 pre-run review inside global GPU caps.
- **Assembled Stage 3 trunk (`m1-trunk-assembly-and-training`, `#2844`, M2):** Evaluated under its own future pre-registration sheet once position (`#2843`), normalization (`#2842`), and Stage T (`#2836`) are ready.

## Plain summary

A language model with multiple stages needs a way to keep its internal number lists ("vectors", here 32 whole numbers split into four 8-number blocks) from growing too large or shrinking to zero as signals add up across stages. Standard neural networks use floating-point division and square roots ("layer normalization") or exponentials ("softmax"), which `CHARTER.md` §3 forbids. `OrbitRootSnapNormV1` (`ScaleMatchedSnapV1`) instead snaps each non-zero 8-number block to the closest of the 16 fixed vectors in the model's geometric root orbit (`O_16`, where each root has four entries of `+2` or `-2` and four entries of `0`, giving a fixed block sum of magnitudes `L1 = 8`), using only integer addition, subtraction, and absolute values (`L1` distance, then sign-agreement count, then lowest root index `0..15` to break ties deterministically). This PR updates the one-page pre-registration sheet `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` with all landed GPU entry points and owner bar rulings, records its fresh-context non-author `TEST FITNESS: FIT` and D68 decision review, logs the result row in `docs/research/notebook/m1-discrete-attention.md`, and closes `#2842`.
